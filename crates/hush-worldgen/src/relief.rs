//! Relieve de la superficie del mundo de prueba.
//!
//! Alturas enteras en una rejilla de [`CELL_SHIFT`] bits de lado, elegidas
//! por hash, e interpolación bilineal en punto fijo Q16 (sin `f32`). La
//! rejilla es global —está indexada por coordenadas de bloque del mundo, no
//! del chunk—, así que el relieve es continuo a través de las costuras y la
//! altura de una columna no depende de qué chunk la contiene.

use crate::hash::hash2d;

/// Unidad de punto fijo Q16: 65 536 unidades por bloque.
const Q16: i64 = 1 << 16;
/// Máscara de la posición dentro de una celda de la rejilla.
const CELL_MASK: i64 = (1 << CELL_SHIFT) - 1;

/// Lado de las celdas del relieve, en bloques (2ᵏ: la celda es `x >> k` y
/// la fracción `(x & (2ᵏ − 1)) << (16 − k)` es exacta en Q16).
pub const CELL_SHIFT: i32 = 3;
/// Amplitud del relieve: la altura de las esquinas vive en
/// `−AMPLITUDE..=AMPLITUDE`, de modo que la superficie ronda `y = 0`.
pub const AMPLITUDE: i32 = 6;

/// Sal de la capa de relieve.
const RELIEF_SALT: u64 = 0x5EED_0000_0000_0001;
/// Valores posibles de una esquina: `2 · AMPLITUDE + 1`.
const CORNER_SPAN: u64 = 13;
/// Sal del grosor de tierra de cada columna.
const DIRT_SALT: u64 = 0x0127_0000_0000_0002;
/// Sal de la profundidad a la que empieza la roca dura.
const HARD_SALT: u64 = 0x0A2D_0000_0000_0003;

/// Altura de la esquina `(gx, gz)` de la rejilla, en bloques.
#[inline]
fn corner_height(seed: u64, gx: i64, gz: i64) -> i32 {
    // `CORNER_SPAN` es 13 y `AMPLITUDE` es 6: el módulo cabe de sobra en i32.
    (hash2d(gx, gz, seed ^ RELIEF_SALT) % CORNER_SPAN) as i32 - AMPLITUDE
}

/// Altura del terreno en la columna `(x, z)`, en bloques: el bloque de
/// superficie es `y = surface_height` y por debajo es todo sólido.
///
/// Interpolación bilineal de las cuatro esquinas de la celda, en Q16 con
/// intermedios `i64`. El mayor término del producto es
/// `AMPLITUDE · 2¹⁶ · 2¹⁶ · 2¹⁶`, unas 6 800 veces por debajo del límite de
/// `i64`, así que no hace falta aritmética saturada en el rango del mundo.
pub fn surface_height(seed: u64, x: i32, z: i32) -> i32 {
    surface_height_at(seed, i64::from(x), i64::from(z))
}

/// Lo mismo que [`surface_height`], con coordenadas `i64` para que el
/// generador no tenga que acotar la posición del chunk.
pub(crate) fn surface_height_at(seed: u64, x: i64, z: i64) -> i32 {
    let gx = x >> CELL_SHIFT;
    let gz = z >> CELL_SHIFT;
    // Fracción dentro de la celda en Q16: exacta porque el lado es 2^k.
    let fx = (x & CELL_MASK) << (16 - CELL_SHIFT);
    let fz = (z & CELL_MASK) << (16 - CELL_SHIFT);
    let c00 = i64::from(corner_height(seed, gx, gz)) << 16;
    let c10 = i64::from(corner_height(seed, gx + 1, gz)) << 16;
    let c01 = i64::from(corner_height(seed, gx, gz + 1)) << 16;
    let c11 = i64::from(corner_height(seed, gx + 1, gz + 1)) << 16;
    let mix = c00 * (Q16 - fx) * (Q16 - fz)
        + c10 * fx * (Q16 - fz)
        + c01 * (Q16 - fx) * fz
        + c11 * fx * fz;
    // `mix` queda en Q16 (las esquinas están en Q16 y los pesos suman 2³²):
    // redondeo al bloque más cercano; el empate va hacia arriba.
    ((mix + (1 << 47)) >> 48) as i32
}

/// Bloques de tierra bajo la superficie de la columna `(x, z)`: 3 a 5.
#[inline]
pub(crate) fn dirt_depth(seed: u64, x: i64, z: i64) -> i64 {
    3 + (hash2d(x, z, seed ^ DIRT_SALT) % 3) as i64
}

/// Profundidad bajo la superficie a la que empieza la roca dura: 20 a 24.
#[inline]
pub(crate) fn hard_rock_depth(seed: u64, x: i64, z: i64) -> i64 {
    20 + (hash2d(x, z, seed ^ HARD_SALT) % 5) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_stays_within_the_amplitude() {
        for seed in [0, 1, 42, u64::MAX] {
            for z in -33..33 {
                for x in -33..33 {
                    let h = surface_height(seed, x, z);
                    assert!((-AMPLITUDE..=AMPLITUDE).contains(&h), "({x}, {z}) = {h}");
                }
            }
        }
    }

    /// En las esquinas de la rejilla la interpolación no altera el valor:
    /// la fracción es cero y el redondeo es exacto.
    #[test]
    fn interpolation_hits_the_corners_exactly() {
        for gz in -5..5 {
            for gx in -5..5 {
                let (x, z) = (gx << CELL_SHIFT, gz << CELL_SHIFT);
                let expected = corner_height(42, i64::from(gx), i64::from(gz));
                assert_eq!(surface_height(42, x, z), expected, "esquina ({gx}, {gz})");
            }
        }
    }

    /// El relieve es suave: entre columnas vecinas la altura no salta más
    /// que lo que permite la pendiente de la rejilla (12 bloques por celda
    /// de 8, redondeados a 2 por paso).
    #[test]
    fn neighbouring_columns_differ_by_at_most_two() {
        for seed in [0, 7, 42] {
            for z in -70..70 {
                let mut prev = surface_height(seed, -70, z);
                for x in -69..70 {
                    let h = surface_height(seed, x, z);
                    assert!(
                        (h - prev).abs() <= 2,
                        "salto de {prev} a {h} en ({x}, {z}), semilla {seed}"
                    );
                    prev = h;
                }
            }
            for x in -70..70 {
                let mut prev = surface_height(seed, x, -70);
                for z in -69..70 {
                    let h = surface_height(seed, x, z);
                    assert!(
                        (h - prev).abs() <= 2,
                        "salto de {prev} a {h} en ({x}, {z}), semilla {seed}"
                    );
                    prev = h;
                }
            }
        }
    }

    #[test]
    fn depths_stay_in_their_ranges() {
        for z in -20..20 {
            for x in -20..20 {
                assert!((3..=5).contains(&dirt_depth(42, i64::from(x), i64::from(z))));
                assert!((20..=24).contains(&hard_rock_depth(42, i64::from(x), i64::from(z))));
            }
        }
    }
}
