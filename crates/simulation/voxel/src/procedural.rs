//! Procedural world generation (deterministic, seeded)

use crate::coords::{ChunkKey, RegionKey, CHUNK_SIZE, REGION_BLOCK_SIZE};
use crate::chunk::{PaletteChunk, Chunk, CHUNK_VOLUME};
use crate::block::{BlockId, BlockState};
use crate::coords::morton3d;
use lithos_engine_math::{Vec3, perlin_3d, fbm_3d, simplex_3d};
use lithos_engine_serialization::{Serialize, TypeId, SerializationError};
use serde::{Deserialize, Serialize as SerdeSerialize};

/// World generation parameters
#[derive(Clone, Debug, SerdeSerialize, serde::Deserialize)]
pub struct GenerationParams {
    pub seed: u64,
    pub generator_version: u32,
    pub noise_octaves: u32,
    pub noise_persistence: f32,
    pub noise_lacunarity: f32,
    pub terrain_height_scale: f32,
    pub terrain_base_height: f32,
    pub cave_frequency: f32,
    pub ore_frequency: f32,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            seed: 0,
            generator_version: 1,
            noise_octaves: 6,
            noise_persistence: 0.5,
            noise_lacunarity: 2.0,
            terrain_height_scale: 64.0,
            terrain_base_height: 64.0,
            cave_frequency: 0.02,
            ore_frequency: 0.005,
        }
    }
}

impl Serialize for GenerationParams {
    fn type_id() -> lithos_engine_serialization::TypeId {
        lithos_engine_serialization::TypeId::new("GenerationParams")
    }
    
    fn schema_version() -> u32 {
        1
    }
    
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), lithos_engine_serialization::SerializationError> {
        buf.extend_from_slice(&self.seed.to_le_bytes());
        buf.extend_from_slice(&self.generator_version.to_le_bytes());
        buf.extend_from_slice(&self.noise_octaves.to_le_bytes());
        buf.extend_from_slice(&self.noise_persistence.to_le_bytes());
        buf.extend_from_slice(&self.noise_lacunarity.to_le_bytes());
        buf.extend_from_slice(&self.terrain_height_scale.to_le_bytes());
        buf.extend_from_slice(&self.terrain_base_height.to_le_bytes());
        buf.extend_from_slice(&self.cave_frequency.to_le_bytes());
        buf.extend_from_slice(&self.ore_frequency.to_le_bytes());
        Ok(())
    }

    fn deserialize(buf: &[u8]) -> Result<Self, lithos_engine_serialization::SerializationError> {
        if buf.len() < 8 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4 {
            return Err(lithos_engine_serialization::SerializationError::BufferTooSmall { need: 40, have: buf.len() });
        }
        let mut reader = lithos_engine_serialization::Reader::new(buf);
        Ok(Self {
            seed: reader.read::<u64>()?,
            generator_version: reader.read::<u32>()?,
            noise_octaves: reader.read::<u32>()?,
            noise_persistence: reader.read::<f32>()?,
            noise_lacunarity: reader.read::<f32>()?,
            terrain_height_scale: reader.read::<f32>()?,
            terrain_base_height: reader.read::<f32>()?,
            cave_frequency: reader.read::<f32>()?,
            ore_frequency: reader.read::<f32>()?,
        })
    }
}

/// Procedural generator for voxel world
#[derive(Clone, Debug)]
pub struct ProceduralGenerator {
    params: GenerationParams,
}

impl Default for ProceduralGenerator {
    fn default() -> Self {
        Self {
            params: GenerationParams::default(),
        }
    }
}

impl ProceduralGenerator {
    pub fn new(params: GenerationParams) -> Self {
        Self { params }
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut params = GenerationParams::default();
        params.seed = seed;
        Self { params }
    }

    /// Generate chunk at given key
    pub fn generate_chunk(&self, chunk_key: ChunkKey) -> Chunk {
        let base_pos = chunk_key.min_block_pos();
        let mut blocks = Vec::with_capacity(CHUNK_VOLUME);

        // World position of chunk base
        let world_x = base_pos.x as f32;
        let world_y = base_pos.y as f32;
        let world_z = base_pos.z as f32;

        for i in 0..CHUNK_VOLUME {
            let (x, y, z) = crate::chunk::block_coords(i);
            let bx = world_x + x as f32;
            let by = world_y + y as f32;
            let bz = world_z + z as f32;

            let block = self.generate_block(bx, by, bz);
            blocks.push(block);
        }

        let data = PaletteChunk::from_blocks(&blocks);
        Chunk::new(data, 0)
    }

