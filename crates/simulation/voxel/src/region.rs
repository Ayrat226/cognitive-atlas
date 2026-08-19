//! Region management (16×16×16 chunks = 512³ blocks)

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::coords::{RegionKey, ChunkKey, CHUNK_SIZE, REGION_SIZE, REGION_BLOCK_SIZE};
use crate::chunk::{PaletteChunk, Chunk, CHUNK_VOLUME, ChunkStorage};
use crate::block::BlockState;
use crate::lod::LodAggregate;
use lithos_engine_memory::GLOBAL_STATS;
use lithos_engine_serialization::{SerializationError, Reader as SerializationReader};

/// Region storage - manages chunks within a 512³ block region
pub struct Region {
    pub key: RegionKey,
    /// Chunks stored as sparse hash map (only non-empty chunks)
    chunks: RwLock<HashMap<ChunkKey, Chunk>>,
    /// LOD aggregate data for this region
    pub lod_aggregate: RwLock<Option<LodAggregate>>,
    /// Last modification tick
    pub last_modified: RwLock<u64>,
    /// Whether region has unsaved changes
    pub dirty: RwLock<bool>,
}

impl Region {
    pub fn new(key: RegionKey) -> Self {
        Self {
            key,
            chunks: RwLock::new(HashMap::new()),
            lod_aggregate: RwLock::new(None),
            last_modified: RwLock::new(0),
            dirty: RwLock::new(false),
        }
    }

    /// Get chunk, generating procedurally if not present
    pub fn get_chunk(&self, chunk_key: ChunkKey, generator: Option<&crate::procedural::ProceduralGenerator>, tick: u64) -> Chunk {
        let mut chunks = self.chunks.write();
        
        if let Some(chunk) = chunks.get(&chunk_key) {
            return chunk.clone();
        }

        // Generate chunk procedurally
        let generated = if let Some(gen) = generator {
            gen.generate_chunk(chunk_key)
        } else {
            Chunk::empty(tick)
        };

        chunks.insert(chunk_key, generated.clone());
        *self.dirty.write() = true;
        *self.last_modified.write() = tick;
        generated
    }

    /// Get chunk if loaded (no generation)
    pub fn get_chunk_loaded(&self, chunk_key: ChunkKey) -> Option<Chunk> {
        self.chunks.read().get(&chunk_key).cloned()
    }

    /// Set chunk (for modifications)
    pub fn set_chunk(&self, chunk_key: ChunkKey, chunk: Chunk) {
        let mut chunks = self.chunks.write();
        chunks.insert(chunk_key, chunk);
        *self.dirty.write() = true;
    }

    /// Remove chunk (mark as empty)
    pub fn remove_chunk(&self, chunk_key: ChunkKey) {
        let mut chunks = self.chunks.write();
        chunks.remove(&chunk_key);
        *self.dirty.write() = true;
    }

    /// Get block at world position within this region
    pub fn get_block(&self, x: u32, y: u32, z: u32, generator: Option<&crate::procedural::ProceduralGenerator>, tick: u64) -> BlockState {
        let chunk_x = x / CHUNK_SIZE as u32;
        let chunk_y = y / CHUNK_SIZE as u32;
        let chunk_z = z / CHUNK_SIZE as u32;
        
        let local_x = x % CHUNK_SIZE as u32;
        let local_y = y % CHUNK_SIZE as u32;
        let local_z = z % CHUNK_SIZE as u32;

        let chunk_key = ChunkKey::new(
            (self.key.x * REGION_SIZE) + chunk_x as i32,
            (self.key.y * REGION_SIZE) + chunk_y as i32,
            (self.key.z * REGION_SIZE) + chunk_z as i32,
        );

        let chunk = self.get_chunk(chunk_key, generator, 0);
        chunk.data.get_block(local_x, local_y, local_z)
    }

    /// Set block at world position within this region
    pub fn set_block(&self, x: u32, y: u32, z: u32, block: BlockState, tick: u64) -> bool {
        let chunk_x = x / CHUNK_SIZE as u32;
        let chunk_y = y / CHUNK_SIZE as u32;
        let chunk_z = z / CHUNK_SIZE as u32;
        
        let local_x = x % CHUNK_SIZE as u32;
        let local_y = y % CHUNK_SIZE as u32;
        let local_z = z % CHUNK_SIZE as u32;

        let chunk_key = ChunkKey::new(
            (self.key.x * REGION_SIZE) + chunk_x as i32,
            (self.key.y * REGION_SIZE) + chunk_y as i32,
            (self.key.z * REGION_SIZE) + chunk_z as i32,
        );

        let mut chunks = self.chunks.write();
        let chunk = chunks.get_mut(&chunk_key);
        
        let changed = if let Some(existing) = chunk {
            let mut data = existing.data.take_chunk();
            let changed = data.set_block(local_x, local_y, local_z, block);
            *existing = Chunk::new(data, tick);
            changed
        } else {
            // Generate base chunk then modify
            let mut base = crate::procedural::ProceduralGenerator::default().generate_chunk(chunk_key);
            let mut data = base.data.take_chunk();
            let changed = data.set_block(local_x, local_y, local_z, block);
            chunks.insert(chunk_key, Chunk::new(data, tick));
            changed
        };

        if changed {
            *self.dirty.write() = true;
            *self.last_modified.write() = tick;
        }
        changed
    }

