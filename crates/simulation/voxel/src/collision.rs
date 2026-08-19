//! Voxel world collision detection
//! Broad phase (chunk-level) + Narrow phase (block-level) + Capsule collision

use crate::coords::{BlockPos, ChunkKey, CHUNK_SIZE};
use crate::block::BlockState;
use crate::chunk::ChunkStorage;
use crate::storage::VoxelStorage;
use lithos_engine_math::{FixedVec3, FixedAABB, Fixed};

/// Capsule shape for player collision
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsule {
    /// Bottom center point
    pub base: FixedVec3,
    /// Top center point  
    pub tip: FixedVec3,
    /// Radius
    pub radius: Fixed,
}

impl Capsule {
    /// Create capsule from center, height, and radius
    #[inline]
    pub fn from_center_height_radius(center: FixedVec3, height: Fixed, radius: Fixed) -> Self {
        let half_height = height / Fixed::from_int(2);
        Self {
            base: center - FixedVec3::new(Fixed::ZERO, half_height, Fixed::ZERO),
            tip: center + FixedVec3::new(Fixed::ZERO, half_height, Fixed::ZERO),
            radius,
        }
    }

    /// Get AABB bounds of capsule
    #[inline]
    pub fn aabb(&self) -> FixedAABB {
        let r = FixedVec3::new(self.radius, self.radius, self.radius);
        let min = self.base.min(self.tip) - r;
        let max = self.base.max(self.tip) + r;
        FixedAABB::new(min, max)
    }

    /// Move capsule by offset
    #[inline]
    pub fn translate(&self, offset: FixedVec3) -> Self {
        Self {
            base: self.base + offset,
            tip: self.tip + offset,
            radius: self.radius,
        }
    }

    /// Get center point
    #[inline]
    pub fn center(&self) -> FixedVec3 {
        (self.base + self.tip) / Fixed::from_int(2)
    }

    /// Get height
    #[inline]
    pub fn height(&self) -> Fixed {
        (self.tip - self.base).y.abs()
    }
}

/// Collision contact point
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionContact {
    /// Contact point in world space
    pub point: FixedVec3,
    /// Contact normal (pointing out of collider)
    pub normal: FixedVec3,
    /// Penetration depth
    pub depth: Fixed,
    /// Block position that caused collision
    pub block_pos: BlockPos,
}

/// Collision query result
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionResult {
    /// All contacts found
    pub contacts: Vec<CollisionContact>,
    /// Whether any collision occurred
    pub collided: bool,
    /// Combined collision normal (for sliding)
    pub collision_normal: FixedVec3,
    /// Maximum penetration depth
    pub max_depth: Fixed,
}

impl CollisionResult {
    #[inline]
    pub fn new() -> Self {
        Self {
            contacts: Vec::new(),
            collided: false,
            collision_normal: FixedVec3::zero(),
            max_depth: Fixed::ZERO,
        }
    }

    #[inline]
    pub fn add_contact(&mut self, contact: CollisionContact) {
        self.collided = true;
        self.contacts.push(contact);
        self.collision_normal = self.collision_normal + contact.normal;
        if contact.depth > self.max_depth {
            self.max_depth = contact.depth;
        }
    }

    /// Normalize the combined collision normal
    #[inline]
    pub fn finalize(&mut self) {
        if self.collided {
            let len_sq = self.collision_normal.length_squared();
            if len_sq > Fixed::ZERO {
                self.collision_normal = self.collision_normal / len_sq.sqrt();
            }
        }
    }
}

impl Default for CollisionResult {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Voxel collision detector
pub struct VoxelCollider<'a> {
    storage: &'a VoxelStorage,
    /// Maximum blocks to check per query (performance limit)
    max_blocks: usize,
    /// Blocks checked in current query
    blocks_checked: usize,
}

impl<'a> VoxelCollider<'a> {
    /// Create new collider
    #[inline]
    pub fn new(storage: &'a VoxelStorage) -> Self {
        Self {
            storage,
            max_blocks: 4096,
            blocks_checked: 0,
        }
    }

