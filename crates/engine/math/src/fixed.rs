//! Fixed-point arithmetic for deterministic simulation
//! Q52.11 format: 52 integer bits, 11 fractional bits = 1/2048 precision
//! Range: ±2^51 ≈ ±2.25e15 (2.25 petameters at 1/2048m units)

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg, Rem, RemAssign};
use std::cmp::Ordering;
use num_traits::{Zero, One, Num, Signed, Bounded, NumOps};
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};

/// Fixed-point number with 11 fractional bits (Q52.11)
/// 1 unit = 1/2048 meter ≈ 0.488 mm
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable, Serialize, Deserialize)]
pub struct Fixed(i64);

impl Fixed {
    /// Fractional bits
    pub const FRAC_BITS: i32 = 11;
    /// Fractional mask
    pub const FRAC_MASK: i64 = (1 << Self::FRAC_BITS) - 1;
    /// One unit in fixed-point
    pub const ONE: Self = Self(1 << Self::FRAC_BITS);
    /// Zero
    pub const ZERO: Self = Self(0);
    /// Minimum value
    pub const MIN: Self = Self(i64::MIN);
    /// Maximum value
    pub const MAX: Self = Self(i64::MAX);
    /// Epsilon (smallest representable difference)
    pub const EPSILON: Self = Self(1);

    /// Create from raw bits
    #[inline]
    pub const fn from_bits(bits: i64) -> Self {
        Self(bits)
    }

    /// Get raw bits
    #[inline]
    pub const fn to_bits(self) -> i64 {
        self.0
    }

    /// Create from integer value
    #[inline]
    pub const fn from_int(val: i64) -> Self {
        Self(val << Self::FRAC_BITS)
    }

    /// Create from float (non-deterministic, use only for init)
    #[inline]
    pub fn from_f64(val: f64) -> Self {
        Self((val * (1 << Self::FRAC_BITS) as f64).round() as i64)
    }

    /// Create from float (non-deterministic, use only for init)
    #[inline]
    pub fn from_f32(val: f32) -> Self {
        Self((val * (1 << Self::FRAC_BITS) as f32).round() as i64)
    }

    /// Convert to f64 (non-deterministic)
    #[inline]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / (1 << Self::FRAC_BITS) as f64
    }

    /// Convert to f32 (non-deterministic)
    #[inline]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / (1 << Self::FRAC_BITS) as f32
    }

    /// Get integer part
    #[inline]
    pub const fn floor(self) -> i64 {
        self.0 >> Self::FRAC_BITS
    }

    /// Get fractional part
    #[inline]
    pub const fn fract(self) -> Self {
        Self(self.0 & Self::FRAC_MASK)
    }

    /// Absolute value
    #[inline]
    pub const fn abs(self) -> Self {
        if self.0 < 0 {
            Self(-self.0)
        } else {
            self
        }
    }

    /// Square root (integer approximation)
    pub fn sqrt(self) -> Self {
        if self.0 <= 0 {
            return Self::ZERO;
        }
        // Newton's method for fixed-point sqrt
        let mut x = self.0;
        let mut y = (x + 1) >> 1;
        while y < x {
            x = y;
            y = (x + self.0 / x) >> 1;
        }
        Self(x)
    }

    /// Sine approximation (using Taylor series)
    pub fn sin(self) -> Self {
        // Reduce to [-π, π]
        let pi = Self::from_f64(std::f64::consts::PI);
        let two_pi = pi + pi;
        let mut x = self;
        while x > pi {
            x = x - two_pi;
        }
        while x < -pi {
            x = x + two_pi;
        }

        // Taylor series: x - x^3/6 + x^5/120 - x^7/5040
        let x2 = x * x;
        let x3 = x2 * x;
        let x5 = x3 * x2;
        let x7 = x5 * x2;

        x - x3 / Self::from_int(6)
            + x5 / Self::from_int(120)
            - x7 / Self::from_int(5040)
    }

    /// Cosine approximation
    pub fn cos(self) -> Self {
        (Self::from_f64(std::f64::consts::PI) / Self::from_int(2) - self).sin()
    }

    /// Clamp to range
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    /// Linear interpolation
    #[inline]
    pub fn lerp(self, other: Self, t: Self) -> Self {
        self + (other - self) * t
    }
}

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fixed({:.6})", self.to_f64())
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.to_f64())
    }
}

