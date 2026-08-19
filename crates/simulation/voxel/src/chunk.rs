//! Chunk storage with palette + RLE compression

use std::fmt;
use crate::coords::CHUNK_SIZE;
use crate::block::{BlockId, BlockState};
use serde::{Serialize, Deserialize};
use lithos_engine_serialization::{TypeId, SerializationError, Writer, Reader};

/// Total blocks per chunk (32³ = 32768)
pub const CHUNK_VOLUME: usize = (CHUNK_SIZE as usize).pow(3);

/// Chunk coordinate to linear index (z * 32² + y * 32 + x)
#[inline]
pub fn block_index(x: u32, y: u32, z: u32) -> usize {
    (z as usize) * 1024 + (y as usize) * 32 + (x as usize)
}

/// Linear index to chunk coordinates
#[inline]
pub fn block_coords(index: usize) -> (u32, u32, u32) {
    let z = index / 1024;
    let y = (index % 1024) / 32;
    let x = index % 32;
    (x as u32, y as u32, z as u32)
}

/// Palette-based chunk storage
/// Uses palette for unique block states + RLE for runs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaletteChunk {
    /// Palette of unique block states in this chunk
    pub palette: Vec<BlockState>,
    /// RLE-compressed data: (run_length, palette_index)
    pub runs: Vec<(u16, u16)>,
    /// Whether chunk is empty (all air)
    pub empty: bool,
    /// Whether chunk is uniform (single block type)
    pub uniform: bool,
    /// Uniform block if uniform
    pub uniform_block: BlockState,
}

impl PaletteChunk {
    /// Create empty chunk
    pub fn empty() -> Self {
        Self {
            palette: vec![],
            runs: vec![],
            empty: true,
            uniform: true,
            uniform_block: BlockState::air(),
        }
    }

    /// Create uniform chunk (all same block)
    pub fn uniform(block: BlockState) -> Self {
        Self {
            palette: vec![block],
            runs: vec![(CHUNK_VOLUME as u16, 0)],
            empty: block.block_id.is_air(),
            uniform: true,
            uniform_block: block,
        }
    }

    /// Create from raw block states (for testing/procedural gen)
    pub fn from_blocks(blocks: &[BlockState]) -> Self {
        if blocks.is_empty() {
            return Self::empty();
        }

        // Check if uniform
        let first = blocks[0];
        if blocks.iter().all(|b| *b == first) {
            return Self::uniform(first);
        }

        // Build palette
        let mut palette = Vec::new();
        let mut palette_map = std::collections::HashMap::new();
        let mut runs: Vec<(u16, u16)> = Vec::new();

        for block in blocks {
            let idx = *palette_map.entry(*block).or_insert_with(|| {
                let idx = palette.len() as u16;
                palette.push(*block);
                idx
            });

            if let Some(last) = runs.last_mut() {
                if last.1 == idx && last.0 < u16::MAX {
                    last.0 += 1;
                } else {
                    runs.push((1, idx));
                }
            } else {
                runs.push((1, idx));
            }
        }

        Self {
            palette,
            runs,
            empty: false,
            uniform: false,
            uniform_block: BlockState::air(),
        }
    }

    /// Get block at linear index
    #[inline]
    pub fn get(&self, index: usize) -> BlockState {
        if self.empty {
            return BlockState::air();
        }
        if self.uniform {
            return self.uniform_block;
        }

        let mut pos = 0;
        for (run_len, palette_idx) in &self.runs {
            let run_len = *run_len as usize;
            if index < pos + run_len {
                return self.palette[*palette_idx as usize];
            }
            pos += run_len;
        }
        BlockState::air() // Should not happen
    }

    /// Set block at linear index (returns new chunk - immutable)
    pub fn set(&self, index: usize, block: BlockState) -> Self {
        if self.empty && block.is_air() {
            return self.clone();
        }
        if self.uniform {
            if block == self.uniform_block {
                return self.clone();
            }
            // Convert uniform to full palette
            let mut blocks = vec![self.uniform_block; CHUNK_VOLUME];
            blocks[index] = block;
            return Self::from_blocks(&blocks);
        }

        // For simplicity, convert to blocks and rebuild
        let mut blocks: Vec<BlockState> = (0..CHUNK_VOLUME).map(|i| self.get(i)).collect();
        blocks[index] = block;
        Self::from_blocks(&blocks)
    }

