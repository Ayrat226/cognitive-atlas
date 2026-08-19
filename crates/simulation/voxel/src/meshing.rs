//! Greedy meshing for voxel chunks
//! Produces optimized face-merged meshes for rendering

use crate::coords::CHUNK_SIZE;
use crate::chunk::PaletteChunk;
use crate::block::BlockState;
use lithos_engine_math::{Vec3, AABB};
use std::collections::HashMap;

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
    
    #[inline]
    pub fn uvs(&self, u: f32, v: f32) -> [f32; 2] {
        match self {
            FaceDir::PosX | FaceDir::NegX => [u, v],
            FaceDir::PosY | FaceDir::NegY => [u, v],
            FaceDir::PosZ | FaceDir::NegZ => [u, v],
        }
    }
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
    pub vertices: Vec<crate::rendering::mesh::MeshVertex>,
    pub indices: Vec<u32>,
    pub aabb: lithos_engine_math::AABB,
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
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    let mut face_count = 0;
    
    // Face directions to process
    let face_dirs = [
        (FaceDir::PosX, (1, 0, 0)),
        (FaceDir::NegX, (-1, 0, 0)),
        (FaceDir::PosY, (0, 1, 0)),
        (FaceDir::NegY, (0, -1, 0)),
        (FaceDir::PosZ, (0, 0, 1)),
        (FaceDir::NegZ, (0, 0, -1)),
    ];
    
    let chunk_size = CHUNK_SIZE as i32;
    let chunk_base_x = chunk_x * CHUNK_SIZE as i32;
    let chunk_base_y = chunk_y * CHUNK_SIZE as i32;
    let chunk_base_z = chunk_z * CHUNK_SIZE as i32;
    
    for (dir, offset) in face_dirs {
        // Create visited grid for this face direction
        let mut visited = vec![false; (CHUNK_SIZE * CHUNK_SIZE) as usize];
        
        // Determine iteration order based on face direction
        let (iter_x, iter_y, iter_z) = match dir {
            FaceDir::PosX | FaceDir::NegX => {
                // Iterate over Y, Z plane
                (0, 1, 2)
            }
            FaceDir::PosY | FaceDir::NegY => {
                // Iterate over X, Z plane
                (1, 0, 2)
            }
            FaceDir::PosZ | FaceDir::NegZ => {
                // Iterate over X, Y plane
                (1, 2, 0)
            }
        };
        
        let (mut u1, mut u2, mut u3) = match dir {
            FaceDir::PosX | FaceDir::NegX => (chunk_size - 1, chunk_size - 1, chunk_size - 1),
            FaceDir::PosY | FaceDir::NegY => (chunk_size - 1, chunk_size - 1, chunk_size - 1),
            FaceDir::PosZ | FaceDir::NegZ => (chunk_size - 1, chunk_size - 1, chunk_size - 1),
        };
        
        // We'll iterate over the two in-plane axes
        let plane_axes = match dir {
            FaceDir::PosX | FaceDir::NegX => (1, 2), // Y, Z
            FaceDir::PosY | FaceDir::NegY => (0, 2), // X, Z
            FaceDir::PosZ | FaceDir::NegZ => (0, 1), // X, Y
        };
        
        let axis_u = plane_axes.0;
        let axis_v = plane_axes.1;
        let axis_normal = match dir {
            FaceDir::PosX | FaceDir::NegX => 0,
            FaceDir::PosY | FaceDir::NegY => 1,
            FaceDir::PosZ | FaceDir::NegZ => 2,
        };
        
        // Iterate over all positions in the face plane
        for v in 0..CHUNK_SIZE {
            for u in 0..CHUNK_SIZE {
                let idx = (v * CHUNK_SIZE + u) as usize;
                if visited[idx] {
                    continue;
                }
                
                // Get block at this position on the face
                let (x, y, z) = match dir {
                    FaceDir::PosX => (chunk_size - 1, u, v),
                    FaceDir::NegX => (0, u, v),
                    FaceDir::PosY => (u, chunk_size - 1, v),
                    FaceDir::NegY => (u, 0, v),
                    FaceDir::PosZ => (u, v, chunk_size - 1),
                    FaceDir::NegZ => (u, v, 0),
                };
                
                let block = chunk.get(x as u32, y as u32, z as u32);
                if !is_solid(&block) {
                    visited[idx] = true;
                    continue;
                }
                
                // Check neighbor
                let neighbor = provider.get_block(
                    chunk_base_x + x + offset.0,
                    chunk_base_y + y + offset.1,
                    chunk_base_z + z + offset.2,
                );
                
                if is_solid(&neighbor) && same_material(&chunk.get(x as u32, y as u32, z as u32), &neighbor) {
                    visited[idx] = true;
                    continue;
                }
                
                // Found exposed face - expand greedily
                let material_id = chunk.get(x as u32, y as u32, z as u32).block_id.0 as u32;
                let (width, height) = expand_face(
                    &chunk,
                    u, v,
                    axis_u, axis_v,
                    &visited,
                    material_id,
                    dir,
                    provider,
                    chunk_base_x, chunk_base_y, chunk_base_z,
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
                    chunk_base_x + x,
                    chunk_base_y + y,
                    chunk_base_z + z,
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
            Vec3::new(chunk_base_x as f32 + 16.0, chunk_base_y as f32 + 16.0, chunk_base_z as f32 + 16.0),
            Vec3::new(16.0, 16.0, 16.0),
        ),
        face_count,
    }
}

fn expand_face(
    chunk: &PaletteChunk,
    start_u: u32,
    start_v: u32,
    axis_u: usize,
    axis_v: usize,
    visited: &[bool],
    material_id: u32,
    dir: FaceDir,
    provider: &impl ChunkProvider,
    chunk_base_x: i32,
    chunk_base_y: i32,
    chunk_base_z: i32,
) -> (u32, u32) {
    let chunk_size = CHUNK_SIZE as u32;
    let offset = dir.offset();
    
    // Find max width
    let mut width = 1;
    while start_u + width < chunk_size {
        let idx = (start_v * CHUNK_SIZE + start_u + width) as usize;
        if visited[idx] {
            break;
        }
        
        let (x, y, z) = match dir {
            FaceDir::PosX => (CHUNK_SIZE - 1, start_u + width, start_v),
            FaceDir::NegX => (0, start_u + width, start_v),
            FaceDir::PosY => (start_u + width, CHUNK_SIZE - 1, start_v),
            FaceDir::NegY => (start_u + width, 0, start_v),
            FaceDir::PosZ => (start_u + width, start_v, CHUNK_SIZE - 1),
            FaceDir::NegZ => (start_u + width, start_v, 0),
        };
        
        let block = chunk.get(x as u32, y as u32, z as u32);
        if !is_solid(&block) || block.block_id.0 as u32 != material_id {
            break;
        }
        
        // Check neighbor
        let (nx, ny, nz) = match dir {
            FaceDir::PosX => (CHUNK_SIZE, start_u + width, start_v),
            FaceDir::NegX => (-1, start_u + width, start_v),
            FaceDir::PosY => (start_u + width, CHUNK_SIZE, start_v),
            FaceDir::NegY => (start_u + width, -1, start_v),
            FaceDir::PosZ => (start_u + width, start_v, CHUNK_SIZE),
            FaceDir::NegZ => (start_u + width, start_v, -1),
        };
        
        let neighbor = provider.get_block(
            chunk_base_x + nx,
            chunk_base_y + ny,
            chunk_base_z + nz,
        );
        
        if is_solid(&neighbor) && same_material(&chunk.get(x as u32, y as u32, z as u32), &neighbor) {
            break;
        }
        
        width += 1;
    }
    
    // Find max height
    let mut height = 1;
    while start_v + height < chunk_size {
        let mut can_expand = true;
        for w in 0..width {
            let idx = ((start_v + height) * CHUNK_SIZE + start_u + w) as usize;
            if visited[idx] {
                can_expand = false;
                break;
            }
            
            let (x, y, z) = match dir {
                FaceDir::PosX => (CHUNK_SIZE - 1, start_u + w, start_v + height),
                FaceDir::NegX => (0, start_u + w, start_v + height),
                FaceDir::PosY => (start_u + w, CHUNK_SIZE - 1, start_v + height),
                FaceDir::NegY => (start_u + w, 0, start_v + height),
                FaceDir::PosZ => (start_u + w, start_v + height, CHUNK_SIZE - 1),
                FaceDir::NegZ => (start_u + w, start_v + height, 0),
            };
            
            let block = chunk.get(x as u32, y as u32, z as u32);
            if !is_solid(&block) || block.block_id.0 as u32 != material_id {
                can_expand = false;
                break;
            }
            
            // Check neighbor
            let (nx, ny, nz) = match dir {
                FaceDir::PosX => (CHUNK_SIZE, start_u + w, start_v + height),
                FaceDir::NegX => (-1, start_u + w, start_v + height),
                FaceDir::PosY => (start_u + w, CHUNK_SIZE, start_v + height),
                FaceDir::NegY => (start_u + w, -1, start_v + height),
                FaceDir::PosZ => (start_u + w, start_v + height, CHUNK_SIZE),
                FaceDir::NegZ => (start_u + w, start_v + height, -1),
            };
            
            let neighbor = provider.get_block(
                chunk_base_x + nx,
                chunk_base_y + ny,
                chunk_base_z + nz,
            );
            
            if is_solid(&neighbor) && same_material(&chunk.get(x as u32, y as u32, z as u32), &neighbor) {
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
    width: u32,
    height: u32,
    material_id: u32,
) -> [crate::rendering::mesh::MeshVertex; 4] {
    let w = width as f32;
    let h = height as f32;
    let fx = x as f32;
    let fy = y as f32;
    let fz = z as f32;
    let normal = dir.normal();
    
    match dir {
        FaceDir::PosX => [
            crate::rendering::mesh::MeshVertex { position: [fx + 1.0, fy, fz], normal: normal.to_array(), uv: [0.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + 1.0, fy + w, fz], normal: normal.to_array(), uv: [1.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + 1.0, fy + w, fz + h], normal: normal.to_array(), uv: [1.0, 1.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + 1.0, fy, fz + h], normal: normal.to_array(), uv: [0.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::NegX => [
            crate::rendering::mesh::MeshVertex { position: [fx, fy + w, fz], normal: normal.to_array(), uv: [1.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy, fz], normal: normal.to_array(), uv: [0.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy, fz + h], normal: normal.to_array(), uv: [0.0, 1.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy + w, fz + h], normal: normal.to_array(), uv: [1.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::PosY => [
            crate::rendering::mesh::MeshVertex { position: [fx, fy + 1.0, fz], normal: normal.to_array(), uv: [0.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy + 1.0, fz], normal: normal.to_array(), uv: [1.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy + 1.0, fz + h], normal: normal.to_array(), uv: [1.0, 1.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy + 1.0, fz + h], normal: normal.to_array(), uv: [0.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::NegY => [
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy, fz], normal: normal.to_array(), uv: [1.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy, fz], normal: normal.to_array(), uv: [0.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy, fz + h], normal: normal.to_array(), uv: [0.0, 1.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy, fz + h], normal: normal.to_array(), uv: [1.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::PosZ => [
            crate::rendering::mesh::MeshVertex { position: [fx, fy, fz + 1.0], normal: normal.to_array(), uv: [0.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy, fz + 1.0], normal: normal.to_array(), uv: [1.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy + h, fz + 1.0], normal: normal.to_array(), uv: [1.0, 1.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy + h, fz + 1.0], normal: normal.to_array(), uv: [0.0, 1.0], material_id, _padding: 0 },
        ],
        FaceDir::NegZ => [
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy, fz], normal: normal.to_array(), uv: [1.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy, fz], normal: normal.to_array(), uv: [0.0, 0.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx, fy + h, fz], normal: normal.to_array(), uv: [0.0, 1.0], material_id, _padding: 0 },
            crate::rendering::mesh::MeshVertex { position: [fx + w, fy + h, fz], normal: normal.to_array(), uv: [1.0, 1.0], material_id, _padding: 0 },
        ],
    }
}

trait Vec3Ext {
    fn to_array(&self) -> [f32; 3];
}

impl Vec3Ext for lithos_engine_math::Vec3 {
    fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
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
        if x < 0 || x >= CHUNK_SIZE as i32 || y < 0 || y >= CHUNK_SIZE as i32 || z < 0 || z >= CHUNK_SIZE as i32 {
            return BlockState::air();
        }
        self.chunk.get(x as u32, y as u32, z as u32)
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