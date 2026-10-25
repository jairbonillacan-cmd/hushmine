//! Ruido celular (Voronoi) 2D y 3D en punto fijo Q16.16.
//!
//! Cada celda de la retícula coloca por hash un punto característico y la
//! muestra devuelve la celda más cercana con sus dos distancias al
//! cuadrado, sin raíces: comparar cuadrados da el mismo orden. Sirve para
//! regiones de bioma subterráneo, fallas y cámaras: `cell_hash` elige el
//! contenido de cada región y `edge_q16`, que vale 0 en las fronteras
//! entre celdas, traza fallas geológicas y grietas.
//!
//! INV-W03: función pura de (semilla, coordenadas), sin `f32`/`f64` y sin
//! desbordamientos para ningún `i64` de entrada: el vector al punto
//! característico de cada vecina se parte en celda vecina más fracción y
//! nunca se materializa en coordenadas de mundo.

use crate::fixed::ONE;
use crate::hash::{hash2d, hash3d};

/// Muestra del ruido celular en un punto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellularSample {
    /// Hash de la celda más cercana (empate: gana la celda menor lexicográfica).
    pub cell_hash: u64,
    /// Menor distancia al cuadrado en unidades de celda (celdas²), en Q16
    /// `[0, ONE]`; satura en `ONE` a partir de 1 celda de distancia.
    pub f1_sq: i32,
    /// Segunda menor distancia al cuadrado, mismo formato.
    pub f2_sq: i32,
    /// Diferencia entre las dos distancias, calculada antes de saturarlas y
    /// acotada a `[0, ONE]`: vale 0 solo en las fronteras entre celdas,
    /// aunque las dos distancias pasen de 1 celda.
    pub edge_q16: i32,
}

/// Ruido celular 2D.
#[derive(Clone, Copy, Debug)]
pub struct Cellular2 {
    seed: u64,
    /// Log2 del lado de celda en bloques; la retícula Q16 desplaza `16 + period_shift`.
    period_shift: u32,
    jitter_q16: i32,
}

/// Ruido celular 3D.
#[derive(Clone, Copy, Debug)]
pub struct Cellular3 {
    seed: u64,
    period_shift: u32,
    jitter_q16: i32,
}

/// Deriva `edge_q16` de las distancias sin saturar y luego satura las dos
/// en `ONE`. Al revés, dos distancias distintas por encima de 1 celda darían
/// borde 0 lejos de cualquier frontera.
fn finish(f1: i64, f2: i64, cell_hash: u64) -> CellularSample {
    let edge_q16 = (f2 - f1).clamp(0, ONE) as i32;
    CellularSample {
        cell_hash,
        f1_sq: f1.min(ONE) as i32,
        f2_sq: f2.min(ONE) as i32,
        edge_q16,
    }
}

impl Cellular2 {
    /// Crea el ruido con jitter completo: el punto característico puede caer
    /// en cualquier lugar de su celda.
    #[must_use]
    pub fn new(seed: u64, period_shift: u32) -> Self {
        Self::with_jitter(seed, period_shift, ONE as i32)
    }

    /// Crea el ruido con jitter dado, acotado a `[0, ONE]`; 0 clava cada
    /// punto característico en la esquina mínima de su celda.
    #[must_use]
    pub fn with_jitter(seed: u64, period_shift: u32, jitter_q16: i32) -> Self {
        Self {
            seed,
            period_shift: period_shift.clamp(1, 16),
            jitter_q16: jitter_q16.clamp(0, ONE as i32),
        }
    }

    /// Muestrea en coordenadas de bloque (enteras).
    ///
    /// Válido para |coordenada| ≤ 2⁴⁷ bloques; más lejos el mundo no llega.
    #[must_use]
    pub fn sample_block(&self, x: i64, z: i64) -> CellularSample {
        self.sample(x << 16, z << 16)
    }