    /// Get all blocks as vector (for mesh generation)
    pub fn to_blocks(&self) -> Vec<BlockState> {
        if self.empty {
            return vec![BlockState::air(); CHUNK_VOLUME];
        }
        if self.uniform {
            return vec![self.uniform_block; CHUNK_VOLUME];
        }

        let mut blocks = Vec::with_capacity(CHUNK_VOLUME);
        for (run_len, palette_idx) in &self.runs {
            let block = self.palette[*palette_idx as usize];
            blocks.extend(std::iter::repeat(block).take(*run_len as usize));
        }
        blocks
    }

    /// Memory usage estimate
    pub fn memory_usage(&self) -> usize {
        let palette_bytes = self.palette.len() * std::mem::size_of::<BlockState>();
        let runs_bytes = self.runs.len() * std::mem::size_of::<(u16, u16)>();
        palette_bytes + runs_bytes + 32 // overhead
    }

    /// Check if chunk needs mesh generation
    pub fn needs_mesh(&self) -> bool {
        !self.empty && (!self.uniform || !self.uniform_block.block_id.is_air())
    }
}

/// Chunk with version for serialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub version: u32,
    pub data: PaletteChunk,
    pub modified_tick: u64,
}

impl Chunk {
    pub fn new(data: PaletteChunk, tick: u64) -> Self {
        Self {
            version: 1,
            data,
            modified_tick: tick,
        }
    }

    pub fn empty(tick: u64) -> Self {
        Self::new(PaletteChunk::empty(), tick)
    }

    pub fn uniform(block: BlockState, tick: u64) -> Self {
        Self::new(PaletteChunk::uniform(block), tick)
    }
}

/// Chunk storage interface for voxel storage
pub trait ChunkStorage {
    fn get_block(&self, x: u32, y: u32, z: u32) -> BlockState;
    fn set_block(&mut self, x: u32, y: u32, z: u32, block: BlockState) -> bool;
    fn get_chunk(&self) -> &PaletteChunk;
    fn take_chunk(&mut self) -> PaletteChunk;
    fn is_empty(&self) -> bool;
    fn is_uniform(&self) -> bool;
    fn needs_mesh(&self) -> bool;
}

impl ChunkStorage for PaletteChunk {
    #[inline]
    fn get_block(&self, x: u32, y: u32, z: u32) -> BlockState {
        self.get(block_index(x, y, z))
    }

    #[inline]
    fn set_block(&mut self, x: u32, y: u32, z: u32, block: BlockState) -> bool {
        let new_chunk = self.set(block_index(x, y, z), block);
        let changed = !std::ptr::eq(self, &new_chunk);
        *self = new_chunk;
        changed
    }

    #[inline]
    fn get_chunk(&self) -> &PaletteChunk {
        self
    }

    #[inline]
    fn take_chunk(&mut self) -> PaletteChunk {
        std::mem::take(self)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.empty
    }

    #[inline]
    fn is_uniform(&self) -> bool {
        self.uniform
    }

    #[inline]
    fn needs_mesh(&self) -> bool {
        self.needs_mesh()
    }
}

impl Default for PaletteChunk {
    fn default() -> Self {
        Self::empty()
    }
}

/// Binary serialization for PaletteChunk
impl PaletteChunk {
    /// Serialize to bytes using a simple binary format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        
        // Header: empty flag, uniform flag
        buf.push(if self.empty { 1 } else { 0 });
        buf.push(if self.uniform { 1 } else { 0 });
        if self.uniform {
            // Uniform block: block_id (2 bytes) + metadata (1 byte)
            buf.extend_from_slice(&self.uniform_block.block_id.0.to_le_bytes());
            buf.push(self.uniform_block.metadata);
        } else {
            // Palette length (2 bytes)
            let palette_len = self.palette.len() as u16;
            buf.extend_from_slice(&palette_len.to_le_bytes());
            
            // Palette entries: each is block_id (2 bytes) + metadata (1 byte)
            for block in &self.palette {
                buf.extend_from_slice(&block.block_id.0.to_le_bytes());
                buf.push(block.metadata);
            }
            
            // RLE runs: count (2 bytes) + (run_len, palette_idx) pairs
            let runs_len = self.runs.len() as u16;
            buf.extend_from_slice(&runs_len.to_le_bytes());
            for (run_len, palette_idx) in &self.runs {
                buf.extend_from_slice(&run_len.to_le_bytes());
                buf.extend_from_slice(&palette_idx.to_le_bytes());
            }
        }
        
