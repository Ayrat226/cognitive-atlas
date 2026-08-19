//! Matrix types for transformations

use std::fmt;
use std::ops::{Add, Mul, Index, IndexMut};
use crate::{Vec3, Vec4};
use bytemuck::{Pod, Zeroable};

/// 3x3 matrix (f32)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat3 {
    pub m: [[f32; 3]; 3],
}

impl Mat3 {
    #[inline]
    pub const fn zero() -> Self { Self { m: [[0.0; 3]; 3] } }
    #[inline]
    pub const fn identity() -> Self {
        Self { m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] }
    }
    #[inline]
    pub fn from_cols(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Self { m: [[x.x, y.x, z.x], [x.y, y.y, z.y], [x.z, y.z, z.z]] }
    }
    #[inline]
    pub fn transpose(self) -> Self {
        Self { m: [[self.m[0][0], self.m[1][0], self.m[2][0]], [self.m[0][1], self.m[1][1], self.m[2][1]], [self.m[0][2], self.m[1][2], self.m[2][2]]] }
    }
    #[inline]
    pub fn determinant(self) -> f32 {
        self.m[0][0] * (self.m[1][1] * self.m[2][2] - self.m[1][2] * self.m[2][1])
            - self.m[0][1] * (self.m[1][0] * self.m[2][2] - self.m[1][2] * self.m[2][0])
            + self.m[0][2] * (self.m[1][0] * self.m[2][1] - self.m[1][1] * self.m[2][0])
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self.m[0][0] * rhs.x + self.m[0][1] * rhs.y + self.m[0][2] * rhs.z,
            self.m[1][0] * rhs.x + self.m[1][1] * rhs.y + self.m[1][2] * rhs.z,
            self.m[2][0] * rhs.x + self.m[2][1] * rhs.y + self.m[2][2] * rhs.z,
        )
    }
}

impl Mul for Mat3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let mut result = Self::zero();
        for i in 0..3 {
            for j in 0..3 {
                result.m[i][j] = self.m[i][0] * rhs.m[0][j] + self.m[i][1] * rhs.m[1][j] + self.m[i][2] * rhs.m[2][j];
            }
        }
        result
    }
}

