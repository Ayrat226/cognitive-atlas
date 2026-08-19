//! Coordinate system for voxel world
//! World → Region(512³) → Chunk(32³) → Block

use std::fmt;
use std::hash::{Hash, Hasher};
use lithos_engine_math::FixedVec3;
use serde::{Deserialize, Serialize};

/// Chunk size in blocks (32³ = 32768 blocks)
pub const CHUNK_SIZE: i32 = 32;
/// Region size in chunks (512³ voxels = 16×16×16 chunks)
pub const REGION_SIZE: i32 = 16;
/// Region size in blocks
pub const REGION_BLOCK_SIZE: i32 = REGION_SIZE * CHUNK_SIZE; // 512

/// 3D block position in world space (fixed-point for determinism)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl BlockPos {
    #[inline]
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    #[inline]
    pub fn from_fixed_vec3(v: FixedVec3) -> Self {
        Self::new(v.x.to_bits(), v.y.to_bits(), v.z.to_bits())
    }

    #[inline]
    pub fn to_fixed_vec3(self) -> FixedVec3 {
        FixedVec3::new(
            lithos_engine_math::Fixed::from_bits(self.x),
            lithos_engine_math::Fixed::from_bits(self.y),
            lithos_engine_math::Fixed::from_bits(self.z),
        )
    }

    /// Convert to chunk-local coordinates (0..31)
    #[inline]
    pub fn chunk_local(&self) -> (u32, u32, u32) {
        let x = ((self.x as i32) & (CHUNK_SIZE - 1)) as u32;
        let y = ((self.y as i32) & (CHUNK_SIZE - 1)) as u32;
        let z = ((self.z as i32) & (CHUNK_SIZE - 1)) as u32;
        (x, y, z)
    }

    /// Get chunk key containing this block
    #[inline]
    pub fn chunk_key(&self) -> ChunkKey {
        ChunkKey::new(
            self.x.div_euclid(CHUNK_SIZE as i64) as i32,
            self.y.div_euclid(CHUNK_SIZE as i64) as i32,
            self.z.div_euclid(CHUNK_SIZE as i64) as i32,
        )
    }

    /// Get region key containing this block
    #[inline]
    pub fn region_key(&self) -> RegionKey {
        self.chunk_key().region_key()
    }

    #[inline]
    pub fn offset(&self, dx: i64, dy: i64, dz: i64) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }
}

impl fmt::Debug for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockPos({}, {}, {})", self.x, self.y, self.z)
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/// Chunk coordinate (which chunk in the world)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkKey {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    #[inline]
    pub fn region_key(&self) -> RegionKey {
        RegionKey::new(
            self.x.div_euclid(REGION_SIZE),
            self.y.div_euclid(REGION_SIZE),
            self.z.div_euclid(REGION_SIZE),
        )
    }

    #[inline]
    pub fn chunk_local(&self) -> (u32, u32, u32) {
        (
            (self.x.rem_euclid(REGION_SIZE)) as u32,
            (self.y.rem_euclid(REGION_SIZE)) as u32,
            (self.z.rem_euclid(REGION_SIZE)) as u32,
        )
    }

    #[inline]
    pub fn min_block_pos(&self) -> BlockPos {
        BlockPos::new(
            (self.x as i64) * CHUNK_SIZE as i64,
            (self.y as i64) * CHUNK_SIZE as i64,
            (self.z as i64) * CHUNK_SIZE as i64,
        )
    }

    #[inline]
    pub fn max_block_pos(&self) -> BlockPos {
        BlockPos::new(
            ((self.x + 1) as i64) * CHUNK_SIZE as i64 - 1,
            ((self.y + 1) as i64) * CHUNK_SIZE as i64 - 1,
            ((self.z + 1) as i64) * CHUNK_SIZE as i64 - 1,
        )
    }

    #[inline]
    pub fn center(&self) -> BlockPos {
        BlockPos::new(
            (self.x as i64) * CHUNK_SIZE as i64 + CHUNK_SIZE as i64 / 2,
            (self.y as i64) * CHUNK_SIZE as i64 + CHUNK_SIZE as i64 / 2,
            (self.z as i64) * CHUNK_SIZE as i64 + CHUNK_SIZE as i64 / 2,
        )
    }
}