        buf
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 2 {
            return Err("Buffer too small for header".to_string());
        }
        
        let mut reader = &buf[..];
        let empty = reader[0] != 0;
        let uniform = reader[1] != 0;
        reader = &reader[2..];
        
        if empty && uniform {
            return Ok(Self::empty());
        }
        
        if uniform {
            if reader.len() < 3 {
                return Err("Buffer too small for uniform block".to_string());
            }
            let block_id = BlockId(u16::from_le_bytes([reader[0], reader[1]]));
            let metadata = reader[2];
            return Ok(Self::uniform(BlockState::new(block_id, metadata)));
        }
        
        if reader.len() < 2 {
            return Err("Buffer too small for palette length".to_string());
        }
        let palette_len = u16::from_le_bytes([reader[0], reader[1]]) as usize;
        reader = &reader[2..];
        
        let mut palette = Vec::with_capacity(palette_len);
        for _ in 0..palette_len {
            if reader.len() < 3 {
                return Err("Buffer too small for palette entry".to_string());
            }
            let block_id = BlockId(u16::from_le_bytes([reader[0], reader[1]]));
            let metadata = reader[2];
            palette.push(BlockState::new(block_id, metadata));
            reader = &reader[3..];
        }
        
        if reader.len() < 2 {
            return Err("Buffer too small for runs length".to_string());
        }
        let runs_len = u16::from_le_bytes([reader[0], reader[1]]) as usize;
        reader = &reader[2..];
        
        let mut runs = Vec::with_capacity(runs_len);
        for _ in 0..runs_len {
            if reader.len() < 4 {
                return Err("Buffer too small for run entry".to_string());
            }
            let run_len = u16::from_le_bytes([reader[0], reader[1]]);
            let palette_idx = u16::from_le_bytes([reader[2], reader[3]]);
            runs.push((run_len, palette_idx));
            reader = &reader[4..];
        }
        
        Ok(Self {
            palette,
            runs,
            empty: false,
            uniform: false,
            uniform_block: BlockState::air(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_chunk() {
        let chunk = PaletteChunk::empty();
        assert!(chunk.empty);
        assert!(chunk.uniform);
        assert_eq!(chunk.get(0), BlockState::air());
        assert_eq!(chunk.get(CHUNK_VOLUME - 1), BlockState::air());
    }

    #[test]
    fn test_uniform_chunk() {
        let chunk = PaletteChunk::uniform(BlockState::solid(BlockId::STONE));
        assert!(!chunk.empty);
        assert!(chunk.uniform);
        assert_eq!(chunk.get(0).block_id, BlockId::STONE);
        assert_eq!(chunk.get(CHUNK_VOLUME - 1).block_id, BlockId::STONE);
    }

    #[test]
    fn test_from_blocks() {
        let mut blocks = vec![BlockState::air(); CHUNK_VOLUME];
        blocks[0] = BlockState::solid(BlockId::STONE);
        blocks[100] = BlockState::solid(BlockId::DIRT);
        
        let chunk = PaletteChunk::from_blocks(&blocks);
        assert!(!chunk.empty);
        assert!(!chunk.uniform);
        assert_eq!(chunk.get(0).block_id, BlockId::STONE);
        assert_eq!(chunk.get(100).block_id, BlockId::DIRT);
        assert_eq!(chunk.get(1).block_id, BlockId::AIR);
    }

    #[test]
    fn test_set_block() {
        let mut chunk = PaletteChunk::empty();
        chunk = chunk.set(0, BlockState::solid(BlockId::STONE));
        assert_eq!(chunk.get(0).block_id, BlockId::STONE);
        
        chunk = chunk.set(0, BlockState::air());
        assert_eq!(chunk.get(0).block_id, BlockId::AIR);
    }

    #[test]
    fn test_serialization() {
        let chunk = PaletteChunk::uniform(BlockState::solid(BlockId::STONE));
        let mut buf = Vec::new();
        chunk.serialize(&mut buf).unwrap();
        
        let decoded = PaletteChunk::deserialize(&buf).unwrap();
        assert_eq!(chunk, decoded);
    }

    #[test]
    fn test_block_index() {
        assert_eq!(block_index(0, 0, 0), 0);
        assert_eq!(block_index(31, 0, 0), 31);
        assert_eq!(block_index(0, 1, 0), 32);
        assert_eq!(block_index(0, 0, 1), 1024);
        assert_eq!(block_index(31, 31, 31), CHUNK_VOLUME - 1);
    }
}