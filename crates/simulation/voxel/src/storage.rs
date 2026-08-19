//! Main voxel storage system
//! Sparse hash map: RegionKey → Region → ChunkKey → Chunk

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::coords::{RegionKey, ChunkKey, BlockPos, REGION_BLOCK_SIZE};
use crate::block::BlockState;
use crate::region::Region;
use crate::chunk::{Chunk};
use crate::procedural::{ProceduralGenerator, GenerationParams};
use crate::delta_log::{DeltaLog, DeltaEntry};
use crate::lod::{LodManager, LodAggregate};
use lithos_engine_math::FixedVec3;
use serde::{Serialize, Deserialize};
use lithos_engine_jobs::parallel_for;
use lithos_engine_serialization::{SerializationError, Writer, Reader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VoxelStorageError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    #[error("Region not found: {0:?}")]
    RegionNotFound(RegionKey),
    #[error("Chunk not found: {0:?}")]
    ChunkNotFound(ChunkKey),
    #[error("Invalid coordinates: {0}")]
    InvalidCoords(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Delta log error: {0}")]
    DeltaLog(#[from] crate::delta_log::DeltaLogError),
}

/// Main voxel storage
pub struct VoxelStorage {
    /// Regions stored sparsely
    regions: RwLock<HashMap<RegionKey, Arc<Region>>>,
    /// Procedural generator for base terrain
    generator: ProceduralGenerator,
    /// Delta log for persistent modifications
    delta_log: RwLock<DeltaLog>,
    /// LOD manager for aggregation
    lod_manager: RwLock<LodManager>,
    /// Current tick (for ordering)
    current_tick: RwLock<u64>,
    /// Save directory
    save_dir: std::path::PathBuf,
}

impl VoxelStorage {
    /// Create new voxel storage
    pub fn new(save_dir: std::path::PathBuf, seed: u64) -> Self {
        let params = GenerationParams {
            seed,
            ..Default::default()
        };
        
        Self {
            regions: RwLock::new(HashMap::new()),
            generator: ProceduralGenerator::new(params),
            delta_log: RwLock::new(DeltaLog::new_default()),
            lod_manager: RwLock::new(LodManager::new()),
            current_tick: RwLock::new(0),
            save_dir,
        }
    }

    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        *self.current_tick.read()
    }

    /// Advance tick
    pub fn advance_tick(&self) -> u64 {
        let mut tick = self.current_tick.write();
        *tick += 1;
        *tick
    }

    /// Get or create region
    fn get_or_create_region(&self, region_key: RegionKey) -> Arc<Region> {
        let mut regions = self.regions.write();
        regions.entry(region_key)
            .or_insert_with(|| Arc::new(Region::new(region_key)))
            .clone()
    }

    /// Get region if loaded
    fn get_region(&self, region_key: RegionKey) -> Option<Arc<Region>> {
        self.regions.read().get(&region_key).cloned()
    }

    /// Get block at world position
    pub fn get_block(&self, pos: BlockPos) -> BlockState {
        let region_key = pos.region_key();
        
        if let Some(region) = self.get_region(region_key) {
            let local = self.world_to_region_local(pos, region_key);
            return region.get_block(local.0, local.1, local.2, Some(&self.generator), 0);
        }

        // Generate procedurally
        let (rx, ry, rz) = self.world_to_region_local(pos, region_key);
        self.generator.generate_block(
            (region_key.x * REGION_BLOCK_SIZE + rx as i32) as f32,
            (region_key.y * REGION_BLOCK_SIZE + ry as i32) as f32,
            (region_key.z * REGION_BLOCK_SIZE + rz as i32) as f32,
        )
    }

    /// Get block at world position (with region generation)
    pub fn get_block_mut(&self, pos: BlockPos) -> BlockState {
        let region_key = pos.region_key();
        let region = self.get_or_create_region(region_key);
        let (rx, ry, rz) = self.world_to_region_local(pos, region_key);
        region.get_block(rx, ry, rz, Some(&self.generator), *self.current_tick.read())
    }

    /// Set block at world position
    pub fn set_block(&self, pos: BlockPos, block: BlockState) -> BlockChange {
        let region_key = pos.region_key();
        let region = self.get_or_create_region(region_key);
        let (rx, ry, rz) = self.world_to_region_local(pos, region_key);
        
        let tick = self.advance_tick();
        let old_block = region.get_block(rx, ry, rz, Some(&self.generator), tick);
        let changed = region.set_block(rx, ry, rz, block, tick);
        
        if changed {
            // Log delta
            self.delta_log.write().push(DeltaEntry {
                pos,
                old_block,
                new_block: block,
                tick,
                author: 0, // TODO: player ID
            });
            
            // Invalidate LOD for this region
            self.lod_manager.write().invalidate_region(region_key);
        }

        BlockChange {
            pos,
            old_block,
            new_block: block,
            changed,
            tick,
        }
    }

    /// Convert world position to region-local coordinates
    fn world_to_region_local(&self, pos: BlockPos, region_key: RegionKey) -> (u32, u32, u32) {
        let region_min = region_key.min_block_pos();
        let rx = (pos.x - region_min.x) as u32;
        let ry = (pos.y - region_min.y) as u32;
        let rz = (pos.z - region_min.z) as u32;
        (rx, ry, rz)
    }

    /// Get chunk at key
    pub fn get_chunk(&self, chunk_key: ChunkKey) -> Chunk {
        let region_key = chunk_key.region_key();
        let region = self.get_or_create_region(region_key);
        region.get_chunk(chunk_key, Some(&self.generator), *self.current_tick.read())
    }

    /// Get loaded chunk keys
    pub fn loaded_chunks(&self) -> Vec<ChunkKey> {
        self.regions.read()
            .values()
            .flat_map(|r| r.loaded_chunks())
            .collect()
    }

    /// Get all dirty regions
    pub fn dirty_regions(&self) -> Vec<RegionKey> {
        self.regions.read()
            .iter()
            .filter(|(_, r)| r.is_dirty())
            .map(|(k, _)| *k)
            .collect()
    }

    /// Save dirty regions
    pub fn save_dirty(&self) -> Result<usize, VoxelStorageError> {
        let dirty_keys = self.dirty_regions();
        let mut saved = 0;
        
        for key in dirty_keys {
            if let Some(region) = self.get_region(key) {
                self.save_region(&region)?;
                region.mark_clean();
                saved += 1;
            }
        }
        
        // Save delta log
        self.delta_log.write().save(&self.save_dir.join("deltas.log"))?;
        
        Ok(saved)
    }

    /// Save single region
    fn save_region(&self, region: &Region) -> Result<(), VoxelStorageError> {
        let path = self.save_dir.join(format!("region_{}_{}_{}.bin", region.key.x, region.key.y, region.key.z));
        std::fs::create_dir_all(&self.save_dir)?;
        
        let mut buf = Vec::new();
        region.serialize(&mut buf)?;
        std::fs::write(&path, buf)?;
        
        Ok(())
    }

    /// Load region from disk
    pub fn load_region(&self, region_key: RegionKey) -> Result<Arc<Region>, VoxelStorageError> {
        let path = self.save_dir.join(format!("region_{}_{}_{}.bin", region_key.x, region_key.y, region_key.z));
        
        if !path.exists() {
            return Err(VoxelStorageError::RegionNotFound(region_key));
        }
        
        let data = std::fs::read(&path)?;
        let mut region = Region::deserialize(&data)?;
        region.key = region_key;
        
        let region_arc = Arc::new(region);
        self.regions.write().insert(region_key, region_arc.clone());
        
        Ok(region_arc)
    }

    /// Unload region (save if dirty)
    pub fn unload_region(&self, region_key: RegionKey) -> Result<(), VoxelStorageError> {
        if let Some(region) = self.get_region(region_key) {
            if region.is_dirty() {
                self.save_region(&region)?;
            }
            self.regions.write().remove(&region_key);
        }
        Ok(())
    }

    /// Raycast through voxel world
    pub fn raycast(&self, origin: FixedVec3, direction: FixedVec3, max_distance: f32) -> Option<RaycastHit> {
        use lithos_engine_math::{Fixed, FixedVec3};
        
        let step = Fixed::from_f32(0.5); // 0.5m steps
        let max_steps = (max_distance / step.to_f32()) as u32;
        let dir_norm = direction.normalize();
        
        let mut pos = origin;
        let mut prev_block = BlockPos::zero();
        
        for _ in 0..max_steps {
            pos = pos + dir_norm * step;
            let block_pos = BlockPos::from_fixed_vec3(pos);
            
            if block_pos != prev_block {
                let block = self.get_block(block_pos);
                if !block.block_id.is_air() {
                    return Some(RaycastHit {
                        pos: block_pos,
                        block,
                        normal: self.estimate_normal(block_pos),
                        distance: (pos - origin).length().to_f32(),
                    });
                }
                prev_block = block_pos;
            }
        }
        
        None
    }

    /// Estimate surface normal at block position
    fn estimate_normal(&self, pos: BlockPos) -> lithos_engine_math::Vec3 {
        let mut nx = 0.0f32;
        let mut ny = 0.0f32;
        let mut nz = 0.0f32;
        
        for (dx, dy, dz) in &[
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ] {
            let neighbor = pos.offset(*dx, *dy, *dz);
            let block = self.get_block(neighbor);
            if !block.block_id.is_air() {
                nx += *dx as f32;
                ny += *dy as f32;
                nz += *dz as f32;
            }
        }
        
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 0.0 {
            lithos_engine_math::Vec3::new(nx / len, ny / len, nz / len)
        } else {
            lithos_engine_math::Vec3::new(0.0, 1.0, 0.0)
        }
    }

    /// Update LOD aggregates
    pub fn update_lod(&self) {
        let mut lod = self.lod_manager.write();
        
        for (key, region) in self.regions.read().iter() {
            if lod.should_update(*key, *self.current_tick.read()) {
                region.update_lod();
                if let Some(aggregate) = region.get_lod() {
                    lod.update(*key, aggregate);
                }
            }
        }
    }

    /// Get LOD aggregate for region
    pub fn get_lod(&self, region_key: RegionKey) -> Option<LodAggregate> {
        self.lod_manager.read().get(region_key).cloned()
    }

    /// Memory usage stats
    pub fn memory_stats(&self) -> VoxelMemoryStats {
        let regions = self.regions.read();
        let mut total_chunks = 0;
        let mut total_memory = 0;
        
        for region in regions.values() {
            total_chunks += region.chunk_count();
            region.iter_chunks(|chunk| {
                total_memory += chunk.data.memory_usage();
            });
        }
        
        VoxelMemoryStats {
            region_count: regions.len(),
            chunk_count: total_chunks,
            estimated_memory_bytes: total_memory,
            delta_log_entries: self.delta_log.read().len(),
        }
    }
}

/// Result of a block change
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockChange {
    pub pos: BlockPos,
    pub old_block: BlockState,
    pub new_block: BlockState,
    pub changed: bool,
    pub tick: u64,
}

/// Raycast hit result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaycastHit {
    pub pos: BlockPos,
    pub block: BlockState,
    pub normal: lithos_engine_math::Vec3,
    pub distance: f32,
}

