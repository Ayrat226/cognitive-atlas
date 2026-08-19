//! Block type definitions

use std::fmt;
use serde::{Deserialize, Serialize};
use lithos_engine_serialization::{TypeId, SerializationError};

/// Block identifier - 16 bits allows 65536 block types
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockId(pub u16);

impl BlockId {
    pub const AIR: BlockId = BlockId(0);
    pub const STONE: BlockId = BlockId(1);
    pub const DIRT: BlockId = BlockId(2);
    pub const GRASS_BLOCK: BlockId = BlockId(3);
    pub const SAND: BlockId = BlockId(4);
    pub const WATER: BlockId = BlockId(5);
    pub const BEDROCK: BlockId = BlockId(255);

    #[inline]
    pub const fn new(id: u16) -> Self {
        BlockId(id)
    }

    #[inline]
    pub const fn raw(self) -> u16 {
        self.0
    }

    #[inline]
    pub fn is_air(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn is_solid(self) -> bool {
        self.0 != 0 && self.0 != 5 // Not air, not water
    }
}

impl fmt::Debug for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockId({})", self.0)
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for BlockId {
    fn from(id: u16) -> Self {
        BlockId(id)
    }
}

impl From<BlockId> for u16 {
    fn from(id: BlockId) -> u16 {
        id.0
    }
}

/// Block state - additional per-block data (orientation, water level, etc.)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockState {
    pub block_id: BlockId,
    pub metadata: u8, // 8 bits for block-specific state
}

impl BlockState {
    #[inline]
    pub const fn new(block_id: BlockId, metadata: u8) -> Self {
        Self { block_id, metadata }
    }

    #[inline]
    pub const fn air() -> Self {
        Self::new(BlockId::AIR, 0)
    }

    #[inline]
    pub const fn solid(block_id: BlockId) -> Self {
        Self::new(block_id, 0)
    }

    #[inline]
    pub fn is_air(&self) -> bool {
        self.block_id.is_air()
    }
}

impl fmt::Debug for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockState({}, meta={})", self.block_id.0, self.metadata)
    }
}

/// Block registry entry (definition, not runtime state)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDef {
    pub id: BlockId,
    pub name: String,
    pub material_id: u16, // Links to material system
    pub solid: bool,
    pub transparent: bool,
    pub liquid: bool,
    pub hardness: f32,      // Mining time multiplier
    pub blast_resistance: f32,
    pub light_emission: u8, // 0-15
    pub light_opacity: u8,  // 0-15
    pub render_type: RenderType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderType {
    Cube,
    Cross,
    Plant,
    Liquid,
    Custom(u8),
}

impl Default for BlockDef {
    fn default() -> Self {
        Self {
            id: BlockId::AIR,
            name: "air".to_string(),
            material_id: 0,
            solid: false,
            transparent: true,
            liquid: false,
            hardness: 0.0,
            blast_resistance: 0.0,
            light_emission: 0,
            light_opacity: 0,
            render_type: RenderType::Cube,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_id() {
        let air = BlockId::AIR;
        assert!(air.is_air());
        assert!(!air.is_solid());

        let stone = BlockId::STONE;
        assert!(!stone.is_air());
        assert!(stone.is_solid());
    }

    #[test]
    fn test_block_state() {
        let state = BlockState::solid(BlockId::STONE);
        assert_eq!(state.block_id, BlockId::STONE);
        assert_eq!(state.metadata, 0);
    }
}