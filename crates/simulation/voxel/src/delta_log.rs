//! Delta log for persistent voxel modifications
//! Append-only log of (pos, old_block, new_block, tick, author)

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use crate::coords::BlockPos;
use crate::block::{BlockId, BlockState};
use serde::{Serialize, Deserialize};
use lithos_engine_serialization::{SerializationError, Writer, Reader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeltaLogError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Corrupted log at entry {0}")]
    Corrupted(usize),
}

/// Single delta entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeltaEntry {
    pub pos: BlockPos,
    pub old_block: BlockState,
    pub new_block: BlockState,
    pub tick: u64,
    pub author: u64, // Player/entity ID
}

impl DeltaEntry {
    pub fn new(pos: BlockPos, old_block: BlockState, new_block: BlockState, tick: u64, author: u64) -> Self {
        Self { pos, old_block, new_block, tick, author }
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(46);
        // Position (3 * i64 = 24 bytes)
        buf.extend_from_slice(&self.pos.x.to_le_bytes());
        buf.extend_from_slice(&self.pos.y.to_le_bytes());
        buf.extend_from_slice(&self.pos.z.to_le_bytes());
        
        // Blocks (3 bytes each = 6 bytes)
        buf.extend_from_slice(&self.old_block.block_id.0.to_le_bytes());
        buf.push(self.old_block.metadata);
        buf.extend_from_slice(&self.new_block.block_id.0.to_le_bytes());
        buf.push(self.new_block.metadata);
        
        // Tick (8 bytes) + author (8 bytes)
        buf.extend_from_slice(&self.tick.to_le_bytes());
        buf.extend_from_slice(&self.author.to_le_bytes());
        
        buf
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 46 {
            return Err("Buffer too small for DeltaEntry".to_string());
        }
        
        let mut reader = &buf[..];
        let pos = BlockPos::new(
            i64::from_le_bytes([reader[0], reader[1], reader[2], reader[3], reader[4], reader[5], reader[6], reader[7]]),
            i64::from_le_bytes([reader[8], reader[9], reader[10], reader[11], reader[12], reader[13], reader[14], reader[15]]),
            i64::from_le_bytes([reader[16], reader[17], reader[18], reader[19], reader[20], reader[21], reader[22], reader[23]]),
        );
        reader = &reader[24..];
        
        let old_block = BlockState::new(
            BlockId(u16::from_le_bytes([reader[0], reader[1]])),
            reader[2],
        );
        reader = &reader[3..];
        
        let new_block = BlockState::new(
            BlockId(u16::from_le_bytes([reader[0], reader[1]])),
            reader[2],
        );
        reader = &reader[3..];
        
        let tick = u64::from_le_bytes([
            reader[0], reader[1], reader[2], reader[3],
            reader[4], reader[5], reader[6], reader[7],
        ]);
        reader = &reader[8..];
        
        let author = u64::from_le_bytes([
            reader[0], reader[1], reader[2], reader[3],
            reader[4], reader[5], reader[6], reader[7],
        ]);
        
        Ok(Self { pos, old_block, new_block, tick, author })
    }
}

/// Delta log - append-only, with optional compaction
pub struct DeltaLog {
    entries: VecDeque<DeltaEntry>,
    max_entries: usize,
    path: Option<std::path::PathBuf>,
    entry_size: usize,
}

impl Default for DeltaLog {
    fn default() -> Self {
        Self::new(100000)
    }
}

