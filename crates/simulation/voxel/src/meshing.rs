//! Greedy meshing for voxel chunks
//! Produces optimized face-merged meshes for rendering

use crate::coords::CHUNK_SIZE;
use crate::chunk::{PaletteChunk, block_index};
use crate::block::BlockState;
use lithos_engine_math::{Vec3, AABB};
use serde::{Serialize, Deserialize};

/// Face direction
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FaceDir {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

impl FaceDir {
    pub const ALL: [FaceDir; 6] = [
        FaceDir::PosX, FaceDir::NegX,
        FaceDir::PosY, FaceDir::NegY,
        FaceDir::PosZ, FaceDir::NegZ,
    ];
    
    #[inline]
    pub fn normal(&self) -> Vec3 {
        match self {
            FaceDir::PosX => Vec3::new(1.0, 0.0, 0.0),
            FaceDir::NegX => Vec3::new(-1.0, 0.0, 0.0),
            FaceDir::PosY => Vec3::new(0.0, 1.0, 0.0),
            FaceDir::NegY => Vec3::new(0.0, -1.0, 0.0),
            FaceDir::PosZ => Vec3::new(0.0, 0.0, 1.0),
            FaceDir::NegZ => Vec3::new(0.0, 0.0, -1.0),
        }
    }
    
    #[inline]
    pub fn offset(&self) -> (i32, i32, i32) {
        match self {
            FaceDir::PosX => (1, 0, 0),
            FaceDir::NegX => (-1, 0, 0),
            FaceDir::PosY => (0, 1, 0),
            FaceDir::NegY => (0, -1, 0),
            FaceDir::PosZ => (0, 0, 1),
            FaceDir::NegZ => (0, 0, -1),
        }
    }
}

/// Vertex structure for voxel meshes
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub material_id: u32,
    pub _padding: u32,
}

/// Mesh face with merged rectangles
#[derive(Clone, Debug)]
pub struct MeshFace {
    pub dir: FaceDir,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: u32,
    pub height: u32,
    pub material_id: u32,
}

/// Greedy mesh result
pub struct GreedyMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub aabb: AABB,
    pub face_count: usize,
}

/// Neighbor chunk access for cross-chunk face merging
pub trait ChunkProvider {
    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockState;
}

/// Check if block is solid (opaque)
#[inline]
fn is_solid(block: &BlockState) -> bool {
    !block.block_id.is_air()
}

/// Check if two blocks have same material
#[inline]
fn same_material(a: &BlockState, b: &BlockState) -> bool {
    a.block_id == b.block_id
}

