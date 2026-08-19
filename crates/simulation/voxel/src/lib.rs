<<<<<<< ours
//! Placeholder module

/// Unit tests
#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        assert!(true);
    }
}
=======
//! Voxel storage system for LITHOS
//! Sparse hash map with procedural base, delta log, and LOD aggregation

pub mod coords;
pub mod block;
pub mod chunk;
pub mod region;
pub mod storage;
pub mod procedural;
pub mod delta_log;
pub mod lod;

pub use coords::{RegionKey, ChunkKey, BlockPos, CHUNK_SIZE, REGION_SIZE};
pub use block::BlockId;
pub use chunk::{Chunk, ChunkStorage, PaletteChunk};
pub use region::Region;
pub use storage::VoxelStorage;
pub use procedural::ProceduralGenerator;
pub use delta_log::{DeltaLog, DeltaEntry};
pub use lod::{LodAggregate, LodManager};
>>>>>>> theirs