    /// Get all loaded chunk keys
    pub fn loaded_chunks(&self) -> Vec<ChunkKey> {
        self.chunks.read().keys().cloned().collect()
    }

    /// Get number of loaded chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.read().len()
    }

    /// Iterate over chunks for memory stats
    pub fn iter_chunks<F>(&self, mut f: F) 
    where
        F: FnMut(&Chunk),
    {
        let chunks = self.chunks.read();
        for chunk in chunks.values() {
            f(chunk);
        }
    }

    /// Check if region is dirty (has unsaved changes)
    pub fn is_dirty(&self) -> bool {
        *self.dirty.read()
    }

    /// Mark region as clean (saved)
    pub fn mark_clean(&self) {
        *self.dirty.write() = false;
    }

    /// Update LOD aggregate
    pub fn update_lod(&self) {
        let chunks = self.chunks.read();
        if chunks.is_empty() {
            *self.lod_aggregate.write() = None;
            return;
        }

        // Calculate aggregate from loaded chunks
        let mut aggregate = LodAggregate::new(self.key);
        for (_, chunk) in chunks.iter() {
            aggregate.accumulate_chunk(&chunk.data);
        }
        aggregate.finalize();
        *self.lod_aggregate.write() = Some(aggregate);
    }

    /// Get LOD aggregate
    pub fn get_lod(&self) -> Option<LodAggregate> {
        self.lod_aggregate.read().clone()
    }

    /// Serialize region for saving
    pub fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), SerializationError> {
        let chunks = self.chunks.read();
        
        // Write chunk count
        buf.extend_from_slice(&(chunks.len() as u32).to_le_bytes());
        
        for (key, chunk) in chunks.iter() {
            // Write chunk key
            buf.extend_from_slice(&key.x.to_le_bytes());
            buf.extend_from_slice(&key.y.to_le_bytes());
            buf.extend_from_slice(&key.z.to_le_bytes());
            
            // Write chunk data
            let chunk_bytes = chunk.data.to_bytes();
            buf.extend_from_slice(&(chunk_bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(&chunk_bytes);
        }
        
        Ok(())
    }

    /// Deserialize region
    pub fn deserialize(buf: &[u8]) -> Result<Self, SerializationError> {
        let mut reader = SerializationReader::new(buf);
        let chunk_count = reader.read::<u32>()? as usize;
        
        let mut chunks = HashMap::with_capacity(chunk_count);
        for _ in 0..chunk_count {
            let x = reader.read::<i32>()?;
            let y = reader.read::<i32>()?;
            let z = reader.read::<i32>()?;
            let key = ChunkKey::new(x, y, z);
            
            let data_len = reader.read::<u32>()? as usize;
            let data_bytes = reader.read_bytes(data_len)?;
            let data = PaletteChunk::from_bytes(data_bytes).map_err(|e| SerializationError::InvalidData(e))?;
            let chunk = Chunk::new(data, 0);
            chunks.insert(key, chunk);
        }
        
        // Note: Need region key from context
        Ok(Region {
            key: RegionKey::zero(),
            chunks: RwLock::new(chunks),
            lod_aggregate: RwLock::new(None),
            last_modified: RwLock::new(0),
            dirty: RwLock::new(false),
        })
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::new(RegionKey::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{BlockId, BlockState};

    #[test]
    fn test_region_basic() {
        let region = Region::new(RegionKey::new(0, 0, 0));
        assert_eq!(region.key, RegionKey::zero());
        assert_eq!(region.chunk_count(), 0);
    }

    #[test]
    fn test_region_get_block() {
        let region = Region::new(RegionKey::new(0, 0, 0));
        let block = region.get_block(10, 20, 30, None, 0);
        assert_eq!(block.block_id, crate::block::BlockId::AIR);
    }

    #[test]
    fn test_region_set_block() {
        let region = Region::new(RegionKey::new(0, 0, 0));
        let changed = region.set_block(10, 20, 30, BlockState::solid(crate::block::BlockId::STONE), 1);
        assert!(changed);
        assert!(region.is_dirty());
    }
}