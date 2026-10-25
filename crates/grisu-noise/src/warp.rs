//! Deformación de dominio.
//!
//! Se desplaza la entrada con otro ruido antes de muestrear: formas
//! orgánicas sin coste de octavas extra. Los fBm de deformación son del
//! llamador, cada uno con su sal.

use crate::fbm::{Fbm2, Fbm3};

/// Deformación de dominio 2D.
///
/// Devuelve la entrada `(x, y)` en Q16 desplazada por dos fBm escalados
/// por `strength` en Q16: `65 536` desplaza hasta un bloque por unidad de
/// ruido. Las sumas son `wrapping` para que las coordenadas extremas no
/// entren en pánico.
#[must_use]
pub fn warp2(x: i64, y: i64, warp_x: &Fbm2, warp_y: &Fbm2, strength: i64) -> (i64, i64) {
    let dx = (warp_x.sample_q16(x, y) * strength) >> 16;
    let dy = (warp_y.sample_q16(x, y) * strength) >> 16;
    (x.wrapping_add(dx), y.wrapping_add(dy))
}

/// Deformación de dominio 3D, como [`warp2`] con un fBm por eje.
#[must_use]
pub fn warp3(
    x: i64,
    y: i64,
    z: i64,
    warp_x: &Fbm3,
    warp_y: &Fbm3,
    warp_z: &Fbm3,
    strength: i64,
) -> (i64, i64, i64) {
    let dx = (warp_x.sample_q16(x, y, z) * strength) >> 16;
    let dy = (warp_y.sample_q16(x, y, z) * strength) >> 16;
    let dz = (warp_z.sample_q16(x, y, z) * strength) >> 16;
    (x.wrapping_add(dx), y.wrapping_add(dy), z.wrapping_add(dz))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::ONE;

    #[test]
    fn fuerza_cero_no_desplaza() {
        let warp_x = Fbm2::new(42, 1, 5, 2);
        let warp_y = Fbm2::new(42, 2, 5, 2);
        assert_eq!(
            warp2(100 << 16, -7 << 16, &warp_x, &warp_y, 0),
            (100 << 16, -7 << 16)
        );
    }

    #[test]
    fn el_desplazamiento_esta_acotado_por_la_fuerza() {
        let warp_x = Fbm2::new(42, 1, 5, 2);
        let warp_y = Fbm2::new(42, 2, 5, 2);
        let strength = 8 * ONE; // hasta 8 bloques
        for i in 0..64i64 {
            let x = (i * 7) << 16;
            let y = (i * -3) << 16;
            let (dx, dy) = warp2(x, y, &warp_x, &warp_y, strength);
            assert!((dx - x).abs() <= strength, "desplazamiento X sin acotar");
            assert!((dy - y).abs() <= strength, "desplazamiento Y sin acotar");
        }
    }

    #[test]
    fn es_continua_y_determinista() {
        let warp_x = Fbm3::new(42, 1, 5, 3);
        let warp_y = Fbm3::new(42, 2, 5, 3);
        let warp_z = Fbm3::new(42, 3, 5, 3);
        let strength = 2 * ONE;
        let at = |x: i64| warp3(x, 5 << 16, -9 << 16, &warp_x, &warp_y, &warp_z, strength);
        let delta = |a: (i64, i64, i64), b: (i64, i64, i64)| {
            (b.0 - a.0)
                .abs()
                .max((b.1 - a.1).abs())
                .max((b.2 - a.2).abs())
        };
        for i in 0..256i64 {
            let p = (i * 3) << 16;
            let base = at(p);
            assert_eq!(at(p), base, "no es determinista");
            // Un paso de un bloque mueve la salida acotado por la fuerza.
            let big = delta(base, at(p + ONE));
            assert!(big <= strength, "salto de {big} superior a la fuerza");
            // Un paso 16 veces menor mueve la salida mucho menos: si hubiera
            // una discontinuidad, también saltaría en el paso pequeño.
            let small = delta(base, at(p + ONE / 16));
            assert!(
                small <= strength / 8,
                "salto de {small} en 1/16 de bloque: discontinuidad"
            );
        }
    }

    #[test]
    fn coordenadas_extremas_no_desbordan() {
        let warp_x = Fbm3::new(1, 1, 5, 2);
        let warp_y = Fbm3::new(1, 2, 5, 2);
        let warp_z = Fbm3::new(1, 3, 5, 2);
        let _ = warp3(i64::MIN, i64::MAX, i64::MIN, &warp_x, &warp_y, &warp_z, ONE);
    }
}