impl DeltaLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries.min(1000)),
            max_entries,
            path: None,
            entry_size: 46,
        }
    }
    
    pub fn new_default() -> Self {
        Self::new(100000)
    }
    
    pub fn with_path(path: std::path::PathBuf, max_entries: usize) -> Self {
        let mut log = Self::new(max_entries);
        log.path = Some(path);
        log
    }
    
    /// Push new entry
    pub fn push(&mut self, entry: DeltaEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }
    
    /// Get entry at index
    pub fn get(&self, index: usize) -> Option<&DeltaEntry> {
        self.entries.get(index)
    }
    
    /// Get all entries
    pub fn entries(&self) -> impl Iterator<Item = &DeltaEntry> {
        self.entries.iter()
    }
    
    /// Get entries since tick
    pub fn since_tick(&self, tick: u64) -> impl Iterator<Item = &DeltaEntry> {
        self.entries.iter().filter(move |e| e.tick > tick)
    }
    
    /// Get entries for position
    pub fn for_position(&self, pos: crate::coords::BlockPos) -> impl Iterator<Item = &DeltaEntry> {
        self.entries.iter().filter(move |e| e.pos == pos)
    }
    
    /// Length
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Header: version (1), entry count
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        
        for entry in &self.entries {
            buf.extend_from_slice(&entry.to_bytes());
        }
        
        buf
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 8 {
            return Err("Buffer too small for header".to_string());
        }
        
        let mut reader = &buf[..];
        let version = u32::from_le_bytes([reader[0], reader[1], reader[2], reader[3]]);
        let count = u32::from_le_bytes([reader[4], reader[5], reader[6], reader[7]]);
        reader = &reader[8..];
        
        if version != 1 {
            return Err("Invalid delta log version".to_string());
        }
        
        let mut entries = VecDeque::with_capacity(count as usize);
        let entry_size = 46;
        
        for i in 0..count {
            if reader.len() < 46 {
                return Err("Buffer too small for entry".to_string());
            }
            let entry = DeltaEntry::from_bytes(&reader[..46])
                .map_err(|e| format!("Corrupted entry {}: {}", i, e))?;
            entries.push_back(entry);
            reader = &reader[46..];
        }
        
        Ok(Self {
            entries,
            max_entries: count as usize * 2,
            path: None,
            entry_size: 46,
        })
    }
    
    /// Save to file
    pub fn save(&mut self, path: &Path) -> Result<(), DeltaLogError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        let bytes = self.to_bytes();
        writer.write_all(&bytes)?;
        writer.flush()?;
        Ok(())
    }
    
    /// Load from file
    pub fn load(path: &Path) -> Result<Self, DeltaLogError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        
        let log = Self::from_bytes(&buf).map_err(|e| DeltaLogError::Corrupted(0))?;
        Ok(log)
    }
    
    /// Compact log by removing redundant entries
    /// Keeps only the latest change per position
    pub fn compact(&mut self) -> usize {
        use std::collections::HashMap;
        
        let mut latest = HashMap::new();
        for entry in self.entries.drain(..) {
            latest.insert(entry.pos, entry);
        }
        
        let before = self.max_entries;
        self.entries = latest.into_values().collect();
        before - self.entries.len()
    }
    
    /// Replay entries since tick onto storage
    pub fn replay_since<F>(&self, tick: u64, mut apply: F) 
    where
        F: FnMut(crate::coords::BlockPos, crate::block::BlockState),
    {
        for entry in self.entries.iter().filter(|e| e.tick > tick) {
            apply(entry.pos, entry.new_block);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::BlockPos;
    use crate::block::{BlockId, BlockState};
    
    #[test]
    fn test_delta_entry() {
        let entry = DeltaEntry::new(
            BlockPos::new(10, 20, 30),
            BlockState::air(),
            BlockState::solid(BlockId::STONE),
            42,
            1,
        );
        
        let buf = entry.to_bytes();
        let decoded = DeltaEntry::from_bytes(&buf).unwrap();
        assert_eq!(entry.pos, decoded.pos);
        assert_eq!(entry.old_block, decoded.old_block);
        assert_eq!(entry.new_block, decoded.new_block);
        assert_eq!(entry.tick, decoded.tick);
        assert_eq!(entry.author, decoded.author);
    }
    
    #[test]
    fn test_delta_log() {
        let mut log = DeltaLog::new(100);
        assert!(log.is_empty());
        
        log.push(DeltaEntry::new(
            BlockPos::new(1, 2, 3),
            BlockState::air(),
            BlockState::solid(BlockId::STONE),
            1,
            1,
        ));
        
        assert_eq!(log.len(), 1);
        
        let entry = log.get(0).unwrap();
        assert_eq!(entry.pos, BlockPos::new(1, 2, 3));
        assert_eq!(entry.new_block.block_id, BlockId::STONE);
    }
    
    #[test]
    fn test_delta_log_compact() {
        let mut log = DeltaLog::new(100);
        
        // Multiple changes to same position
        for i in 0..10 {
            log.push(DeltaEntry::new(
                BlockPos::new(0, 0, 0),
                BlockState::air(),
                BlockState::solid(BlockId::new(i as u16)),
                i,
                1,
            ));
        }
        
        assert_eq!(log.len(), 10);
        let removed = log.compact();
        assert_eq!(removed, 9);
        assert_eq!(log.len(), 1);
        
        // Should keep the last one
        let entry = log.get(0).unwrap();
        assert_eq!(entry.new_block.block_id.raw(), 9);
    }
    
    #[test]
    fn test_delta_log_save_load() {
        let mut log = DeltaLog::new(100);
        log.push(DeltaEntry::new(
            BlockPos::new(10, 20, 30),
            BlockState::air(),
            BlockState::solid(BlockId::STONE),
            42,
            1,
        ));
        
        let temp_path = std::env::temp_dir().join("test_deltalog.bin");
        log.save(&temp_path).unwrap();
        
        let loaded = DeltaLog::load(&temp_path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.get(0).unwrap().tick, 42);
        
        std::fs::remove_file(temp_path).ok();
    }
}