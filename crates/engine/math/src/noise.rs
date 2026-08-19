//! Deterministic noise functions for procedural generation

use crate::{Vec2, Vec3};
use std::hash::{Hash, Hasher};

// Simple hash function for deterministic noise
fn hash_u64(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;
    x
}

fn hash_i32(x: i32) -> u64 {
    hash_u64(x as u64)
}

fn hash_2d(x: i32, y: i32) -> u64 {
    hash_u64(((x as u64) << 32) ^ (y as u64 & 0xFFFFFFFF))
}

fn hash_3d(x: i32, y: i32, z: i32) -> u64 {
    let mut h = hash_u64(x as u64);
    h = hash_u64(h ^ (y as u64).wrapping_mul(0x9e3779b97f4a7c15));
    h = hash_u64(h ^ (z as u64).wrapping_mul(0x9e3779b97f4a7c15));
    h
}

/// Convert hash to f32 in [0, 1)
fn hash_to_f32(h: u64) -> f32 {
    (h >> 11) as f32 * (1.0 / (1u64 << 53) as f32)
}

/// Convert hash to f32 in [-1, 1)
fn hash_to_f32_signed(h: u64) -> f32 {
    hash_to_f32(h) * 2.0 - 1.0
}

/// Gradient noise (Perlin-style)
pub fn perlin_2d(p: Vec2, seed: u64) -> f32 {
    let x0 = p.x.floor() as i32;
    let y0 = p.y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let fx = p.x - x0 as f32;
    let fy = p.y - y0 as f32;

    // Fade function
    let u = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
    let v = fy * fy * fy * (fy * (fy * 6.0 - 15.0) + 10.0);

    // Gradient vectors (predefined set)
    let gradients = [
        Vec2::new(1.0, 0.0), Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(0.0, -1.0),
        Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0), Vec2::new(1.0, -1.0), Vec2::new(-1.0, -1.0),
    ];

    let g00 = gradients[(hash_2d(x0, y0) ^ seed) as usize % 8];
    let g10 = gradients[(hash_2d(x1, y0) ^ seed) as usize % 8];
    let g01 = gradients[(hash_2d(x0, y1) ^ seed) as usize % 8];
    let g11 = gradients[(hash_2d(x1, y1) ^ seed) as usize % 8];

    let n00 = g00.dot(Vec2::new(fx, fy));
    let n10 = g10.dot(Vec2::new(fx - 1.0, fy));
    let n01 = g01.dot(Vec2::new(fx, fy - 1.0));
    let n11 = g11.dot(Vec2::new(fx - 1.0, fy - 1.0));

    let nx0 = n00 + u * (n10 - n00);
    let nx1 = n01 + u * (n11 - n01);
    nx0 + v * (nx1 - nx0)
}

/// 3D Perlin noise
pub fn perlin_3d(p: Vec3, seed: u64) -> f32 {
    let x0 = p.x.floor() as i32;
    let y0 = p.y.floor() as i32;
    let z0 = p.z.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let z1 = z0 + 1;

    let fx = p.x - x0 as f32;
    let fy = p.y - y0 as f32;
    let fz = p.z - z0 as f32;

    let u = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
    let v = fy * fy * fy * (fy * (fy * 6.0 - 15.0) + 10.0);
    let w = fz * fz * fz * (fz * (fz * 6.0 - 15.0) + 10.0);

    let gradients = [
        Vec3::new(1.0, 1.0, 0.0), Vec3::new(-1.0, 1.0, 0.0), Vec3::new(1.0, -1.0, 0.0), Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, 0.0, 1.0), Vec3::new(-1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, -1.0), Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, -1.0, 1.0), Vec3::new(0.0, 1.0, -1.0), Vec3::new(0.0, -1.0, -1.0),
    ];

    let g = |x: i32, y: i32, z: i32| -> Vec3 {
        gradients[(hash_3d(x, y, z) ^ seed) as usize % 12]
    };

    let g000 = g(x0, y0, z0);
    let g100 = g(x1, y0, z0);
    let g010 = g(x0, y1, z0);
    let g110 = g(x1, y1, z0);
    let g001 = g(x0, y0, z1);
    let g101 = g(x1, y0, z1);
    let g011 = g(x0, y1, z1);
    let g111 = g(x1, y1, z1);

    let n = |g: Vec3, dx: f32, dy: f32, dz: f32| -> f32 {
        g.x * dx + g.y * dy + g.z * dz
    };

    let n000 = n(g000, fx, fy, fz);
    let n100 = n(g100, fx - 1.0, fy, fz);
    let n010 = n(g010, fx, fy - 1.0, fz);
    let n110 = n(g110, fx - 1.0, fy - 1.0, fz);
    let n001 = n(g001, fx, fy, fz - 1.0);
    let n101 = n(g101, fx - 1.0, fy, fz - 1.0);
    let n011 = n(g011, fx, fy - 1.0, fz - 1.0);
    let n111 = n(g111, fx - 1.0, fy - 1.0, fz - 1.0);

    let nx00 = n000 + u * (n100 - n000);
    let nx10 = n010 + u * (n110 - n010);
    let nx01 = n001 + u * (n101 - n001);
    let nx11 = n011 + u * (n111 - n011);

    let nxy0 = nx00 + v * (nx10 - nx00);
    let nxy1 = nx01 + v * (nx11 - nx01);

    nxy0 + w * (nxy1 - nxy0)
}