    /// Set maximum blocks to check
    #[inline]
    pub fn with_max_blocks(mut self, max: usize) -> Self {
        self.max_blocks = max;
        self
    }

    /// Check capsule vs voxel world collision
    pub fn check_capsule(&mut self, capsule: Capsule) -> CollisionResult {
        self.blocks_checked = 0;
        let mut result = CollisionResult::new();
        
        let aabb = capsule.aabb();
        let min_block = BlockPos::from_fixed_vec3(aabb.min);
        let max_block = BlockPos::from_fixed_vec3(aabb.max);
        
        // Broad phase: iterate chunks overlapping AABB
        let min_chunk = min_block.chunk_key();
        let max_chunk = max_block.chunk_key();
        
        for cx in min_chunk.x..=max_chunk.x {
            for cy in min_chunk.y..=max_chunk.y {
                for cz in min_chunk.z..=max_chunk.z {
                    let chunk_key = ChunkKey::new(cx, cy, cz);
                    if self.blocks_checked >= self.max_blocks {
                        break;
                    }
                    self.check_chunk_capsule(chunk_key, capsule, &mut result);
                }
            }
        }
        
        result.finalize();
        result
    }

    /// Check AABB vs voxel world collision
    pub fn check_aabb(&mut self, aabb: FixedAABB) -> CollisionResult {
        self.blocks_checked = 0;
        let mut result = CollisionResult::new();
        
        let min_block = BlockPos::from_fixed_vec3(aabb.min);
        let max_block = BlockPos::from_fixed_vec3(aabb.max);
        
        let min_chunk = min_block.chunk_key();
        let max_chunk = max_block.chunk_key();
        
        for cx in min_chunk.x..=max_chunk.x {
            for cy in min_chunk.y..=max_chunk.y {
                for cz in min_chunk.z..=max_chunk.z {
                    let chunk_key = ChunkKey::new(cx, cy, cz);
                    if self.blocks_checked >= self.max_blocks {
                        break;
                    }
                    self.check_chunk_aabb(chunk_key, aabb, &mut result);
                }
            }
        }
        
        result.finalize();
        result
    }

    /// Check single chunk against capsule
    fn check_chunk_capsule(&mut self, chunk_key: ChunkKey, capsule: Capsule, result: &mut CollisionResult) {
        let chunk = self.storage.get_chunk(chunk_key);
        
        // Quick AABB check first
        let chunk_aabb = chunk_aabb(chunk_key);
        if !capsule.aabb().intersects(&chunk_aabb) {
            return;
        }
        
        // Narrow phase: check each block in chunk
        let chunk_min = chunk_key.min_block_pos();
        
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.blocks_checked >= self.max_blocks {
                        return;
                    }
                    self.blocks_checked += 1;
                    
                    let block_pos = BlockPos::new(
                        chunk_min.x + x as i64,
                        chunk_min.y + y as i64,
                        chunk_min.z + z as i64,
                    );
                    
                    let block = chunk.data.get_block(x as u32, y as u32, z as u32);
                    if block.block_id.is_air() {
                        continue;
                    }
                    
                    let block_aabb = block_aabb(block_pos);
                    if let Some(contact) = capsule_aabb_collision(capsule, block_aabb, block_pos) {
                        result.add_contact(contact);
                    }
                }
            }
        }
    }

    /// Check single chunk against AABB
    fn check_chunk_aabb(&mut self, chunk_key: ChunkKey, aabb: FixedAABB, result: &mut CollisionResult) {
        let chunk = self.storage.get_chunk(chunk_key);
        
        let chunk_aabb = chunk_aabb(chunk_key);
        if !aabb.intersects(&chunk_aabb) {
            return;
        }
        
        let chunk_min = chunk_key.min_block_pos();
        
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if self.blocks_checked >= self.max_blocks {
                        return;
                    }
                    self.blocks_checked += 1;
                    
                    let block_pos = BlockPos::new(
                        chunk_min.x + x as i64,
                        chunk_min.y + y as i64,
                        chunk_min.z + z as i64,
                    );
                    
                    let block = chunk.data.get_block(x as u32, y as u32, z as u32);
                    if block.block_id.is_air() {
                        continue;
                    }
                    
                    let block_aabb = block_aabb(block_pos);
                    if let Some(contact) = aabb_aabb_collision(aabb, block_aabb, block_pos) {
                        result.add_contact(contact);
                    }
                }
            }
        }
    }

    /// Sweep capsule through world (for movement prediction)
    pub fn sweep_capsule(&mut self, start: Capsule, end: Capsule, step: Fixed) -> CollisionResult {
        let mut result = CollisionResult::new();
        let delta = end.center() - start.center();
        let distance = delta.length();
        
        if distance == Fixed::ZERO {
            return self.check_capsule(start);
        }
        
        let steps = ((distance / step).floor() as u32).max(1);
        let step_vec = delta / Fixed::from_int(steps as i64);
        
        let mut current = start;
        for _ in 0..steps {
            current = current.translate(step_vec);
            let step_result = self.check_capsule(current);
            if step_result.collided {
                // Merge contacts
                for contact in step_result.contacts {
                    result.add_contact(contact);
                }
                break; // Stop at first collision
            }
        }
        
        result.finalize();
        result
    }
}

