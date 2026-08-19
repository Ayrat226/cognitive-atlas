//! Axis-Aligned Bounding Box (f32)

use std::fmt;
use crate::Vec3;
use bytemuck::{Pod, Zeroable};

/// 3D AABB (f32)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline]
    pub const fn new(min: Vec3, max: Vec3) -> Self { Self { min, max } }
    #[inline]
    pub fn empty() -> Self { Self::new(Vec3::new(f32::MAX, f32::MAX, f32::MAX), Vec3::new(f32::MIN, f32::MIN, f32::MIN)) }
    #[inline]
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self { Self::new(center - half_extents, center + half_extents) }
    #[inline]
    pub fn center(&self) -> Vec3 { (self.min + self.max) * 0.5 }
    #[inline]
    pub fn half_extents(&self) -> Vec3 { (self.max - self.min) * 0.5 }
    #[inline]
    pub fn size(&self) -> Vec3 { self.max - self.min }
    #[inline]
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
    #[inline]
    pub fn expand(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
    #[inline]
    pub fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
    #[inline]
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }
    #[inline]
    pub fn surface_area(&self) -> f32 {
        let size = self.size();
        2.0 * (size.x * size.y + size.y * size.z + size.z * size.x)
    }
    #[inline]
    pub fn ray_intersect(&self, origin: Vec3, dir: Vec3) -> Option<(f32, f32)> {
        let inv_dir = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
        let t1 = (self.min - origin) * inv_dir;
        let t2 = (self.max - origin) * inv_dir;
        let t_min = t1.min(t2);
        let t_max = t1.max(t2);
        let t_enter = t_min.x.max(t_min.y).max(t_min.z);
        let t_exit = t_max.x.min(t_max.y).min(t_max.z);
        if t_exit >= t_enter && t_exit >= 0.0 {
            Some((t_enter.max(0.0), t_exit))
        } else {
            None
        }
    }
}

impl fmt::Debug for AABB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AABB(min: {:?}, max: {:?})", self.min, self.max)
    }
}