/// Simplex noise (2D) - faster than Perlin, fewer artifacts
pub fn simplex_2d(p: Vec2, seed: u64) -> f32 {
    // Skewing factors
    let f2 = 0.5 * (3.0_f32.sqrt() - 1.0);
    let g2 = (3.0 - 3.0_f32.sqrt()) / 6.0;

    let s = (p.x + p.y) * f2;
    let i = (p.x + s).floor() as i32;
    let j = (p.y + s).floor() as i32;

    let t = (i + j) as f32 * g2;
    let x0 = i as f32 - t;
    let y0 = j as f32 - t;

    let x0 = p.x - x0;
    let y0 = p.y - y0;

    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

    let x1 = x0 - i1 as f32 + g2;
    let y1 = y0 - j1 as f32 + g2;
    let x2 = x0 - 1.0 + 2.0 * g2;
    let y2 = y0 - 1.0 + 2.0 * g2;

    let ii = i & 255;
    let jj = j & 255;

    let grad = |x: i32, y: i32| -> Vec2 {
        let h = (hash_2d(x, y) ^ seed) & 7;
        match h {
            0 => Vec2::new(1.0, 0.0),
            1 => Vec2::new(-1.0, 0.0),
            2 => Vec2::new(0.0, 1.0),
            3 => Vec2::new(0.0, -1.0),
            4 => Vec2::new(1.0, 1.0),
            5 => Vec2::new(-1.0, 1.0),
            6 => Vec2::new(1.0, -1.0),
            _ => Vec2::new(-1.0, -1.0),
        }
    };

    let g0 = grad(ii, jj);
    let g1 = grad(ii + i1, jj + j1);
    let g2 = grad(ii + 1, jj + 1);

    let t0 = 0.5 - x0 * x0 - y0 * y0;
    let n0 = if t0 < 0.0 { 0.0 } else { t0 * t0 * t0 * t0 * g0.dot(Vec2::new(x0, y0)) };

    let t1 = 0.5 - x1 * x1 - y1 * y1;
    let n1 = if t1 < 0.0 { 0.0 } else { t1 * t1 * t1 * t1 * g1.dot(Vec2::new(x1, y1)) };

    let t2 = 0.5 - x2 * x2 - y2 * y2;
    let n2 = if t2 < 0.0 { 0.0 } else { t2 * t2 * t2 * t2 * g2.dot(Vec2::new(x2, y2)) };

    70.0 * (n0 + n1 + n2)
}

/// Simplex noise (3D)
pub fn simplex_3d(p: Vec3, seed: u64) -> f32 {
    // Simplified 3D simplex - for full implementation use a proper library
    // This is a placeholder that uses Perlin
    perlin_3d(p, seed)
}

/// Fractal Brownian Motion (fBm)
pub fn fbm_2d(p: Vec2, octaves: u32, persistence: f32, lacunarity: f32, seed: u64) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += amplitude * perlin_2d(p * frequency, seed);
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

/// 3D fBm
pub fn fbm_3d(p: Vec3, octaves: u32, persistence: f32, lacunarity: f32, seed: u64) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += amplitude * perlin_3d(p * frequency, seed);
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

/// Domain warping
pub fn domain_warp_2d(p: Vec2, warp_strength: f32, octaves: u32, seed: u64) -> Vec2 {
    let mut q = Vec2::new(
        fbm_2d(p + Vec2::new(0.0, 0.0), octaves, 0.5, 2.0, seed),
        fbm_2d(p + Vec2::new(5.2, 1.3), octaves, 0.5, 2.0, seed),
    );
    let mut r = Vec2::new(
        fbm_2d(p + 4.0 * q + Vec2::new(1.7, 9.2), octaves, 0.5, 2.0, seed),
        fbm_2d(p + 4.0 * q + Vec2::new(8.3, 2.8), octaves, 0.5, 2.0, seed),
    );
    p + warp_strength * r
}

/// Cellular noise (Worley)
pub fn cellular_2d(p: Vec2, seed: u64) -> (f32, f32) {
    let x0 = p.x.floor() as i32;
    let y0 = p.y.floor() as i32;

    let mut min_dist = f32::MAX;
    let mut second_min_dist = f32::MAX;

    for dy in -1..=1 {
        for dx in -1..=1 {
            let cell_x = x0 + dx;
            let cell_y = y0 + dy;

            // Random point in cell
            let h = hash_2d(cell_x, cell_y) ^ seed;
            let rx = hash_to_f32(h);
            let ry = hash_to_f32(h >> 16);

            let point_x = cell_x as f32 + rx;
            let point_y = cell_y as f32 + ry;

            let dx = p.x - point_x;
            let dy = p.y - point_y;
            let dist = dx * dx + dy * dy;

            if dist < min_dist {
                second_min_dist = min_dist;
                min_dist = dist;
            } else if dist < second_min_dist {
                second_min_dist = dist;
            }
        }
    }

    (min_dist.sqrt(), second_min_dist.sqrt())
}

/// Ridged multifractal noise
pub fn ridged_multifractal_3d(p: Vec3, octaves: u32, persistence: f32, lacunarity: f32, seed: u64) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;

    for _ in 0..octaves {
        let n = perlin_3d(p * frequency, seed);
        let r = 1.0 - n.abs(); // Ridge
        value += amplitude * r * r; // Square for sharp ridges
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_2d_deterministic() {
        let p = Vec2::new(1.5, 2.5);
        let v1 = perlin_2d(p, 42);
        let v2 = perlin_2d(p, 42);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_perlin_3d_deterministic() {
        let p = Vec3::new(1.5, 2.5, 3.5);
        let v1 = perlin_3d(p, 42);
        let v2 = perlin_3d(p, 42);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_fbm() {
        let p = Vec2::new(10.0, 20.0);
        let v = fbm_2d(p, 4, 0.5, 2.0, 123);
        assert!(v >= -1.0 && v <= 1.0);
    }
}