/// Get AABB for a chunk
#[inline]
fn chunk_aabb(chunk_key: ChunkKey) -> FixedAABB {
    let min = chunk_key.min_block_pos().to_fixed_vec3();
    let size = FixedVec3::new(
        Fixed::from_int(CHUNK_SIZE as i64),
        Fixed::from_int(CHUNK_SIZE as i64),
        Fixed::from_int(CHUNK_SIZE as i64),
    );
    FixedAABB::new(min, min + size)
}

/// Get AABB for a block at position
#[inline]
fn block_aabb(pos: BlockPos) -> FixedAABB {
    let min = pos.to_fixed_vec3();
    let one = Fixed::ONE;
    FixedAABB::new(min, min + FixedVec3::new(one, one, one))
}

/// Capsule vs AABB collision detection
/// Returns contact if colliding
fn capsule_aabb_collision(capsule: Capsule, aabb: FixedAABB, block_pos: BlockPos) -> Option<CollisionContact> {
    // Find closest point on AABB to capsule segment
    let closest = closest_point_on_segment_to_aabb(capsule.base, capsule.tip, aabb);
    
    // Check distance from closest point to capsule segment
    let dist_sq = distance_squared_point_segment(closest, capsule.base, capsule.tip);
    let radius_sq = capsule.radius * capsule.radius;
    
    if dist_sq >= radius_sq {
        return None;
    }
    
    // Compute penetration depth and normal
    let dist = dist_sq.sqrt();
    let depth = capsule.radius - dist;
    
    // Compute normal (from block to capsule)
    let normal = if dist > Fixed::ZERO {
        (closest - closest_point_on_segment(closest, capsule.base, capsule.tip)).normalize()
    } else {
        // Capsule center inside block - push up
        FixedVec3::y_axis()
    };
    
    Some(CollisionContact {
        point: closest,
        normal,
        depth,
        block_pos,
    })
}

