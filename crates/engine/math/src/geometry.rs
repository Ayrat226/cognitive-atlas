//! Geometry utilities: ray casting, intersection tests, frustum culling

use crate::{Vec3, AABB, Mat4};

/// Ray in 3D space
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub inv_direction: Vec3,
    pub t_min: f32,
    pub t_max: f32,
}

impl Ray {
    #[inline]
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        let dir = direction.normalize();
        let inv_dir = Vec3::new(
            if dir.x != 0.0 { 1.0 / dir.x } else { f32::MAX },
            if dir.y != 0.0 { 1.0 / dir.y } else { f32::MAX },
            if dir.z != 0.0 { 1.0 / dir.z } else { f32::MAX },
        );
        Self {
            origin,
            direction: dir,
            inv_direction: inv_dir,
            t_min: 0.0,
            t_max: f32::MAX,
        }
    }

    #[inline]
    pub fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Ray-AABB intersection (slab method)
#[inline]
pub fn ray_aabb(ray: &Ray, aabb: &AABB) -> Option<(f32, f32)> {
    // Component-wise division by direction (or multiplication by inverse direction)
    let t1 = Vec3::new(
        (aabb.min.x - ray.origin.x) * ray.inv_direction.x,
        (aabb.min.y - ray.origin.y) * ray.inv_direction.y,
        (aabb.min.z - ray.origin.z) * ray.inv_direction.z,
    );
    let t2 = Vec3::new(
        (aabb.max.x - ray.origin.x) * ray.inv_direction.x,
        (aabb.max.y - ray.origin.y) * ray.inv_direction.y,
        (aabb.max.z - ray.origin.z) * ray.inv_direction.z,
    );

    let t_min = t1.min(t2);
    let t_max = t1.max(t2);

    let t_enter = t_min.x.max(t_min.y).max(t_min.z).max(ray.t_min);
    let t_exit = t_max.x.min(t_max.y).min(t_max.z).min(ray.t_max);

    if t_exit >= t_enter && t_exit >= 0.0 {
        Some((t_enter, t_exit))
    } else {
        None
    }
}

/// Ray-triangle intersection (Möller–Trumbore)
#[inline]
pub fn ray_triangle(ray: &Ray, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<f32> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray.direction.cross(edge2);
    let a = edge1.dot(h);

    if a > -1e-6 && a < 1e-6 {
        return None; // Parallel
    }

    let f = 1.0 / a;
    let s = ray.origin - v0;
    let u = f * s.dot(h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray.direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * edge2.dot(q);

    if t > ray.t_min && t < ray.t_max {
        Some(t)
    } else {
        None
    }
}

/// Ray-sphere intersection
#[inline]
pub fn ray_sphere(ray: &Ray, center: Vec3, radius: f32) -> Option<(f32, f32)> {
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();
    let t1 = (-b - sqrt_d) / (2.0 * a);
    let t2 = (-b + sqrt_d) / (2.0 * a);

    if t2 < ray.t_min || t1 > ray.t_max {
        return None;
    }

    Some((t1.max(ray.t_min), t2.min(ray.t_max)))
}

/// Plane representation
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32, // Signed distance from origin
}

impl Plane {
    #[inline]
    pub fn new(normal: Vec3, point: Vec3) -> Self {
        let n = normal.normalize();
        Self { normal: n, distance: n.dot(point) }
    }

    #[inline]
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let n = (b - a).cross(c - a).normalize();
        Self { normal: n, distance: n.dot(a) }
    }

    #[inline]
    pub fn signed_distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.normal.length();
        if len == 0.0 { self } else {
            Self { normal: self.normal / len, distance: self.distance / len }
        }
    }
}

/// Frustum for culling
#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6], // Left, Right, Bottom, Top, Near, Far
}

impl Frustum {
    /// Extract frustum planes from view-projection matrix
    pub fn from_view_proj(view_proj: Mat4) -> Self {
        let m = view_proj.m;
        let mut planes = [Plane::new(Vec3::zero(), Vec3::zero()); 6];

        // Left:   col3 + col0
        planes[0] = Plane::new(
            Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
            Vec3::zero(),
        ).normalize();

        // Right:  col3 - col0
        planes[1] = Plane::new(
            Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
            Vec3::zero(),
        ).normalize();

        // Bottom: col3 + col1
        planes[2] = Plane::new(
            Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
            Vec3::zero(),
        ).normalize();

        // Top:    col3 - col1
        planes[3] = Plane::new(
            Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
            Vec3::zero(),
        ).normalize();

        // Near:   col3 + col2
        planes[4] = Plane::new(
            Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
            Vec3::zero(),
        ).normalize();

        // Far:    col3 - col2
        planes[5] = Plane::new(
            Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
            Vec3::zero(),
        ).normalize();

        Self { planes }
    }

    /// Test AABB against frustum
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            // Find the vertex of the AABB that is most in the direction of the plane normal
            let positive = Vec3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            if plane.signed_distance(positive) < 0.0 {
                return false; // AABB is outside this plane
            }
        }
        true
    }

    /// Test sphere against frustum
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.signed_distance(center) < -radius {
                return false;
            }
        }
        true
    }
}

/// Triangle
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
}

impl Triangle {
    #[inline]
    pub fn normal(&self) -> Vec3 {
        (self.v1 - self.v0).cross(self.v2 - self.v0).normalize()
    }

    #[inline]
    pub fn area(&self) -> f32 {
        ((self.v1 - self.v0).cross(self.v2 - self.v0).length()) * 0.5
    }

    #[inline]
    pub fn centroid(&self) -> Vec3 {
        (self.v0 + self.v1 + self.v2) / 3.0
    }

    #[inline]
    pub fn aabb(&self) -> AABB {
        let min = self.v0.min(self.v1).min(self.v2);
        let max = self.v0.max(self.v1).max(self.v2);
        AABB::new(min, max)
    }
}

/// Barycentric coordinates
#[inline]
pub fn barycentric(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> (f32, f32, f32) {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;
    if denom == 0.0 { return (0.0, 0.0, 0.0); }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    (u, v, w)
}

/// Closest point on triangle to point
#[inline]
pub fn closest_point_triangle(p: Vec3, tri: Triangle) -> Vec3 {
    // Simplified - for full implementation use proper algorithm
    // This projects onto plane and clamps to barycentric
    let n = tri.normal();
    let proj = p - n * (p - tri.v0).dot(n);
    let (u, v, w) = barycentric(proj, tri.v0, tri.v1, tri.v2);
    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
        proj
    } else {
        // Clamp to edges - simplified
        tri.v0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_aabb() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
        let aabb = AABB::from_center_half_extents(Vec3::zero(), Vec3::one());
        let hit = ray_aabb(&ray, &aabb);
        assert!(hit.is_some());
        let (t1, t2) = hit.unwrap();
        assert!((t1 - 4.0).abs() < 0.001);
        assert!((t2 - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_ray_triangle() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let v0 = Vec3::new(-1.0, -1.0, 0.0);
        let v1 = Vec3::new(1.0, -1.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        let hit = ray_triangle(&ray, v0, v1, v2);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_frustum_culling() {
        let proj = Mat4::perspective(1.0, 1.0, 0.1, 100.0);
        let view = Mat4::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::zero(), Vec3::y_axis());
        let frustum = Frustum::from_view_proj(view * proj);

        let aabb_inside = AABB::from_center_half_extents(Vec3::zero(), Vec3::new(0.5, 0.5, 0.5));
        assert!(frustum.intersects_aabb(&aabb_inside));

        let aabb_outside = AABB::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::one());
        assert!(!frustum.intersects_aabb(&aabb_outside));
    }
}