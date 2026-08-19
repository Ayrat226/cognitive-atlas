//! Floating-point vector types for rendering and non-deterministic math

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg, Index, IndexMut};
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};

/// 2D vector (f32)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    #[inline]
    pub const fn zero() -> Self { Self::new(0.0, 0.0) }
    #[inline]
    pub const fn one() -> Self { Self::new(1.0, 1.0) }
    #[inline]
    pub fn dot(self, other: Self) -> f32 { self.x * other.x + self.y * other.y }
    #[inline]
    pub fn length_squared(self) -> f32 { self.dot(self) }
    #[inline]
    pub fn length(self) -> f32 { self.length_squared().sqrt() }
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::zero() } else { self / len }
    }
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self { self + (other - self) * t }
    #[inline]
    pub const fn splat(v: f32) -> Self { Self::new(v, v) }
    #[inline]
    pub fn mul_vec(self, other: Self) -> Self { Self::new(self.x * other.x, self.y * other.y) }
}

impl Add for Vec2 { type Output = Self; #[inline] fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) } }
impl Add<Vec2> for f32 { type Output = Vec2; #[inline] fn add(self, rhs: Vec2) -> Vec2 { Vec2::new(self + rhs.x, self + rhs.y) } }
impl Sub for Vec2 { type Output = Self; #[inline] fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) } }
impl Mul<f32> for Vec2 { type Output = Self; #[inline] fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs) } }
impl Mul<Vec2> for f32 { type Output = Vec2; #[inline] fn mul(self, rhs: Vec2) -> Vec2 { Vec2::new(self * rhs.x, self * rhs.y) } }
impl Div<f32> for Vec2 { type Output = Self; #[inline] fn div(self, rhs: f32) -> Self { Self::new(self.x / rhs, self.y / rhs) } }
impl Neg for Vec2 { type Output = Self; #[inline] fn neg(self) -> Self { Self::new(-self.x, -self.y) } }
impl AddAssign for Vec2 { #[inline] fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; } }
impl SubAssign for Vec2 { #[inline] fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; } }
impl MulAssign<f32> for Vec2 { #[inline] fn mul_assign(&mut self, rhs: f32) { *self = *self * rhs; } }
impl DivAssign<f32> for Vec2 { #[inline] fn div_assign(&mut self, rhs: f32) { *self = *self / rhs; } }
impl Index<usize> for Vec2 { type Output = f32; #[inline] fn index(&self, i: usize) -> &f32 { match i { 0 => &self.x, 1 => &self.y, _ => panic!("Vec2 index out of bounds") } } }
impl IndexMut<usize> for Vec2 { #[inline] fn index_mut(&mut self, i: usize) -> &mut f32 { match i { 0 => &mut self.x, 1 => &mut self.y, _ => panic!("Vec2 index out of bounds") } } }
impl fmt::Debug for Vec2 { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Vec2({}, {})", self.x, self.y) } }

/// 3D vector (f32)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    #[inline]
    pub const fn zero() -> Self { Self::new(0.0, 0.0, 0.0) }
    #[inline]
    pub const fn one() -> Self { Self::new(1.0, 1.0, 1.0) }
    #[inline]
    pub const fn x_axis() -> Self { Self::new(1.0, 0.0, 0.0) }
    #[inline]
    pub const fn y_axis() -> Self { Self::new(0.0, 1.0, 0.0) }
    #[inline]
    pub const fn z_axis() -> Self { Self::new(0.0, 0.0, 1.0) }
    #[inline]
    pub fn dot(self, other: Self) -> f32 { self.x * other.x + self.y * other.y + self.z * other.z }
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    #[inline]
    pub fn length_squared(self) -> f32 { self.dot(self) }
    #[inline]
    pub fn length(self) -> f32 { self.length_squared().sqrt() }
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::zero() } else { self / len }
    }
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self { self + (other - self) * t }
    #[inline]
    pub fn min(self, other: Self) -> Self { Self::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z)) }
    #[inline]
    pub fn max(self, other: Self) -> Self { Self::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z)) }
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self { Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z)) }
    #[inline]
    pub const fn splat(v: f32) -> Self { Self::new(v, v, v) }
    #[inline]
    pub fn mul_vec(self, other: Self) -> Self { Self::new(self.x * other.x, self.y * other.y, self.z * other.z) }
}

impl Add for Vec3 { type Output = Self; #[inline] fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z) } }
impl Sub for Vec3 { type Output = Self; #[inline] fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z) } }
impl Mul<f32> for Vec3 { type Output = Self; #[inline] fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs, self.z * rhs) } }
impl Mul<Vec3> for f32 { type Output = Vec3; #[inline] fn mul(self, rhs: Vec3) -> Vec3 { Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z) } }
impl Div<f32> for Vec3 { type Output = Self; #[inline] fn div(self, rhs: f32) -> Self { Self::new(self.x / rhs, self.y / rhs, self.z / rhs) } }
impl Neg for Vec3 { type Output = Self; #[inline] fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) } }
impl AddAssign for Vec3 { #[inline] fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; } }
impl SubAssign for Vec3 { #[inline] fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; } }
impl MulAssign<f32> for Vec3 { #[inline] fn mul_assign(&mut self, rhs: f32) { *self = *self * rhs; } }
impl DivAssign<f32> for Vec3 { #[inline] fn div_assign(&mut self, rhs: f32) { *self = *self / rhs; } }
impl Index<usize> for Vec3 { type Output = f32; #[inline] fn index(&self, i: usize) -> &f32 { match i { 0 => &self.x, 1 => &self.y, 2 => &self.z, _ => panic!("Vec3 index out of bounds") } } }
impl IndexMut<usize> for Vec3 { #[inline] fn index_mut(&mut self, i: usize) -> &mut f32 { match i { 0 => &mut self.x, 1 => &mut self.y, 2 => &mut self.z, _ => panic!("Vec3 index out of bounds") } } }
impl fmt::Debug for Vec3 { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Vec3({}, {}, {})", self.x, self.y, self.z) } }

/// 4D vector (f32)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }
    #[inline]
    pub const fn zero() -> Self { Self::new(0.0, 0.0, 0.0, 0.0) }
    #[inline]
    pub fn dot(self, other: Self) -> f32 { self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w }
    #[inline]
    pub fn length_squared(self) -> f32 { self.dot(self) }
    #[inline]
    pub fn length(self) -> f32 { self.length_squared().sqrt() }
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::zero() } else { self / len }
    }
}

impl Add for Vec4 { type Output = Self; #[inline] fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w) } }
impl Sub for Vec4 { type Output = Self; #[inline] fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w) } }
impl Mul<f32> for Vec4 { type Output = Self; #[inline] fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs) } }
impl Div<f32> for Vec4 { type Output = Self; #[inline] fn div(self, rhs: f32) -> Self { Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs) } }
impl Neg for Vec4 { type Output = Self; #[inline] fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z, -self.w) } }
impl fmt::Debug for Vec4 { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Vec4({}, {}, {}, {})", self.x, self.y, self.z, self.w) } }

/// 3D vector (f64) for high-precision intermediate calculations
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3d {
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    #[inline]
    pub const fn zero() -> Self { Self::new(0.0, 0.0, 0.0) }
    #[inline]
    pub fn dot(self, other: Self) -> f64 { self.x * other.x + self.y * other.y + self.z * other.z }
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    #[inline]
    pub fn length_squared(self) -> f64 { self.dot(self) }
    #[inline]
    pub fn length(self) -> f64 { self.length_squared().sqrt() }
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::zero() } else { self / len }
    }
}

impl Add for Vec3d { type Output = Self; #[inline] fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z) } }
impl Sub for Vec3d { type Output = Self; #[inline] fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z) } }
impl Mul<f64> for Vec3d { type Output = Self; #[inline] fn mul(self, rhs: f64) -> Self { Self::new(self.x * rhs, self.y * rhs, self.z * rhs) } }
impl Div<f64> for Vec3d { type Output = Self; #[inline] fn div(self, rhs: f64) -> Self { Self::new(self.x / rhs, self.y / rhs, self.z / rhs) } }
impl Neg for Vec3d { type Output = Self; #[inline] fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) } }
impl fmt::Debug for Vec3d { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Vec3d({}, {}, {})", self.x, self.y, self.z) } }