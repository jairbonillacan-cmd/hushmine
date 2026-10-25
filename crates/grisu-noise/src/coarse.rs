//! Muestreo grueso con interpolación trilineal en punto fijo.
//!
//! El ruido 3D de densidad se evalúa solo en una rejilla de 4 × 8 × 4
//! bloques por chunk (5 × 3 × 5 = 75 esquinas) y el resto de los 4 096
//! bloques se rellena interpolando: unas 16 veces más barato y apenas se
//! nota.

use crate::fixed::{ONE, lerp};

/// Esquinas por eje: 16/4 + 1 en X y Z, 16/8 + 1 en Y.
const CORNERS_XZ: usize = 5;
const CORNERS_Y: usize = 3;

/// Rejilla gruesa de un chunk de 16³.
#[derive(Clone, Debug)]
pub struct Coarse3 {
    /// Valores Q16 de las esquinas: `[x][y][z]`.
    corners: [[[i64; CORNERS_XZ]; CORNERS_Y]; CORNERS_XZ],
}

/// Celda del eje y fracción Q16 dentro de ella.
///
/// `v` debe estar en `[0, 16 · ONE]`; `spacing_log2` es el log2 de la
/// separación de la rejilla en bloques (2 para X y Z, 3 para Y). El extremo
/// `16 · ONE` cae en la última celda con `t = ONE`.
fn axis_cell(v: i64, spacing_log2: u32) -> (usize, i64) {
    let last_cell = (16 >> spacing_log2) - 1;
    let shift = 16 + spacing_log2;
    let raw = v >> shift;
    if raw > i64::from(last_cell) {
        return (last_cell as usize, ONE);
    }
    let t = (v & ((1i64 << shift) - 1)) >> spacing_log2;
    (raw as usize, t)
}

impl Coarse3 {
    /// Evalúa `noise` (Q16) en las 75 esquinas del chunk indicado.
    ///
    /// `chunk` son coordenadas de chunk; la esquina (0, 0, 0) local es
    /// `chunk · 16` en bloques. `noise` recibe coordenadas de bloque.
    #[must_use]
    pub fn evaluate<F>(chunk: [i64; 3], mut noise: F) -> Self
    where
        F: FnMut(i64, i64, i64) -> i64,
    {
        let origin = [chunk[0] << 4, chunk[1] << 4, chunk[2] << 4];
        let mut corners = [[[0i64; CORNERS_XZ]; CORNERS_Y]; CORNERS_XZ];
        for (ix, plane) in corners.iter_mut().enumerate() {
            for (iy, row) in plane.iter_mut().enumerate() {
                for (iz, corner) in row.iter_mut().enumerate() {
                    *corner = noise(
                        origin[0] + 4 * (ix as i64),
                        origin[1] + 8 * (iy as i64),
                        origin[2] + 4 * (iz as i64),
                    );
                }
            }
        }
        Self { corners }
    }

    /// Valor interpolado en coordenadas locales de bloque del chunk.
    ///
    /// El rango es `0..=16` por eje (16 es la esquina compartida con el
    /// chunk vecino); fuera de él se satura.
    #[must_use]
    pub fn at_block(&self, local: [i64; 3]) -> i64 {
        self.at_q16([local[0] << 16, local[1] << 16, local[2] << 16])
    }

