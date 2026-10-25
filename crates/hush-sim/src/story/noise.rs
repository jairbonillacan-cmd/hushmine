//! Ruido de valor 3D determinista para la generación del mundo historia.
//!
//! Todo con enteros: retícula hash + interpolación trilineal suavizada.
//! El resultado está en Q7 con signo (`-127..=127`).

use crate::Rng;

/// Ruido de valor en el punto `(x, y, z)`, en unidades de bloque, con
/// celdas de retícula de `scale` bloques. Devuelve `-127..=127`.
pub fn value_noise(seed: u64, x: i32, y: i32, z: i32, scale: i32) -> i32 {
    let scale = scale.max(1);
    let fx = x.div_euclid(scale);
    let fy = y.div_euclid(scale);
    let fz = z.div_euclid(scale);
    let lx = smooth(x.rem_euclid(scale) * 256 / scale);
    let ly = smooth(y.rem_euclid(scale) * 256 / scale);
    let lz = smooth(z.rem_euclid(scale) * 256 / scale);
    let corner = |dx, dy, dz| lattice(seed, fx + dx, fy + dy, fz + dz);
    // Interpolación trilineal en Q7.
    let c00 = lerp(corner(0, 0, 0), corner(1, 0, 0), lx);
    let c10 = lerp(corner(0, 1, 0), corner(1, 1, 0), lx);
    let c01 = lerp(corner(0, 0, 1), corner(1, 0, 1), lx);
    let c11 = lerp(corner(0, 1, 1), corner(1, 1, 1), lx);
    let c0 = lerp(c00, c10, ly);
    let c1 = lerp(c01, c11, ly);
    lerp(c0, c1, lz)
}

/// Dos ruidos independientes: `value_noise(seed ^ salt, ...)`.
pub fn value_noise2(seed: u64, salt: u64, x: i32, y: i32, z: i32, scale: i32) -> i32 {
    value_noise(
        seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        x,
        y,
        z,
        scale,
    )
}

fn lerp(a: i32, b: i32, t: i32) -> i32 {
    a + (b - a) * t / 256
}

/// Suavizado hermite en Q8 (0..=256).
fn smooth(t: i32) -> i32 {
    t * t * (768 - 2 * t) / 65536
}

fn lattice(seed: u64, x: i32, y: i32, z: i32) -> i32 {
    let mut rng = Rng::new(seed ^ mix(x, y, z));
    (rng.below(255) as i32) - 127
}

fn mix(x: i32, y: i32, z: i32) -> u64 {
    let mut h = (u64::from(x as u32) << 20) ^ (u64::from(y as u32) << 10) ^ u64::from(z as u32);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D_7F4A_7C15);
    h ^ (h >> 13)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_bounded() {
        for &(x, y, z) in &[(0, 0, 0), (13, -7, 42), (-100, 200, -300), (95, 95, 95)] {
            let a = value_noise(7, x, y, z, 24);
            assert_eq!(a, value_noise(7, x, y, z, 24));
            assert!((-127..=127).contains(&a));
        }
    }

    #[test]
    fn varies_smoothly_and_covers_range() {
        let mut values = Vec::new();
        for i in 0..64 {
            values.push(value_noise(3, i * 3, 40, 40, 8));
        }
        let max = values.iter().max().copied().unwrap();
        let min = values.iter().min().copied().unwrap();
        // Con retícula de 8 bloques y 192 bloques de muestreo hay variedad.
        assert!(max - min > 32, "poca variedad: {min}..{max}");
        // Los valores contiguos no dan saltos bruscos.
        for pair in values.windows(2) {
            assert!((pair[0] - pair[1]).abs() <= 110, "salto brusco: {pair:?}");
        }
    }

    #[test]
    fn different_seeds_give_different_noise() {
        let a = value_noise(1, 10, 10, 10, 16);
        let b = value_noise(2, 10, 10, 10, 16);
        assert_ne!(a, b);
    }
}