impl fmt::Debug for ChunkKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChunkKey({}, {}, {})", self.x, self.y, self.z)
    }
}

/// Region coordinate (which region in the world, 512³ blocks each)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl RegionKey {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    #[inline]
    pub fn min_block_pos(&self) -> BlockPos {
        BlockPos::new(
            (self.x as i64) * REGION_BLOCK_SIZE as i64,
            (self.y as i64) * REGION_BLOCK_SIZE as i64,
            (self.z as i64) * REGION_BLOCK_SIZE as i64,
        )
    }

    #[inline]
    pub fn max_block_pos(&self) -> BlockPos {
        BlockPos::new(
            ((self.x + 1) as i64) * REGION_BLOCK_SIZE as i64 - 1,
            ((self.y + 1) as i64) * REGION_BLOCK_SIZE as i64 - 1,
            ((self.z + 1) as i64) * REGION_BLOCK_SIZE as i64 - 1,
        )
    }

    #[inline]
    pub fn chunk_keys(&self) -> impl Iterator<Item = ChunkKey> {
        let base_x = self.x * REGION_SIZE;
        let base_y = self.y * REGION_SIZE;
        let base_z = self.z * REGION_SIZE;
        (0..REGION_SIZE).flat_map(move |z| {
            (0..REGION_SIZE).flat_map(move |y| {
                (0..REGION_SIZE).map(move |x| ChunkKey::new(base_x + x, base_y + y, base_z + z))
            })
        })
    }

    #[inline]
    pub fn contains_chunk(&self, chunk: ChunkKey) -> bool {
        chunk.region_key() == *self
    }
}

impl fmt::Debug for RegionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegionKey({}, {}, {})", self.x, self.y, self.z)
    }
}

/// Morton order (Z-order curve) for spatial hashing
pub fn morton3d(x: u32, y: u32, z: u32) -> u64 {
    let mut answer = 0u64;
    for i in 0..21 {
        answer |= (((x >> i) & 1) as u64) << (3 * i);
        answer |= (((y >> i) & 1) as u64) << (3 * i + 1);
        answer |= (((z >> i) & 1) as u64) << (3 * i + 2);
    }
    answer
}

/// Hash a RegionKey for hashmap
pub fn hash_region_key(key: RegionKey) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

/// Hash a ChunkKey for hashmap
pub fn hash_chunk_key(key: ChunkKey) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_pos_chunk_key() {
        let pos = BlockPos::new(100, 200, 300);
        let chunk = pos.chunk_key();
        assert_eq!(chunk.x, 3); // 100 / 32 = 3
        assert_eq!(chunk.y, 6); // 200 / 32 = 6
        assert_eq!(chunk.z, 9); // 300 / 32 = 9
    }

    #[test]
    fn test_negative_coords() {
        let pos = BlockPos::new(-100, -200, -300);
        let chunk = pos.chunk_key();
        assert_eq!(chunk.x, -4); // -100 / 32 = -4 (floor division)
        assert_eq!(chunk.y, -7); // -200 / 32 = -7
        assert_eq!(chunk.z, -10); // -300 / 32 = -10
    }

    #[test]
    fn test_chunk_local() {
        let pos = BlockPos::new(100, 200, 300);
        let (x, y, z) = pos.chunk_local();
        assert_eq!(x, 4); // 100 % 32 = 4
        assert_eq!(y, 8); // 200 % 32 = 8
        assert_eq!(z, 12); // 300 % 32 = 12
    }

    #[test]
    fn test_region_key() {
        let chunk = ChunkKey::new(100, 200, 300);
        let region = chunk.region_key();
        assert_eq!(region.x, 6); // 100 / 16 = 6
        assert_eq!(region.y, 12); // 200 / 16 = 12
        assert_eq!(region.z, 18); // 300 / 16 = 18
    }

    #[test]
    fn test_morton3d() {
        assert_eq!(morton3d(0, 0, 0), 0);
        assert_eq!(morton3d(1, 0, 0), 1);
        assert_eq!(morton3d(0, 1, 0), 2);
        assert_eq!(morton3d(0, 0, 1), 4);
    }
}