/// AABB vs AABB collision detection
fn aabb_aabb_collision(a: FixedAABB, b: FixedAABB, block_pos: BlockPos) -> Option<CollisionContact> {
    if !a.intersects(&b) {
        return None;
    }
    
    // Compute overlap on each axis
    let overlap_x = (a.max.x - b.min.x).min(b.max.x - a.min.x);
    let overlap_y = (a.max.y - b.min.y).min(b.max.y - a.min.y);
    let overlap_z = (a.max.z - b.min.z).min(b.max.z - a.min.z);
    
    // Find minimum overlap axis
    let (min_overlap, normal) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
        (overlap_x, if a.center().x < b.center().x { FixedVec3::new(-Fixed::ONE, Fixed::ZERO, Fixed::ZERO) } else { FixedVec3::x_axis() })
    } else if overlap_y <= overlap_z {
        (overlap_y, if a.center().y < b.center().y { FixedVec3::new(Fixed::ZERO, -Fixed::ONE, Fixed::ZERO) } else { FixedVec3::y_axis() })
    } else {
        (overlap_z, if a.center().z < b.center().z { FixedVec3::new(Fixed::ZERO, Fixed::ZERO, -Fixed::ONE) } else { FixedVec3::z_axis() })
    };
    
    let contact_point = a.center() + normal * (min_overlap / Fixed::from_int(2));
    
    Some(CollisionContact {
        point: contact_point,
        normal,
        depth: min_overlap,
        block_pos,
    })
}

/// Find closest point on line segment to AABB
fn closest_point_on_segment_to_aabb(seg_a: FixedVec3, seg_b: FixedVec3, aabb: FixedAABB) -> FixedVec3 {
    let seg_dir = seg_b - seg_a;
    let seg_len_sq = seg_dir.length_squared();
    
    if seg_len_sq == Fixed::ZERO {
        return closest_point_on_aabb(seg_a, aabb);
    }
    
    // Sample points along segment and find closest to AABB
    // Use 5 samples for good accuracy
    let mut best_point = seg_a;
    let mut best_dist_sq = Fixed::MAX;
    
    for i in 0..=4 {
        let t = Fixed::from_int(i) / Fixed::from_int(4);
        let point = seg_a + seg_dir * t;
        let closest = closest_point_on_aabb(point, aabb);
        let dist_sq = (point - closest).length_squared();
        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_point = closest;
        }
    }
    
    best_point
}

/// Find closest point on AABB to point
#[inline]
fn closest_point_on_aabb(point: FixedVec3, aabb: FixedAABB) -> FixedVec3 {
    FixedVec3::new(
        point.x.clamp(aabb.min.x, aabb.max.x),
        point.y.clamp(aabb.min.y, aabb.max.y),
        point.z.clamp(aabb.min.z, aabb.max.z),
    )
}

/// Find closest point on segment to point
#[inline]
fn closest_point_on_segment(point: FixedVec3, seg_a: FixedVec3, seg_b: FixedVec3) -> FixedVec3 {
    let seg_dir = seg_b - seg_a;
    let seg_len_sq = seg_dir.length_squared();
    
    if seg_len_sq == Fixed::ZERO {
        return seg_a;
    }
    
    let t = (point - seg_a).dot(seg_dir) / seg_len_sq;
    let t = t.clamp(Fixed::ZERO, Fixed::ONE);
    seg_a + seg_dir * t
}

/// Distance squared from point to segment
#[inline]
fn distance_squared_point_segment(point: FixedVec3, seg_a: FixedVec3, seg_b: FixedVec3) -> Fixed {
    let closest = closest_point_on_segment(point, seg_a, seg_b);
    (point - closest).length_squared()
}

/// Check if position is on ground (block below)
pub fn is_on_ground(storage: &VoxelStorage, pos: FixedVec3, _radius: Fixed, height: Fixed) -> bool {
    let check_pos = pos - FixedVec3::new(Fixed::ZERO, height / Fixed::from_int(2) + Fixed::from_int(1), Fixed::ZERO);
    let block_pos = BlockPos::from_fixed_vec3(check_pos);
    let block = storage.get_block(block_pos);
    !block.block_id.is_air()
}

/// Check if position is in water
pub fn is_in_water(storage: &VoxelStorage, pos: FixedVec3) -> bool {
    let block_pos = BlockPos::from_fixed_vec3(pos);
    let block = storage.get_block(block_pos);
    matches!(block.block_id, crate::block::BlockId::WATER)
}