/// Greedy meshing for a single chunk with neighbor access
pub fn greedy_mesh_chunk<P: ChunkProvider>(
    chunk: &PaletteChunk,
    provider: &P,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
) -> GreedyMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut face_count = 0;
    
    let chunk_base_x = chunk_x * CHUNK_SIZE;
    let chunk_base_y = chunk_y * CHUNK_SIZE;
    let chunk_base_z = chunk_z * CHUNK_SIZE;
    
    // Face directions to process
    let face_dirs = [
        (FaceDir::PosX, (1, 0, 0)),
        (FaceDir::NegX, (-1, 0, 0)),
        (FaceDir::PosY, (0, 1, 0)),
        (FaceDir::NegY, (0, -1, 0)),
        (FaceDir::PosZ, (0, 0, 1)),
        (FaceDir::NegZ, (0, 0, -1)),
    ];
    
    for (dir, offset) in face_dirs {
        // Create visited grid for this face direction
        let mut visited = vec![false; (CHUNK_SIZE * CHUNK_SIZE) as usize];
        
        // Iterate over all positions in the face plane
        for v in 0..CHUNK_SIZE {
            for u in 0..CHUNK_SIZE {
                let idx = (v * CHUNK_SIZE + u) as usize;
                if visited[idx] {
                    continue;
                }
                
                // Get block at this position on the face
                let (x, y, z) = match dir {
                    FaceDir::PosX => (CHUNK_SIZE - 1, u, v),
                    FaceDir::NegX => (0, u, v),
                    FaceDir::PosY => (u, CHUNK_SIZE - 1, v),
                    FaceDir::NegY => (u, 0, v),
                    FaceDir::PosZ => (u, v, CHUNK_SIZE - 1),
                    FaceDir::NegZ => (u, v, 0),
                };
                
                let block = chunk.get(block_index(x as u32, y as u32, z as u32));
                if !is_solid(&block) {
                    visited[idx] = true;
                    continue;
                }
                
                // Check neighbor
                let neighbor = provider.get_block(
                    chunk_x * CHUNK_SIZE + x + dir.offset().0,
                    chunk_y * CHUNK_SIZE + y + dir.offset().1,
                    chunk_z * CHUNK_SIZE + z + dir.offset().2,
                );
                
                if is_solid(&neighbor) && same_material(&chunk.get(block_index(x as u32, y as u32, z as u32)), &neighbor) {
                    visited[idx] = true;
                    continue;
                }
                
                // Found exposed face - expand greedily
                let material_id = chunk.get(block_index(x as u32, y as u32, z as u32)).block_id.0 as u32;
                let (width, height) = expand_face(
                    &chunk,
                    u as i32, v as i32,
                    material_id,
                    dir,
                    provider,
                    &visited,
                );
                
                // Mark visited
                for vv in 0..height {
                    for uu in 0..width {
                        let v_idx = (v + vv) * CHUNK_SIZE + (u + uu);
                        visited[v_idx as usize] = true;
                    }
                }
                
                // Generate quad for this face
                let face_vertices = generate_face_vertices(
                    dir,
                    x, y, z,
                    width,
                    height,
                    material_id,
                );
                
                let base_idx = vertices.len() as u32;
                vertices.extend(face_vertices);
                indices.push(base_idx);
                indices.push(base_idx + 1);
                indices.push(base_idx + 2);
                indices.push(base_idx + 2);
                indices.push(base_idx + 3);
                indices.push(base_idx);
                
                face_count += 1;
            }
        }
    }
    
    GreedyMesh {
        vertices,
        indices,
        aabb: AABB::from_center_half_extents(
            Vec3::new(chunk_x as f32 * CHUNK_SIZE as f32 + 16.0, chunk_y as f32 * CHUNK_SIZE as f32 + 16.0, chunk_z as f32 * CHUNK_SIZE as f32 + 16.0),
            Vec3::new(16.0, 16.0, 16.0),
        ),
        face_count,
    }
}

fn expand_face(
    chunk: &PaletteChunk,
    start_u: i32,
    start_v: i32,
    material_id: u32,
    dir: FaceDir,
    provider: &impl ChunkProvider,
    visited: &[bool],
) -> (i32, i32) {
    // Find max width
    let mut width = 1;
    while start_u + width < CHUNK_SIZE {
        let idx = (start_v * CHUNK_SIZE + start_u + width) as usize;
        if visited[idx] {
            break;
        }
        
        let (x, y, z) = match dir {
            FaceDir::PosX => (CHUNK_SIZE - 1, start_u + width, 0),
            FaceDir::NegX => (0, start_u + width, 0),
            FaceDir::PosY => (start_u + width, CHUNK_SIZE - 1, 0),
            FaceDir::NegY => (start_u + width, 0, 0),
            FaceDir::PosZ => (start_u + width, 0, CHUNK_SIZE - 1),
            FaceDir::NegZ => (start_u + width, 0, 0),
        };
        
        let block = chunk.get(block_index(x as u32, y as u32, z as u32));
        if !is_solid(&block) || block.block_id.0 as u32 != material_id {
            break;
        }
        
        // Check neighbor in the direction of the face
        let neighbor = match dir {
            FaceDir::PosX => provider.get_block(CHUNK_SIZE, start_u + width, 0),
            FaceDir::NegX => provider.get_block(-1, start_u + width, 0),
            FaceDir::PosY => provider.get_block(start_u + width, CHUNK_SIZE, 0),
            FaceDir::NegY => provider.get_block(start_u + width, -1, 0),
            FaceDir::PosZ => provider.get_block(start_u + width, 0, CHUNK_SIZE),
            FaceDir::NegZ => provider.get_block(start_u + width, 0, -1),
        };
        
        if is_solid(&neighbor) && same_material(&chunk.get(block_index((start_u + width) as u32, 0, 0)), &neighbor) {
            break;
        }
        
        width += 1;
    }
    
    // Find max height
    let mut height = 1;
    while start_v + height < CHUNK_SIZE {
        let mut can_expand = true;
        for w in 0..width {
            let idx = ((start_v + height) * CHUNK_SIZE + start_u + w) as usize;
            if visited[idx] {
                can_expand = false;
                break;
            }
            
            let (x, y, z) = match dir {
                FaceDir::PosX => (CHUNK_SIZE - 1, w, height),
                FaceDir::NegX => (0, w, height),
                FaceDir::PosY => (w, CHUNK_SIZE - 1, height),
                FaceDir::NegY => (w, 0, height),
                FaceDir::PosZ => (w, height, CHUNK_SIZE - 1),
                FaceDir::NegZ => (w, height, 0),
            };
            
            let block = chunk.get(block_index(x as u32, y as u32, z as u32));
            if !is_solid(&block) || block.block_id.0 as u32 != material_id {
                can_expand = false;
                break;
            }
        }
        
        if !can_expand {
            break;
        }
        height += 1;
    }
    
    (width, height)
}

