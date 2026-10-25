//! Densidad 3D del mundo grande (W1, fase 2): el frente de superficie con
//! salientes, la Bóveda y el lecho.
//!
//! [`raw_solid`] es la referencia de crudeza en cualquier coordenada del
//! mundo: una función pura de (semilla, posición) con enteros y Q16, sin
//! estado ni reloj. Por encima de [`MANTO_BOTTOM`] el terreno es sólido
//! hasta la superficie de la columna ([`crate::column::sample_column`]) y
//! una banda de ±[`BAND_HALF`] bloques alrededor de esa altura se decide
//! con un Fbm3 de densidad interpolado en una rejilla gruesa **global**
//! (X y Z cada 4 bloques, Y cada 8): como los orígenes de chunk son
//! múltiplos de 16, la rejilla es la misma para todos los chunks y no hay
//! costuras. Por debajo del manto mandan la Bóveda (módulo [`vault`]) y el
//! lecho, siempre sólido.
//!
//! Este módulo no decide el tipo de roca (eso es `crate::lithology`):
//! solo sólido o aire.

mod vault;

use grisu_noise::{Fbm3, ONE, lerp};

use crate::column::sample_column;
use vault::vault_solid;

/// Pendiente del frente de superficie: 0,5 en Q16 de densidad por bloque
/// de profundidad.
pub(crate) const SLOPE_Q16: i64 = ONE / 2;
/// Amplitud del ruido de salientes: equivale a ±8 bloques de densidad.
pub(crate) const NOISE_AMP_Q16: i64 = 4 * ONE;
/// Semiancho de la banda del frente en bloques: ±8 alrededor de la
/// superficie, el único sitio donde el ruido puede invertir la densidad.
pub(crate) const BAND_HALF: i64 = NOISE_AMP_Q16 / SLOPE_Q16;
/// Suelo del manto: por debajo empieza la Bóveda.
pub(crate) const MANTO_BOTTOM: i64 = -640;
/// Techo de las Raíces: ningún pilar de la Bóveda baja de aquí.
pub(crate) const ROOTS_TOP: i64 = -3072;
/// Lecho del mundo: siempre sólido.
pub(crate) const BEDROCK_Y: i64 = -4096;

/// Sal del Fbm3 de densidad del frente de superficie.
const SALT_DENSITY: u64 = 0x22D0_0000_0000_0001;

/// Zona vertical de un bloque respecto al frente de superficie de su
/// columna.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Zone {
    /// Siempre sólido: demasiado por debajo del frente.
    Solid,
    /// Siempre aire: demasiado por encima del frente.
    Air,
    /// Banda del frente: decide la densidad con ruido 3D.
    Band,
}

/// Zona del bloque `y` en una columna con superficie `surface_y`.
///
/// Prueba de la cota (el atajo analítico es exacto): la densidad es
/// `d = (h − y)·SLOPE + (ruido·AMP >> 16)` con `|ruido| ≤ ONE`, así que en
/// Solid (`y ≤ h − BAND_HALF`) se cumple `d ≥ BAND_HALF·SLOPE − AMP = 0` y
/// en Air (`y ≥ h + BAND_HALF`), `d ≤ −BAND_HALF·SLOPE + AMP = 0`. La
/// igualdad exigiría un ruido de exactamente ∓ONE, y eso no ocurre: el
/// ruido de gradiente queda estrictamente dentro de (−ONE, ONE) —toda
/// esquina de la retícula cuyo producto escalar alcanza ±2·ONE está en un
/// borde de celda donde su peso de interpolación es nulo, `lerp` no sale de
/// sus extremos y el fBm normalizado hereda la cota—. Con ruido real, en
/// Solid `d ≥ BAND_HALF·SLOPE − (AMP − 4) = 4 > 0` y en Air `d ≤ −4 < 0`:
/// decidir por la zona sin evaluar el Fbm3 da el mismo resultado que la
/// fórmula completa, y los chunks de cielo o de roca profunda ni tocan el
/// ruido 3D.
#[must_use]
pub(crate) fn terrain_zone(surface_y: i32, y: i64) -> Zone {
    let h = i64::from(surface_y);
    if y <= h - BAND_HALF {
        Zone::Solid
    } else if y >= h + BAND_HALF {
        Zone::Air
    } else {
        Zone::Band
    }
}

