//! Quaternion for rotations

use std::fmt;
use std::ops::{Mul, MulAssign, Div};
use crate::Vec3;
use bytemuck::{Pod, Zeroable};

/// Quaternion (f32)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }
    #[inline]
    pub const fn identity() -> Self { Self::new(0.0, 0.0, 0.0, 1.0) }
    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half = angle * 0.5;
        let s = half.sin();
        Self::new(axis.x * s, axis.y * s, axis.z * s, half.cos())
    }
    #[inline]
    pub fn from_euler(x: f32, y: f32, z: f32) -> Self {
        let cx = (x * 0.5).cos();
        let sx = (x * 0.5).sin();
        let cy = (y * 0.5).cos();
        let sy = (y * 0.5).sin();
        let cz = (z * 0.5).cos();
        let sz = (z * 0.5).sin();

        Self::new(
            sx * cy * cz - cx * sy * sz,
            cx * sy * cz + sx * cy * sz,
            cx * cy * sz - sx * sy * cz,
            cx * cy * cz + sx * sy * sz,
        )
    }
    #[inline]
    pub fn from_mat3(m: crate::Mat3) -> Self {
        let trace = m.m[0][0] + m.m[1][1] + m.m[2][2];
        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Self::new(
                (m.m[2][1] - m.m[1][2]) / s,
                (m.m[0][2] - m.m[2][0]) / s,
                (m.m[1][0] - m.m[0][1]) / s,
                0.25 * s,
            )
        } else if m.m[0][0] > m.m[1][1] && m.m[0][0] > m.m[2][2] {
            let s = (1.0 + m.m[0][0] - m.m[1][1] - m.m[2][2]).sqrt() * 2.0;
            Self::new(
                0.25 * s,
                (m.m[0][1] + m.m[1][0]) / s,
                (m.m[0][2] + m.m[2][0]) / s,
                (m.m[2][1] - m.m[1][2]) / s,
            )
        } else if m.m[1][1] > m.m[2][2] {
            let s = (1.0 + m.m[1][1] - m.m[0][0] - m.m[2][2]).sqrt() * 2.0;
            Self::new(
                (m.m[0][1] + m.m[1][0]) / s,
                0.25 * s,
                (m.m[1][2] + m.m[2][1]) / s,
                (m.m[0][2] - m.m[2][0]) / s,
            )
        } else {
            let s = (1.0 + m.m[2][2] - m.m[0][0] - m.m[1][1]).sqrt() * 2.0;
            Self::new(
                (m.m[0][2] + m.m[2][0]) / s,
                (m.m[1][2] + m.m[2][1]) / s,
                0.25 * s,
                (m.m[1][0] - m.m[0][1]) / s,
            )
        }
    }
    #[inline]
    pub fn dot(self, other: Self) -> f32 { self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w }
    #[inline]
    pub fn length_squared(self) -> f32 { self.dot(self) }
    #[inline]
    pub fn length(self) -> f32 { self.length_squared().sqrt() }
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::identity() } else { self / len }
    }
    #[inline]
    pub fn conjugate(self) -> Self { Self::new(-self.x, -self.y, -self.z, self.w) }
    #[inline]
    pub fn inverse(self) -> Self { self.conjugate() / self.length_squared() }
    #[inline]
    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut cos_theta = self.dot(other);
        let mut end = other;
        if cos_theta < 0.0 {
            cos_theta = -cos_theta;
            end = Self::new(-other.x, -other.y, -other.z, -other.w);
        }
        if cos_theta > 0.9995 {
            return self.lerp(end, t).normalize();
        }
        let theta = cos_theta.acos();
        let sin_theta = theta.sin();
        let a = ((1.0 - t) * theta).sin() / sin_theta;
        let b = (t * theta).sin() / sin_theta;
        Self::new(
            self.x * a + end.x * b,
            self.y * a + end.y * b,
            self.z * a + end.z * b,
            self.w * a + end.w * b,
        )
    }
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
            self.w + (other.w - self.w) * t,
        )
    }
    #[inline]
    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let qv = Self::new(v.x, v.y, v.z, 0.0);
        let result = self * qv * self.conjugate();
        Vec3::new(result.x, result.y, result.z)
    }
}

impl Mul for Quat {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        )
    }
}

impl MulAssign for Quat {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
}

impl Div<f32> for Quat {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl fmt::Debug for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Quat({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}