fn generate_face_vertices(
    dir: FaceDir,
    x: i32,
    y: i32,
    z: i32,
    width: i32,
    height: i32,
    material_id: u32,
) -> [MeshVertex; 4] {
    let w = width as f32;
    let h = height as f32;
    let fx = x as f32;
    let fy = y as f32;
    let fz = z as f32;
    let normal = dir.normal();
    
    match dir {
        FaceDir::PosX => [
            MeshVertex { position: [fx + 1.0, fy, fz], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + 1.0, fy + w, fz], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + 1.0, fy + w, fz + h], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + 1.0, fy, fz + h], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::NegX => [
            MeshVertex { position: [fx, fy + w, fz], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy, fz], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy, fz + h], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy + w, fz + h], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::PosY => [
            MeshVertex { position: [fx, fy + 1.0, fz], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + w, fy + 1.0, fz], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + w, fy + 1.0, fz + h], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy + 1.0, fz + h], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::NegY => [
            MeshVertex { position: [fx + w, fy, fz], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy, fz], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy, fz + h], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + w, fy, fz + h], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::PosZ => [
            MeshVertex { position: [fx, fy, fz + 1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + w, fy, fz + 1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + w, fy + h, fz + 1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy + h, fz + 1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::NegZ => [
            MeshVertex { position: [fx + w, fy, fz], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy, fz], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0], material_id, _padding: 0 },
            MeshVertex { position: [fx, fy + h, fz], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0], material_id, _padding: 0 },
            MeshVertex { position: [fx + w, fy + h, fz], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0], material_id, _padding: 0 },
        ],
    }
}

/// Simple chunk provider for isolated chunk meshing (no neighbors)
pub struct IsolatedChunkProvider<'a> {
    chunk: &'a PaletteChunk,
}

impl<'a> IsolatedChunkProvider<'a> {
    pub fn new(chunk: &'a PaletteChunk) -> Self {
        Self { chunk }
    }
}

impl<'a> ChunkProvider for IsolatedChunkProvider<'a> {
    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockState {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_SIZE || z < 0 || z >= CHUNK_SIZE {
            return BlockState::air();
        }
        self.chunk.get(block_index(x as u32, y as u32, z as u32))
    }
}

/// Mesh a chunk with no neighbor info (isolated)
pub fn mesh_chunk_isolated(chunk: &PaletteChunk, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> GreedyMesh {
    let provider = IsolatedChunkProvider::new(chunk);
    greedy_mesh_chunk(chunk, &provider, chunk_x, chunk_y, chunk_z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{BlockId, BlockState};
    
    #[test]
    fn test_empty_chunk() {
        let chunk = PaletteChunk::empty();
        let mesh = mesh_chunk_isolated(&chunk, 0, 0, 0);
        assert_eq!(mesh.vertices.len(), 0);
        assert_eq!(mesh.indices.len(), 0);
    }
    
    #[test]
    fn test_uniform_chunk() {
        let chunk = PaletteChunk::uniform(BlockState::solid(BlockId::STONE));
        let mesh = mesh_chunk_isolated(&chunk, 0, 0, 0);
        // Should produce 6 faces (cube)
        assert_eq!(mesh.face_count, 6);
        assert_eq!(mesh.vertices.len(), 24); // 6 faces * 4 vertices
        assert_eq!(mesh.indices.len(), 36); // 6 faces * 6 indices
    }
    
    #[test]
    fn test_single_block() {
        let mut chunk = PaletteChunk::empty();
        chunk = chunk.set(0, BlockState::solid(BlockId::STONE));
        let mesh = mesh_chunk_isolated(&chunk, 0, 0, 0);
        assert_eq!(mesh.face_count, 6);
    }
}