impl Index<usize> for Mat3 { type Output = [f32; 3]; #[inline] fn index(&self, i: usize) -> &[f32; 3] { &self.m[i] } }
impl IndexMut<usize> for Mat3 { #[inline] fn index_mut(&mut self, i: usize) -> &mut [f32; 3] { &mut self.m[i] } }
impl fmt::Debug for Mat3 { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Mat3") } }

/// 4x4 matrix (f32)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    #[inline]
    pub const fn zero() -> Self { Self { m: [[0.0; 4]; 4] } }
    #[inline]
    pub const fn identity() -> Self {
        Self { m: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] }
    }
    #[inline]
    pub fn from_translation(t: Vec3) -> Self {
        let mut m = Self::identity();
        m.m[3][0] = t.x;
        m.m[3][1] = t.y;
        m.m[3][2] = t.z;
        m
    }
    #[inline]
    pub fn from_scale(s: Vec3) -> Self {
        let mut m = Self::zero();
        m.m[0][0] = s.x;
        m.m[1][1] = s.y;
        m.m[2][2] = s.z;
        m.m[3][3] = 1.0;
        m
    }
    #[inline]
    pub fn from_rotation_x(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let mut m = Self::identity();
        m.m[1][1] = c;
        m.m[1][2] = -s;
        m.m[2][1] = s;
        m.m[2][2] = c;
        m
    }
    #[inline]
    pub fn from_rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let mut m = Self::identity();
        m.m[0][0] = c;
        m.m[0][2] = s;
        m.m[2][0] = -s;
        m.m[2][2] = c;
        m
    }
    #[inline]
    pub fn from_rotation_z(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let mut m = Self::identity();
        m.m[0][0] = c;
        m.m[0][1] = -s;
        m.m[1][0] = s;
        m.m[1][1] = c;
        m
    }
    #[inline]
    pub fn from_quaternion(q: crate::Quat) -> Self {
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;

        let mut m = Self::identity();
        m.m[0][0] = 1.0 - 2.0 * (yy + zz);
        m.m[0][1] = 2.0 * (xy + wz);
        m.m[0][2] = 2.0 * (xz - wy);
        m.m[1][0] = 2.0 * (xy - wz);
        m.m[1][1] = 1.0 - 2.0 * (xx + zz);
        m.m[1][2] = 2.0 * (yz + wx);
        m.m[2][0] = 2.0 * (xz + wy);
        m.m[2][1] = 2.0 * (yz - wx);
        m.m[2][2] = 1.0 - 2.0 * (xx + yy);
        m
    }
    #[inline]
    pub fn from_translation_rotation_scale(t: Vec3, r: crate::Quat, s: Vec3) -> Self {
        Self::from_translation(t) * Self::from_quaternion(r) * Self::from_scale(s)
    }
    #[inline]
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(f);
        let mut m = Self::identity();
        m.m[0][0] = r.x; m.m[0][1] = u.x; m.m[0][2] = -f.x;
        m.m[1][0] = r.y; m.m[1][1] = u.y; m.m[1][2] = -f.y;
        m.m[2][0] = r.z; m.m[2][1] = u.z; m.m[2][2] = -f.z;
        m.m[3][0] = -r.dot(eye);
        m.m[3][1] = -u.dot(eye);
        m.m[3][2] = f.dot(eye);
        m
    }
    #[inline]
    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fovy * 0.5).tan();
        let mut m = Self::zero();
        m.m[0][0] = f / aspect;
        m.m[1][1] = f;
        m.m[2][2] = (far + near) / (near - far);
        m.m[2][3] = -1.0;
        m.m[3][2] = 2.0 * far * near / (near - far);
        m
    }
    #[inline]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let mut m = Self::identity();
        m.m[0][0] = 2.0 / (right - left);
        m.m[1][1] = 2.0 / (top - bottom);
        m.m[2][2] = 1.0 / (near - far);
        m.m[3][0] = (left + right) / (left - right);
        m.m[3][1] = (bottom + top) / (bottom - top);
        m.m[3][2] = near / (near - far);
        m
    }
    #[inline]
    pub fn inverse(self) -> Option<Self> {
        // Simple 4x4 inverse for affine transforms
        // For general matrices, use a proper linear algebra library
        None
    }
    #[inline]
    pub fn transpose(self) -> Self {
        let mut r = Self::zero();
        for i in 0..4 { for j in 0..4 { r.m[i][j] = self.m[j][i]; } }
        r
    }
    #[inline]
    pub fn transform_point(self, p: Vec3) -> Vec3 {
        let v = self * Vec4::new(p.x, p.y, p.z, 1.0);
        if v.w != 0.0 { Vec3::new(v.x / v.w, v.y / v.w, v.z / v.w) } else { Vec3::zero() }
    }
    #[inline]
    pub fn transform_vector(self, v: Vec3) -> Vec3 {
        let r = self * Vec4::new(v.x, v.y, v.z, 0.0);
        Vec3::new(r.x, r.y, r.z)
    }
}

impl Mul for Mat4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let mut result = Self::zero();
        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = self.m[i][0] * rhs.m[0][j] + self.m[i][1] * rhs.m[1][j] + self.m[i][2] * rhs.m[2][j] + self.m[i][3] * rhs.m[3][j];
            }
        }
        result
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.m[0][0] * rhs.x + self.m[0][1] * rhs.y + self.m[0][2] * rhs.z + self.m[0][3] * rhs.w,
            self.m[1][0] * rhs.x + self.m[1][1] * rhs.y + self.m[1][2] * rhs.z + self.m[1][3] * rhs.w,
            self.m[2][0] * rhs.x + self.m[2][1] * rhs.y + self.m[2][2] * rhs.z + self.m[2][3] * rhs.w,
            self.m[3][0] * rhs.x + self.m[3][1] * rhs.y + self.m[3][2] * rhs.z + self.m[3][3] * rhs.w,
        )
    }
}

impl Index<usize> for Mat4 { type Output = [f32; 4]; #[inline] fn index(&self, i: usize) -> &[f32; 4] { &self.m[i] } }
impl IndexMut<usize> for Mat4 { #[inline] fn index_mut(&mut self, i: usize) -> &mut [f32; 4] { &mut self.m[i] } }
impl fmt::Debug for Mat4 { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Mat4") } }