impl Default for Fixed {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Zero for Fixed {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl One for Fixed {
    fn one() -> Self {
        Self::ONE
    }
}

impl Neg for Fixed {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Add for Fixed {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl AddAssign for Fixed {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl Sub for Fixed {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl SubAssign for Fixed {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

impl Mul for Fixed {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        // (a * b) >> FRAC_BITS
        let result = (self.0 as i128 * rhs.0 as i128) >> Self::FRAC_BITS;
        Self(result as i64)
    }
}

impl MulAssign for Fixed {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for Fixed {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        // (a << FRAC_BITS) / b
        let result = ((self.0 as i128) << Self::FRAC_BITS) / rhs.0 as i128;
        Self(result as i64)
    }
}

impl DivAssign for Fixed {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Rem for Fixed {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}

impl RemAssign for Fixed {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl PartialOrd for Fixed {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fixed {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Num for Fixed {
    type FromStrRadixErr = std::num::ParseIntError;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        i64::from_str_radix(str, radix).map(Self)
    }
}

impl Signed for Fixed {
    fn abs(&self) -> Self {
        if self.0 < 0 {
            Self(-self.0)
        } else {
            *self
        }
    }
    fn abs_sub(&self, other: &Self) -> Self {
        if self > other {
            *self - *other
        } else {
            Self::ZERO
        }
    }
    fn signum(&self) -> Self {
        if self.0 > 0 {
            Self::ONE
        } else if self.0 < 0 {
            -Self::ONE
        } else {
            Self::ZERO
        }
    }
    fn is_positive(&self) -> bool {
        self.0 > 0
    }
    fn is_negative(&self) -> bool {
        self.0 < 0
    }
}

impl Bounded for Fixed {
    fn min_value() -> Self {
        Self::MIN
    }
    fn max_value() -> Self {
        Self::MAX
    }
}

/// 3D vector with fixed-point components
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable, Serialize, Deserialize)]
pub struct FixedVec3 {
    pub x: Fixed,
    pub y: Fixed,
    pub z: Fixed,
}

impl FixedVec3 {
    #[inline]
    pub const fn new(x: Fixed, y: Fixed, z: Fixed) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO)
    }

    #[inline]
    pub const fn one() -> Self {
        Self::new(Fixed::ONE, Fixed::ONE, Fixed::ONE)
    }

    #[inline]
    pub const fn x_axis() -> Self {
        Self::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO)
    }

    #[inline]
    pub const fn y_axis() -> Self {
        Self::new(Fixed::ZERO, Fixed::ONE, Fixed::ZERO)
    }

    #[inline]
    pub const fn z_axis() -> Self {
        Self::new(Fixed::ZERO, Fixed::ZERO, Fixed::ONE)
    }

    #[inline]
    pub fn dot(self, other: Self) -> Fixed {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn length_squared(self) -> Fixed {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> Fixed {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == Fixed::ZERO {
            Self::zero()
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.clamp(min.x, max.x),
            self.y.clamp(min.y, max.y),
            self.z.clamp(min.z, max.z),
        )
    }

    #[inline]
    pub fn lerp(self, other: Self, t: Fixed) -> Self {
        Self::new(
            self.x.lerp(other.x, t),
            self.y.lerp(other.y, t),
            self.z.lerp(other.z, t),
        )
    }
}

impl Add for FixedVec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for FixedVec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for FixedVec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<Fixed> for FixedVec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Fixed) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<Fixed> for FixedVec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Fixed) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for FixedVec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl FixedVec3 {
    /// Convert to f32 Vec3 (for rendering)
    #[inline]
    pub fn to_f32_vec3(self) -> crate::Vec3 {
        crate::Vec3::new(self.x.to_f32(), self.y.to_f32(), self.z.to_f32())
    }
}

impl fmt::Debug for FixedVec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixedVec3({:?}, {:?}, {:?})", self.x, self.y, self.z)
    }
}

/// 3D axis-aligned bounding box with fixed-point coordinates
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable, Serialize, Deserialize)]
pub struct FixedAABB {
    pub min: FixedVec3,
    pub max: FixedVec3,
}

