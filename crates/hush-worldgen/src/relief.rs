//! Relieve de la superficie del mundo grande (capa fina de compatibilidad
//! del W0).
//!
//! [`surface_height`] expone la altura de superficie de la columna del
//! mundo grande; los cinco parámetros climáticos y el bioma viven en
//! [`sample_column`](crate::column::sample_column). [`AMPLITUDE`] ya no es
//! la amplitud de un ruido acotado sino la cota absoluta de la superficie
//! ([`MAX_SURFACE`](crate::column::MAX_SURFACE)), y [`CELL_SHIFT`] es el
//! lado de las celdas de la rejilla de columnas
//! ([`GRID_SHIFT`](crate::column::GRID_SHIFT)), no el paso del
//! interpolador viejo del W0.

use crate::column::{self, sample_column};

/// Cota absoluta de la altura de la superficie, en bloques: la altura de
/// cada columna queda dentro de `−AMPLITUDE..=AMPLITUDE` (la cota baja
/// real es `column::MIN_SURFACE`, −64, pero esta constante mantiene la
/// promesa simétrica que el W0 hacía con su relieve de prueba).
pub const AMPLITUDE: i32 = column::MAX_SURFACE;

/// Bits del lado de las celdas de la rejilla de columnas: 4 bloques.
pub const CELL_SHIFT: i32 = column::GRID_SHIFT;

/// Altura del terreno en la columna `(x, z)`, en bloques: el bloque de
/// superficie de la columna del mundo grande (función pura de la semilla
/// y las coordenadas, continua a través de las costuras de chunk).
#[must_use]
pub fn surface_height(seed: u64, x: i32, z: i32) -> i32 {
    sample_column(seed, x, z).surface_y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_stays_within_the_bound() {
        for seed in [0, 1, 42, u64::MAX] {
            for z in (-33..33).step_by(3) {
                for x in (-33..33).step_by(3) {
                    let h = surface_height(seed, x, z);
                    assert!((-AMPLITUDE..=AMPLITUDE).contains(&h), "({x},{z}) = {h}");
                }
            }
        }
    }

    #[test]
    fn survives_i32_extremes() {
        // El generador solo se usa dentro de ±7,5 M, pero la rejilla y la
        // interpolación no deben desbordarse antes.
        for (x, z) in [
            (i32::MIN, i32::MIN),
            (i32::MAX, i32::MAX),
            (i32::MIN, i32::MAX),
        ] {
            let h = surface_height(42, x, z);
            assert!((-AMPLITUDE..=AMPLITUDE).contains(&h), "({x},{z}) = {h}");
        }
    }

    #[test]
    fn is_deterministic() {
        for (x, z) in [(0, 0), (-5, 9), (7_499_999, -7_499_999)] {
            assert_eq!(
                surface_height(42, x, z),
                surface_height(42, x, z),
                "({x},{z})"
            );
        }
    }
}