    /// Igual que [`Coarse3::at_block`] con coordenadas locales en Q16.
    #[must_use]
    pub fn at_q16(&self, local: [i64; 3]) -> i64 {
        let limit = 16 * ONE;
        let x = local[0].clamp(0, limit);
        let y = local[1].clamp(0, limit);
        let z = local[2].clamp(0, limit);
        let (ix, tx) = axis_cell(x, 2);
        let (iy, ty) = axis_cell(y, 3);
        let (iz, tz) = axis_cell(z, 2);
        let c = &self.corners;
        let x00 = lerp(c[ix][iy][iz], c[ix + 1][iy][iz], tx);
        let x10 = lerp(c[ix][iy + 1][iz], c[ix + 1][iy + 1][iz], tx);
        let x01 = lerp(c[ix][iy][iz + 1], c[ix + 1][iy][iz + 1], tx);
        let x11 = lerp(c[ix][iy + 1][iz + 1], c[ix + 1][iy + 1][iz + 1], tx);
        let y0 = lerp(x00, x10, ty);
        let y1 = lerp(x01, x11, ty);
        lerp(y0, y1, tz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fbm::Fbm3;

    /// Chunk de la esquina del mundo de los ejemplos de `docs/mundo/`.
    const FAR_CHUNK: [i64; 3] = [-468_750, -10, 468_749];

    #[test]
    fn evalua_75_esquinas_con_las_coordenadas_correctas() {
        let mut calls = 0;
        let chunk = [-3, -2, 5];
        let coarse = Coarse3::evaluate(chunk, |x, y, z| {
            calls += 1;
            assert_eq!(x % 4, 0);
            assert_eq!(y % 8, 0);
            assert_eq!(z % 4, 0);
            assert!((-48..=-32).contains(&x));
            x + y + z
        });
        assert_eq!(calls, 75);
        assert_eq!(coarse.corners[0][0][0], -48 + -32 + 80);
        assert_eq!(coarse.corners[4][2][4], -32 + -16 + 96);
    }

    #[test]
    fn las_esquinas_son_exactas() {
        let fbm = Fbm3::new(42, 0xD3AD, 6, 4);
        let chunk = [7, -13, 21];
        let coarse = Coarse3::evaluate(chunk, |x, y, z| fbm.sample_block(x, y, z));
        let origin = [chunk[0] << 4, chunk[1] << 4, chunk[2] << 4];
        // Esquinas de la rejilla y bordes del chunk (compartidos con los
        // vecinos, por eso 16 también es válido).
        for lx in [0i64, 4, 8, 12, 16] {
            for ly in [0i64, 8, 16] {
                for lz in [0i64, 4, 8, 12, 16] {
                    let expected = fbm.sample_block(origin[0] + lx, origin[1] + ly, origin[2] + lz);
                    assert_eq!(
                        coarse.at_block([lx, ly, lz]),
                        expected,
                        "esquina {lx},{ly},{lz}"
                    );
                }
            }
        }
    }

    #[test]
    fn el_punto_medio_es_la_media() {
        let fbm = Fbm3::new(42, 1, 6, 4);
        let coarse = Coarse3::evaluate([0, 0, 0], |x, y, z| fbm.sample_block(x, y, z));
        let a = coarse.at_block([0, 0, 0]);
        let b = coarse.at_block([4, 0, 0]);
        let mid = coarse.at_block([2, 0, 0]);
        // La lerp con t = 1/2 trunca, así que admite un error de 1.
        assert!(
            (mid - (a + b) / 2).abs() <= 1,
            "medio {mid}, media {}",
            (a + b) / 2
        );
    }

    #[test]
    fn la_interpolacion_es_continua() {
        let fbm = Fbm3::new(42, 1, 6, 4);
        let coarse = Coarse3::evaluate([-1, 0, 2], |x, y, z| fbm.sample_block(x, y, z));
        // Entre esquinas separadas 4 bloques con ruido en [-ONE, ONE], un
        // paso de 1 bloque no puede saltar más de 2 · ONE / 4.
        let mut max_delta = 0;
        for y in 0..16i64 {
            for z in 0..16i64 {
                for x in 0..16i64 {
                    let delta = (coarse.at_block([x, y, z]) - coarse.at_block([x + 1, y, z])).abs();
                    assert!(delta <= ONE / 2, "salto de {delta} en {x},{y},{z}");
                    max_delta = max_delta.max(delta);
                }
            }
        }
        assert!(max_delta > 0, "rejilla constante: no prueba nada");
    }

    #[test]
    fn funciona_en_chunks_negativos_y_lejanos() {
        let fbm = Fbm3::new(7, 3, 6, 4);
        let coarse = Coarse3::evaluate(FAR_CHUNK, |x, y, z| fbm.sample_block(x, y, z));
        let origin = [FAR_CHUNK[0] << 4, FAR_CHUNK[1] << 4, FAR_CHUNK[2] << 4];
        assert_eq!(
            coarse.at_block([0, 0, 0]),
            fbm.sample_block(origin[0], origin[1], origin[2])
        );
        assert_eq!(
            coarse.at_block([16, 16, 16]),
            fbm.sample_block(origin[0] + 16, origin[1] + 16, origin[2] + 16)
        );
    }

    #[test]
    fn fuera_de_rango_se_satura_sin_panico() {
        let coarse = Coarse3::evaluate([0, 0, 0], |x, y, z| x + y + z);
        assert_eq!(coarse.at_block([-5, 0, 0]), 0);
        assert_eq!(coarse.at_block([17, 20, -1]), coarse.at_block([16, 16, 0]));
        assert_eq!(
            coarse.at_q16([-1, 17 << 16, 0]),
            coarse.at_q16([0, 16 << 16, 0])
        );
    }
}