impl FixedAABB {
    #[inline]
    pub const fn new(min: FixedVec3, max: FixedVec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn empty() -> Self {
        Self::new(
            FixedVec3::new(Fixed::MAX, Fixed::MAX, Fixed::MAX),
            FixedVec3::new(Fixed::MIN, Fixed::MIN, Fixed::MIN),
        )
    }

    #[inline]
    pub fn from_center_half_extents(center: FixedVec3, half_extents: FixedVec3) -> Self {
        Self::new(center - half_extents, center + half_extents)
    }

    #[inline]
    pub fn center(&self) -> FixedVec3 {
        (self.min + self.max) / Fixed::from_int(2)
    }

    #[inline]
    pub fn half_extents(&self) -> FixedVec3 {
        (self.max - self.min) / Fixed::from_int(2)
    }

    #[inline]
    pub fn size(&self) -> FixedVec3 {
        self.max - self.min
    }

    #[inline]
    pub fn contains_point(&self, point: FixedVec3) -> bool {
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
    pub fn expand(&mut self, point: FixedVec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    #[inline]
    pub fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    #[inline]
    pub fn volume(&self) -> Fixed {
        let size = self.size();
        size.x * size.y * size.z
    }

    #[inline]
    pub fn surface_area(&self) -> Fixed {
        let size = self.size();
        (size.x * size.y + size.y * size.z + size.z * size.x) * Fixed::from_int(2)
    }
}

impl fmt::Debug for FixedAABB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixedAABB(min: {:?}, max: {:?})", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_basic() {
        let a = Fixed::from_int(5);
        let b = Fixed::from_int(3);
        assert_eq!((a + b).to_bits(), Fixed::from_int(8).to_bits());
        assert_eq!((a - b).to_bits(), Fixed::from_int(2).to_bits());
        assert_eq!((a * b).to_bits(), Fixed::from_int(15).to_bits());
        assert_eq!((a / b).to_bits(), Fixed::from_int(1).to_bits());
    }

    #[test]
    fn test_fixed_from_f64() {
        let a = Fixed::from_f64(1.5);
        let b = Fixed::from_f64(2.5);
        let c = a + b;
        assert!((c.to_f64() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_fixed_vec3() {
        let a = FixedVec3::new(Fixed::from_int(1), Fixed::from_int(2), Fixed::from_int(3));
        let b = FixedVec3::new(Fixed::from_int(4), Fixed::from_int(5), Fixed::from_int(6));
        let dot = a.dot(b);
        assert_eq!(dot.to_bits(), Fixed::from_int(32).to_bits());
    }

    #[test]
    fn test_fixed_aabb() {
        let aabb = FixedAABB::from_center_half_extents(
            FixedVec3::zero(),
            FixedVec3::new(Fixed::from_int(1), Fixed::from_int(1), Fixed::from_int(1)),
        );
        assert!(aabb.contains_point(FixedVec3::zero()));
        assert!(aabb.contains_point(FixedVec3::new(Fixed::from_int(0), Fixed::ZERO, Fixed::ZERO)));
        assert!(!aabb.contains_point(FixedVec3::new(Fixed::from_int(2), Fixed::ZERO, Fixed::ZERO)));
    }
}