/// Esquina de la rejilla gruesa global del ruido de densidad: Fbm3 de
/// período base 32 bloques y 4 octavas, salida en `[-ONE, ONE]`. La
/// rejilla alinea X y Z cada 4 bloques y Y cada 8; los orígenes de chunk
/// (múltiplos de 16) comparten esquinas con los vecinos, así que dos
/// chunks contiguos interpolan la misma celda con los mismos números.
#[must_use]
pub(crate) fn noise_corner(seed: u64, cx: i64, cy: i64, cz: i64) -> i64 {
    Fbm3::new(seed, SALT_DENSITY, 5, 4).sample_block(cx, cy, cz)
}

/// Interpolación trilineal de las 8 esquinas de una celda gruesa.
///
/// `corners` está indexado `[ix][iy][iz]`; se interpola primero X (con
/// `fx`), luego Y (con `fy`) y por último Z (con `fz`). Como las esquinas
/// son globales y las fracciones salen de las coordenadas globales del
/// bloque, este es el único orden que deben seguir el generador por chunks
/// y [`raw_solid`] para clavar el mismo resultado.
#[must_use]
pub(crate) fn trilerp8(corners: &[[[i64; 2]; 2]; 2], fx: i64, fy: i64, fz: i64) -> i64 {
    let mut along_x = [[0i64; 2]; 2];
    for (iy, x_row) in along_x.iter_mut().enumerate() {
        for (iz, v) in x_row.iter_mut().enumerate() {
            *v = lerp(corners[0][iy][iz], corners[1][iy][iz], fx);
        }
    }
    let mut along_y = [0i64; 2];
    for (iz, v) in along_y.iter_mut().enumerate() {
        *v = lerp(along_x[0][iz], along_x[1][iz], fy);
    }
    lerp(along_y[0], along_y[1], fz)
}

/// Densidad del frente de superficie en Q16: positiva (sólido) por debajo
/// de la superficie, negativa (aire) por encima, con el ruido 3D añadiendo
/// salientes y huecos dentro de la banda. Es el único sitio donde vive la
/// fórmula, así que el generador por chunks y [`raw_solid`] hacen
/// exactamente la misma aritmética.
#[must_use]
pub(crate) fn band_density(
    corners: &[[[i64; 2]; 2]; 2],
    fx: i64,
    fy: i64,
    fz: i64,
    surface_y: i32,
    y: i64,
) -> i64 {
    (i64::from(surface_y) - y) * SLOPE_Q16 + ((trilerp8(corners, fx, fy, fz) * NOISE_AMP_Q16) >> 16)
}

/// Esquinas y fracciones de la celda gruesa global que contiene al bloque
/// `(x, y, z)`: base `x & !3`, `y & !7`, `z & !3` y fracciones Q16
/// `(x & 3) << 14`, `(y & 7) << 13`, `(z & 3) << 14` (un cuarto y un
/// octavo de bloque).
fn band_cell(seed: u64, x: i64, y: i64, z: i64) -> ([[[i64; 2]; 2]; 2], i64, i64, i64) {
    let (cx0, cy0, cz0) = (x & !3, y & !7, z & !3);
    let corners = std::array::from_fn(|ix| {
        std::array::from_fn(|iy| {
            std::array::from_fn(|iz| {
                noise_corner(
                    seed,
                    cx0 + 4 * (ix as i64),
                    cy0 + 8 * (iy as i64),
                    cz0 + 4 * (iz as i64),
                )
            })
        })
    });
    (corners, (x & 3) << 14, (y & 7) << 13, (z & 3) << 14)
}

