//! Block type definitions

use serde::{Deserialize, Serialize};
use lithos_engine_math::Fixed;

/// Block identifier - 16 bits allows 65536 block types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
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

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockId({})", self.0)
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
    pub material_state: MaterialState,
    pub metadata: u8,
}

/// Runtime material state for a block
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct MaterialState {
    pub temperature: Fixed,
    pub phase: MaterialPhase,
    pub composition: Composition,
}

/// Material composition fractions (sums to 1.0 for alloys)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Composition(pub Fixed);

impl Composition {
    pub const fn one() -> Self {
        Composition(Fixed::from_f32(1.0))
    }

    pub const fn zero() -> Self {
        Composition(Fixed::ZERO)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

/// Material phase states
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialPhase {
    Solid,
    Liquid,
    Gas,
    Plasma,
}

impl MaterialState {
    /// Create a material state at standard temperature (20°C / 293.15K)
    pub fn standard(phase: MaterialPhase) -> Self {
        Self {
            temperature: Fixed::from_f32(293.15),
            phase,
            composition: Composition::one(),
        }
    }

    /// Create a material state at a specific temperature
    pub fn at_temperature(temperature: Fixed, phase: MaterialPhase) -> Self {
        Self {
            temperature,
            phase,
            composition: Composition::one(),
        }
    }
}

impl BlockState {
    #[inline]
    pub const fn new(block_id: BlockId, metadata: u8) -> Self {
        Self {
            block_id,
            material_state: MaterialState {
                temperature: Fixed::from_f32(293.15),
                phase: MaterialPhase::Solid,
                composition: Composition(Fixed::from_f32(1.0)),
            },
            metadata,
        }
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

    /// Create a block state with custom material state
    pub fn with_material(block_id: BlockId, material_state: MaterialState, metadata: u8) -> Self {
        Self {
            block_id,
            material_state,
            metadata,
        }
    }
}

impl std::fmt::Debug for BlockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BlockState({}, meta={}, temp={:.1}K, phase={:?})",
            self.block_id.0,
            self.metadata,
            self.material_state.temperature.to_f32(),
            self.material_state.phase
        )
    }
}

/// Block registry entry (definition, not runtime state)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDef {
    pub id: BlockId,
    pub name: String,
    pub material_id: u16,
    pub solid: bool,
    pub transparent: bool,
    pub liquid: bool,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub light_emission: u8,
    pub light_opacity: u8,
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
        assert_eq!(state.material_state.phase, MaterialPhase::Solid);
    }

    #[test]
    fn test_material_state() {
        let hot_stone = BlockState::with_material(
            BlockId::STONE,
            MaterialState::at_temperature(Fixed::from_f32(1500.0), MaterialPhase::Solid),
            0,
        );
        assert_eq!(hot_stone.material_state.temperature.to_f32(), 1500.0);
        assert_eq!(hot_stone.material_state.phase, MaterialPhase::Solid);
    }

    #[test]
    fn test_composition() {
        let comp = Composition::one();
        assert!(!comp.is_zero());
        let zero = Composition::zero();
        assert!(zero.is_zero());
    }
}