/// Memory statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VoxelMemoryStats {
    pub region_count: usize,
    pub chunk_count: usize,
    pub estimated_memory_bytes: usize,
    pub delta_log_entries: usize,
}

impl Default for VoxelStorage {
    fn default() -> Self {
        Self::new(std::path::PathBuf::from("saves"), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{BlockId, BlockState};

    #[test]
    fn test_voxel_storage_basic() {
        let temp_dir = std::env::temp_dir().join("lithos_test_voxel");
        let storage = VoxelStorage::new(temp_dir.clone(), 42);
        
        // Test get/set
        let pos = BlockPos::new(100, 64, 100);
        let old = storage.get_block_mut(pos);
        assert_eq!(old.block_id, BlockId::AIR);
        
        let change = storage.set_block(pos, BlockState::solid(BlockId::STONE));
        assert!(change.changed);
        assert_eq!(change.new_block.block_id, BlockId::STONE);
        
        let new = storage.get_block(pos);
        assert_eq!(new.block_id, BlockId::STONE);
    }

    #[test]
    fn test_procedural_generation() {
        let temp_dir = std::env::temp_dir().join("lithos_test_voxel2");
        let storage = VoxelStorage::new(temp_dir.clone(), 123);
        
        // Test procedural generation at various heights
        let surface = storage.get_block(BlockPos::new(0, 70, 0));
        assert!(!surface.block_id.is_air());
        
        let air = storage.get_block(BlockPos::new(0, 100, 0));
        assert_eq!(air.block_id, BlockId::AIR);
    }

    #[test]
    fn test_raycast() {
        let temp_dir = std::env::temp_dir().join("lithos_test_raycast");
        let storage = VoxelStorage::new(temp_dir.clone(), 42);
        
        // Place stone column
        for y in 50..80 {
            storage.set_block(BlockPos::new(10, y, 10), BlockState::solid(BlockId::STONE));
        }
        
        // Raycast from above
        let hit = storage.raycast(
            crate::coords::BlockPos::new(10, 100, 10).to_fixed_vec3(),
            lithos_engine_math::FixedVec3::new(
                lithos_engine_math::Fixed::ZERO,
                -lithos_engine_math::Fixed::ONE,
                lithos_engine_math::Fixed::ZERO,
            ),
            100.0,
        );
        
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert!(hit.distance > 20.0 && hit.distance < 50.0);
        assert_eq!(hit.block.block_id, BlockId::STONE);
    }
}