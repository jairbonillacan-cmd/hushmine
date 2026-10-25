//! Curvas de relieve por tramos de la fase de columna (W1).
//!
//! Combinan continentalidad, erosión, picos y ríos para dar la altura de
//! superficie en Q16. Todo con `lerp` de `i64`: sin `f32`, determinista en
//! cualquier plataforma.

use grisu_noise::{ONE, lerp};

use super::{MAX_SURFACE, MIN_SURFACE, WATER_LEVEL};

/// Puntos de control de la altura base: (continentalidad, altura) en Q16.
// Se moverá a data/worldgen/relieve.ron
const BASE_HEIGHT_CURVE: [(i64, i64); 5] = [
    (-ONE, -48 * ONE),
    (-ONE / 2, -36 * ONE),
    (0, 24 * ONE),
    (ONE / 2, 200 * ONE),
    (ONE, 480 * ONE),
];

/// Escala del aporte de picos según la erosión: escarpado a suave.
// Se moverá a data/worldgen/relieve.ron
const PEAKS_SCALE_CURVE: [(i64, i64); 2] = [(-ONE, 320 * ONE), (ONE, 40 * ONE)];

/// Umbral de cresta por encima del cual hay río (0,88 en Q16).
const RIVER_START: i64 = (88 * ONE) / 100;

/// Evalúa una curva por tramos (x, y) en Q16 con `lerp` de `i64`.
///
/// Los puntos están ordenados por x creciente; fuera del dominio se satura
/// al extremo. En los nodos el resultado es exacto.
fn eval_curve<const N: usize>(knots: &[(i64, i64); N], v: i64) -> i64 {
    if v <= knots[0].0 {
        return knots[0].1;
    }
    let mut prev = knots[0];
    for &(x1, y1) in &knots[1..] {
        if v <= x1 {
            let (x0, y0) = prev;
            return lerp(y0, y1, ((v - x0) << 16) / (x1 - x0));
        }
        prev = (x1, y1);
    }
    knots[N - 1].1
}

/// Altura de superficie en Q16 para los campos de un punto de la rejilla:
/// curva base por continentalidad, aporte de picos según la erosión,
/// hundimiento del cauce de los ríos y recorte final al dominio.
pub(super) fn surface_height_q16(cont: i64, ero: i64, peaks: i64, river: i64) -> i64 {
    let scale = eval_curve(&PEAKS_SCALE_CURVE, ero);
    let mut h = eval_curve(&BASE_HEIGHT_CURVE, cont) + ((peaks * scale) >> 16);
    let water = i64::from(WATER_LEVEL) << 16;
    if river > RIVER_START {
        // Hundir hacia el nivel del agua sin cruzarlo. El factor lleva la
        // escala Q16: 0 en el umbral y ONE en la cresta máxima.
        let factor = ((river - RIVER_START) << 16) / (ONE - RIVER_START);
        let before = h;
        h -= (factor * (h - water)) >> 16;
        if (before > water && h < water) || (before < water && h > water) {
            h = water;
        }
    }
    h.clamp(i64::from(MIN_SURFACE) << 16, i64::from(MAX_SURFACE) << 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_pass_through_their_knots() {
        for &(x, y) in &BASE_HEIGHT_CURVE {
            assert_eq!(eval_curve(&BASE_HEIGHT_CURVE, x), y);
        }
        for &(x, y) in &PEAKS_SCALE_CURVE {
            assert_eq!(eval_curve(&PEAKS_SCALE_CURVE, x), y);
        }
        // Fuera del dominio saturan y el punto medio de la escala es exacto.
        assert_eq!(eval_curve(&BASE_HEIGHT_CURVE, ONE + 1), 480 * ONE);
        assert_eq!(eval_curve(&PEAKS_SCALE_CURVE, 0), 180 * ONE);
    }

    #[test]
    fn river_sinks_to_water_and_clamp_bounds() {
        // Sin río la altura queda como la curva base; con cresta máxima se
        // hunde justo hasta el nivel del agua, sin cruzarlo.
        assert_eq!(surface_height_q16(0, ONE, 0, 0), 24 * ONE);
        assert_eq!(
            surface_height_q16(0, ONE, 0, ONE),
            i64::from(WATER_LEVEL) << 16
        );
        // Suelo oceánico sin picos y recorte del techo con picos extremos.
        assert_eq!(surface_height_q16(-ONE, ONE, 0, 0), -48 * ONE);
        assert_eq!(
            surface_height_q16(ONE, -ONE, ONE, 0),
            i64::from(MAX_SURFACE) << 16
        );
    }
}