/// Referencia de crudeza en cualquier coordenada del mundo: lecho siempre
/// sólido, Bóveda por debajo de [`MANTO_BOTTOM`] y frente de superficie
/// encima, con [`terrain_zone`] como atajo analítico. Precondición:
/// `|x|, |z| ≤ 7_499_999` y `|y| ≤ 8_192`; las conversiones a `i32` de la
/// fase de columna son seguras en ese rango.
#[must_use]
pub(crate) fn raw_solid(seed: u64, x: i64, y: i64, z: i64) -> bool {
    if y <= BEDROCK_Y {
        true
    } else if y <= MANTO_BOTTOM {
        vault_solid(seed, x, y, z)
    } else {
        let surface_y = sample_column(seed, x as i32, z as i32).surface_y;
        match terrain_zone(surface_y, y) {
            Zone::Solid => true,
            Zone::Air => false,
            Zone::Band => {
                let (corners, fx, fy, fz) = band_cell(seed, x, y, z);
                band_density(&corners, fx, fy, fz, surface_y, y) > 0
            }
        }
    }
}

/// Crudeza con soporte: sólido y con alguna de sus 6 vecinas también
/// sólida.
///
/// Prueba de ausencia de bloques 1×1×1 flotantes: si `supported_solid(p)`
/// es cierto por la vecina `n` (`raw_solid(n)` cierto), entonces
/// `supported_solid(n)` también es cierto, porque `p` es a su vez una
/// vecina de `n` con `raw_solid` cierto. Por inducción, todo bloque que
/// pasa el filtro tiene alguna vecina que también lo pasa: no queda
/// ningún sólido aislado.
#[must_use]
pub(crate) fn supported_solid(seed: u64, x: i64, y: i64, z: i64) -> bool {
    if !raw_solid(seed, x, y, z) {
        return false;
    }
    const OFFSETS: [(i64, i64, i64); 6] = [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];
    OFFSETS
        .iter()
        .any(|&(dx, dy, dz)| raw_solid(seed, x + dx, y + dy, z + dz))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Semilla fija de las pruebas del módulo.
    const SEED: u64 = 42;

    /// Las 6 direcciones de la vecindad 6-conectada.
    const OFFSETS: [(i64, i64, i64); 6] = [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];

    /// Xorshift64 como en `tests/bench.rs`: posiciones deterministas sin
    /// tocar el reloj ni la aleatoriedad del sistema.
    struct XorShift(u64);

    impl XorShift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }

    #[test]
    fn zone_bound_justifies_the_shortcut() {
        // Esquinas sintéticas en los extremos del contrato del ruido
        // (±ONE): los peores casos teóricos de la fórmula. En el borde de
        // la zona Solid, el ruido +ONE deja densidad holgada y el −ONE la
        // deja justo en 0; en el borde Air, al revés. Como el ruido real
        // queda estrictamente dentro de (−ONE, ONE), en Solid siempre
        // d > 0 y en Air siempre d < 0: el atajo de `terrain_zone` decide
        // lo mismo que la fórmula completa sin evaluar el Fbm3.
        let h = 128i32;
        let top = [[[ONE; 2]; 2]; 2];
        let bottom = [[[-ONE; 2]; 2]; 2];
        let solid_edge = i64::from(h) - BAND_HALF;
        let air_edge = i64::from(h) + BAND_HALF;
        assert!(band_density(&top, 0, 0, 0, h, solid_edge) > 0);
        assert!(band_density(&bottom, 0, 0, 0, h, solid_edge) >= 0);
        assert!(band_density(&top, 0, 0, 0, h, air_edge) <= 0);
        assert!(band_density(&bottom, 0, 0, 0, h, air_edge) < 0);
    }

    #[test]
    fn trilerp8_is_exact_and_respects_order() {
        // Potencias de dos: el resultado delata qué índice se interpoló.
        let c = [[[1i64, 2], [4, 8]], [[16, 32], [64, 128]]];
        assert_eq!(trilerp8(&c, 0, 0, 0), 1);
        assert_eq!(trilerp8(&c, ONE, 0, 0), 16);
        assert_eq!(trilerp8(&c, 0, ONE, 0), 4);
        assert_eq!(trilerp8(&c, 0, 0, ONE), 2);
        assert_eq!(trilerp8(&c, ONE, ONE, ONE), 128);
        // Punto medio en X: media exacta de las dos esquinas de la arista.
        assert_eq!(trilerp8(&c, ONE / 2, 0, 0), (1 + 16) / 2);
    }

    #[test]
    fn zone_matches_density_in_samples() {
        let mut rng = XorShift(0x1234_5678_9ABC_DEF0);
        let mut seen = [0usize; 3];
        for _ in 0..96 {
            let x = ((rng.next() % 4096) as i64) - 2048;
            let z = ((rng.next() % 4096) as i64) - 2048;
            let h = sample_column(SEED, x as i32, z as i32).surface_y;
            let y = i64::from(h) + ((rng.next() % 48) as i64) - 24;
            let (corners, fx, fy, fz) = band_cell(SEED, x, y, z);
            let d = band_density(&corners, fx, fy, fz, h, y);
            match terrain_zone(h, y) {
                Zone::Solid => {
                    seen[0] += 1;
                    assert!(d > 0, "sólido con d = {d} en ({x},{y},{z})");
                }
                Zone::Air => {
                    seen[1] += 1;
                    assert!(d < 0, "aire con d = {d} en ({x},{y},{z})");
                }
                Zone::Band => seen[2] += 1,
            }
        }
        // La muestra cubre las tres zonas.
        assert!(seen.iter().all(|&n| n > 0), "zonas vistas: {seen:?}");
    }

    #[test]
    fn raw_solid_is_deterministic_and_survives_extremes() {
        let edge = 7_499_999i64;
        let points = [
            (edge, 8_192, edge),
            (-edge, -8_192, -edge),
            (edge, -8_192, -edge),
            (-edge, 8_192, edge),
            (0, MANTO_BOTTOM, 0),
            (1, -1, 1),
            (-4, -8, -4),
            (3, -7, 3),
        ];
        for &(x, y, z) in &points {
            assert_eq!(
                raw_solid(SEED, x, y, z),
                raw_solid(SEED, x, y, z),
                "dos llamadas en ({x},{y},{z})"
            );
        }
        assert!(raw_solid(SEED, 0, BEDROCK_Y, 0));
        assert!(raw_solid(SEED, edge, BEDROCK_Y - 1, -edge));
    }

    /// Rejilla booleana de `raw_solid` de un cubo de lado `n` con esquina
    /// mínima `(x0, y0, z0)`: se evalúa UNA vez y la regla del soporte se
    /// aplica sobre la rejilla, sin reevaluar el ruido por bloque.
    fn raw_grid(seed: u64, x0: i64, y0: i64, z0: i64, n: i64) -> Vec<Vec<Vec<bool>>> {
        (0..n)
            .map(|ix| {
                (0..n)
                    .map(|iy| {
                        (0..n)
                            .map(|iz| raw_solid(seed, x0 + ix, y0 + iy, z0 + iz))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[test]
    fn every_supported_solid_has_a_supported_neighbour() {
        // Volúmenes de 24³: uno en el frente de superficie del origen y otro
        // en la Bóveda, a la altura de las islas. La rejilla lleva MARGEN 2:
        // la regla del soporte mira 1 bloque alrededor y la comprobación de
        // la propiedad vuelve a mirar otro más.
        const SIDE: i64 = 24;
        const MARGIN: i64 = 2;
        let half = SIDE / 2;
        let h = i64::from(sample_column(SEED, 0, 0).surface_y);
        for y0 in [h - half - MARGIN, -1600 - half - MARGIN] {
            let grid = raw_grid(SEED, -half - MARGIN, y0, -half - MARGIN, SIDE + 2 * MARGIN);
            let at = |v: i64, base: i64| usize::try_from(v - base).unwrap();
            let raw = |x: i64, y: i64, z: i64| {
                grid[at(x, -half - MARGIN)][at(y, y0)][at(z, -half - MARGIN)]
            };
            let supported = |x: i64, y: i64, z: i64| {
                raw(x, y, z)
                    && OFFSETS
                        .iter()
                        .any(|&(dx, dy, dz)| raw(x + dx, y + dy, z + dz))
            };
            for y in (y0 + MARGIN)..(y0 + MARGIN + SIDE) {
                for x in -half..half {
                    for z in -half..half {
                        if supported(x, y, z) {
                            assert!(
                                OFFSETS.iter().any(|&(dx, dy, dz)| supported(
                                    x + dx,
                                    y + dy,
                                    z + dz
                                )),
                                "({x},{y},{z}) se queda aislado"
                            );
                        }
                    }
                }
            }
        }
    }
}