    /// Generate single block at world coordinates
    pub fn generate_block(&self, x: f32, y: f32, z: f32) -> BlockState {
        // Heightmap
        let surface_height = self.surface_height(x, z);
        
        if y > surface_height + 2.0 {
            return BlockState::air();
        }
        if y > surface_height {
            return BlockState::solid(BlockId::GRASS_BLOCK);
        }
        if y > surface_height - 3.0 {
            return BlockState::solid(BlockId::DIRT);
        }
        if y > surface_height - 10.0 {
            return BlockState::solid(BlockId::STONE);
        }
        if y < -50.0 {
            return BlockState::solid(BlockId::BEDROCK);
        }

        // Caves
        if self.is_cave(x, y, z) {
            return BlockState::air();
        }

        // Ores
        if self.is_ore(x, y, z) {
            return BlockState::solid(BlockId::new(100 + (self.ore_type(x, y, z) as u16)));
        }

        BlockState::solid(BlockId::STONE)
    }

    /// Surface height at (x, z)
    fn surface_height(&self, x: f32, z: f32) -> f32 {
        let scale = 0.01;
        let nx = x * scale;
        let nz = z * scale;
        
        let base = fbm_3d(
            Vec3::new(nx, 0.0, nz),
            self.params.noise_octaves,
            self.params.noise_persistence,
            self.params.noise_lacunarity,
            self.params.seed,
        );
        
        let hills = fbm_3d(
            Vec3::new(nx * 2.0, 0.0, nz * 2.0),
            4,
            0.5,
            2.0,
            self.params.seed ^ 0x1234,
        );
        
        self.params.terrain_base_height + base * self.params.terrain_height_scale + hills * 20.0
    }

    /// Cave generation using 3D noise
    fn is_cave(&self, x: f32, y: f32, z: f32) -> bool {
        if y > 60.0 || y < 5.0 {
            return false;
        }
        
        let scale = self.params.cave_frequency;
        let noise = fbm_3d(
            Vec3::new(x * scale, y * scale, z * scale),
            4,
            0.5,
            2.0,
            self.params.seed ^ 0x5678,
        );
        
        noise > 0.7
    }

    /// Ore generation
    fn is_ore(&self, x: f32, y: f32, z: f32) -> bool {
        if y > 50.0 || y < 5.0 {
            return false;
        }
        
        let scale = self.params.ore_frequency;
        let noise = simplex_3d(
            Vec3::new(x * scale, y * scale, z * scale),
            self.params.seed ^ 0x9ABC,
        );
        
        noise > 0.85
    }

    fn ore_type(&self, x: f32, y: f32, z: f32) -> u8 {
        let hash = morton3d(x as u32, y as u32, z as u32) ^ self.params.seed;
        (hash % 8) as u8
    }

    /// Generate region (for bulk generation)
    pub fn generate_region(&self, region_key: RegionKey) -> Vec<(ChunkKey, Chunk)> {
        region_key.chunk_keys()
            .map(|ck| (ck, self.generate_chunk(ck)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_deterministic() {
        let gen = ProceduralGenerator::with_seed(42);
        let chunk1 = gen.generate_chunk(ChunkKey::new(0, 0, 0));
        let chunk2 = gen.generate_chunk(ChunkKey::new(0, 0, 0));
        assert_eq!(chunk1.data, chunk2.data);
    }

    #[test]
    fn test_different_seeds() {
        let gen1 = ProceduralGenerator::with_seed(1);
        let gen2 = ProceduralGenerator::with_seed(2);
        let chunk1 = gen1.generate_chunk(ChunkKey::new(0, 0, 0));
        let chunk2 = gen2.generate_chunk(ChunkKey::new(0, 0, 0));
        assert_ne!(chunk1.data, chunk2.data);
    }

    #[test]
    fn test_surface_height() {
        let gen = ProceduralGenerator::with_seed(123);
        let h1 = gen.surface_height(0.0, 0.0);
        let h2 = gen.surface_height(100.0, 0.0);
        // Heights should vary
        assert_ne!(h1, h2);
    }
}