    /// Muestrea en coordenadas Q16 (1 bloque = 65 536). Válido para
    /// cualquier `i64` sin desbordar.
    #[must_use]
    pub fn sample(&self, x: i64, z: i64) -> CellularSample {
        let k = self.period_shift;
        let span = 1i64 << (16 + k);
        let bx = x >> (16 + k);
        let bz = z >> (16 + k);
        let local_x = x & (span - 1);
        let local_z = z & (span - 1);
        let mut f1 = i64::MAX;
        let mut f2 = i64::MAX;
        let mut best = (0i64, 0i64);
        let mut best_hash = 0u64;
        for dx in -1..=1i64 {
            for dz in -1..=1i64 {
                let cx = bx.wrapping_add(dx);
                let cz = bz.wrapping_add(dz);
                let h = hash2d(cx, cz, self.seed);
                let off_x = ((h & 0xFFFF) as i64 * i64::from(self.jitter_q16)) >> 16;
                let off_z = (((h >> 16) & 0xFFFF) as i64 * i64::from(self.jitter_q16)) >> 16;
                // Vector al punto característico por partes: celda vecina
                // (`dx * span`) más fracción dentro de la celda (`off << k`).
                // Así no hay coordenada de mundo que desborde en `i64::MIN`.
                let ex = local_x - (dx * span + (off_x << k));
                let ez = local_z - (dz * span + (off_z << k));
                // Distancia al cuadrado en celdas² Q16: con |n| <= 2·ONE la
                // suma no pasa de 8·ONE² y el desplazamiento la expresa en
                // Q16 (media celda -> ONE/4; satura a partir de 1 celda).
                let nx = ex >> k;
                let nz = ez >> k;
                let dist_sq = (nx * nx + nz * nz) >> 16;
                if dist_sq < f1 || (dist_sq == f1 && (cx, cz) < best) {
                    f2 = f1;
                    f1 = dist_sq;
                    best = (cx, cz);
                    best_hash = h;
                } else if dist_sq < f2 {
                    f2 = dist_sq;
                }
            }
        }
        finish(f1, f2, best_hash)
    }
}

impl Cellular3 {
    /// Crea el ruido como [`Cellular2::new`].
    #[must_use]
    pub fn new(seed: u64, period_shift: u32) -> Self {
        Self::with_jitter(seed, period_shift, ONE as i32)
    }

    /// Crea el ruido como [`Cellular2::with_jitter`].
    #[must_use]
    pub fn with_jitter(seed: u64, period_shift: u32, jitter_q16: i32) -> Self {
        Self {
            seed,
            period_shift: period_shift.clamp(1, 16),
            jitter_q16: jitter_q16.clamp(0, ONE as i32),
        }
    }

    /// Muestrea en coordenadas de bloque (enteras).
    ///
    /// Válido para |coordenada| ≤ 2⁴⁷ bloques; más lejos el mundo no llega.
    #[must_use]
    pub fn sample_block(&self, x: i64, y: i64, z: i64) -> CellularSample {
        self.sample(x << 16, y << 16, z << 16)
    }

    /// Muestrea en coordenadas Q16 (1 bloque = 65 536). Válido para
    /// cualquier `i64` sin desbordar.
    #[must_use]
    pub fn sample(&self, x: i64, y: i64, z: i64) -> CellularSample {
        let k = self.period_shift;
        let span = 1i64 << (16 + k);
        let bx = x >> (16 + k);
        let by = y >> (16 + k);
        let bz = z >> (16 + k);
        let local_x = x & (span - 1);
        let local_y = y & (span - 1);
        let local_z = z & (span - 1);
        let mut f1 = i64::MAX;
        let mut f2 = i64::MAX;
        let mut best = (0i64, 0i64, 0i64);
        let mut best_hash = 0u64;
        for dx in -1..=1i64 {
            for dy in -1..=1i64 {
                for dz in -1..=1i64 {
                    let cx = bx.wrapping_add(dx);
                    let cy = by.wrapping_add(dy);
                    let cz = bz.wrapping_add(dz);
                    let h = hash3d(cx, cy, cz, self.seed);
                    let off_x = ((h & 0xFFFF) as i64 * i64::from(self.jitter_q16)) >> 16;
                    let off_y = (((h >> 16) & 0xFFFF) as i64 * i64::from(self.jitter_q16)) >> 16;
                    let off_z = (((h >> 32) & 0xFFFF) as i64 * i64::from(self.jitter_q16)) >> 16;
                    let ex = local_x - (dx * span + (off_x << k));
                    let ey = local_y - (dy * span + (off_y << k));
                    let ez = local_z - (dz * span + (off_z << k));
                    // Misma normalización que en 2D: celdas² en Q16.
                    let nx = ex >> k;
                    let ny = ey >> k;
                    let nz = ez >> k;
                    let dist_sq = (nx * nx + ny * ny + nz * nz) >> 16;
                    if dist_sq < f1 || (dist_sq == f1 && (cx, cy, cz) < best) {
                        f2 = f1;
                        f1 = dist_sq;
                        best = (cx, cy, cz);
                        best_hash = h;
                    } else if dist_sq < f2 {
                        f2 = dist_sq;
                    }
                }
            }
        }
        finish(f1, f2, best_hash)
    }
}

#[cfg(test)]
#[path = "cellular_tests.rs"]
mod tests;

#[cfg(test)]
mod edge_tests {
    use super::*;

    /// Caso de la revisión de #10: las dos distancias pasan de 1 celda
    /// (65 708 y 65 878 en Q16) y son distintas, así que no es frontera.
    #[test]
    fn edge_is_not_zero_when_both_distances_saturate() {
        let s = Cellular2::new(42, 4).sample_block(4_861_678, 6_020_082);
        assert_eq!((s.f1_sq, s.f2_sq), (ONE as i32, ONE as i32));
        assert_eq!(s.edge_q16, 65_878 - 65_708);
    }
}