/// Check if position is in lava
pub fn is_in_lava(storage: &VoxelStorage, pos: FixedVec3) -> bool {
    let block_pos = BlockPos::from_fixed_vec3(pos);
    let block = storage.get_block(block_pos);
    // LAVA not defined yet, return false
    let _ = block;
    false
}

/// Get block at position
pub fn get_block_at(storage: &VoxelStorage, pos: FixedVec3) -> BlockState {
    let block_pos = BlockPos::from_fixed_vec3(pos);
    storage.get_block(block_pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{BlockId, BlockState};
    use std::path::PathBuf;

    #[test]
    fn test_capsule_aabb() {
        let temp_dir = std::env::temp_dir().join("lithos_test_collision");
        let storage = VoxelStorage::new(temp_dir.clone(), 42);
        
        // Place solid blocks
        for x in 0..5 {
            for z in 0..5 {
                storage.set_block(BlockPos::new(x, 0, z), BlockState::solid(BlockId::STONE));
            }
        }
        
        let mut collider = VoxelCollider::new(&storage);
        
        // Capsule standing on floor
        let capsule = Capsule::from_center_height_radius(
            FixedVec3::new(Fixed::from_int(2), Fixed::from_int(2), Fixed::from_int(2)),
            Fixed::from_int(2),
            Fixed::from_f32(0.3),
        );
        
        let result = collider.check_capsule(capsule);
        assert!(result.collided);
        assert!(result.max_depth > Fixed::ZERO);
    }

    #[test]
    fn test_capsule_no_collision() {
        let temp_dir = std::env::temp_dir().join("lithos_test_collision2");
        let storage = VoxelStorage::new(temp_dir.clone(), 42);
        
        let mut collider = VoxelCollider::new(&storage);
        
        // Capsule in empty space
        let capsule = Capsule::from_center_height_radius(
            FixedVec3::new(Fixed::from_int(2), Fixed::from_int(50), Fixed::from_int(2)),
            Fixed::from_int(2),
            Fixed::from_f32(0.3),
        );
        
        let result = collider.check_capsule(capsule);
        assert!(!result.collided);
    }

    #[test]
    fn test_sweep_capsule() {
        let temp_dir = std::env::temp_dir().join("lithos_test_sweep");
        let storage = VoxelStorage::new(temp_dir.clone(), 42);
        
        // Place wall
        for y in 0..5 {
            storage.set_block(BlockPos::new(5, y, 2), BlockState::solid(BlockId::STONE));
        }
        
        let mut collider = VoxelCollider::new(&storage);
        
        let start = Capsule::from_center_height_radius(
            FixedVec3::new(Fixed::from_int(0), Fixed::from_int(2), Fixed::from_int(2)),
            Fixed::from_int(2),
            Fixed::from_f32(0.3),
        );
        
        let end = Capsule::from_center_height_radius(
            FixedVec3::new(Fixed::from_int(10), Fixed::from_int(2), Fixed::from_int(2)),
            Fixed::from_int(2),
            Fixed::from_f32(0.3),
        );
        
        let result = collider.sweep_capsule(start, end, Fixed::from_f32(0.5));
        assert!(result.collided);
    }

    #[test]
    fn test_is_on_ground() {
        let temp_dir = std::env::temp_dir().join("lithos_test_ground");
        let storage = VoxelStorage::new(temp_dir.clone(), 42);
        
        storage.set_block(BlockPos::new(0, 0, 0), BlockState::solid(BlockId::STONE));
        
        let pos = FixedVec3::new(Fixed::ZERO, Fixed::from_int(1), Fixed::ZERO);
        assert!(is_on_ground(&storage, pos, Fixed::from_f32(0.3), Fixed::from_int(2)));
        
        let pos_air = FixedVec3::new(Fixed::ZERO, Fixed::from_int(5), Fixed::ZERO);
        assert!(!is_on_ground(&storage, pos_air, Fixed::from_f32(0.3), Fixed::from_int(2)));
    }
}