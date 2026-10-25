//! Columna 2D del generador del mundo grande (W1, fase 1).
//!
//! [`sample_column`] resume la columna del mundo en `(x, z)`: cinco
//! parámetros climáticos en Q16, la altura de superficie en bloques y el
//! bioma. Todo es una función pura de (semilla, coordenadas) sobre una
//! rejilla global de celdas de 4 bloques (índice `x >> 2`, alineada al
//! mundo) interpolada bilinealmente en Q16: los chunks vecinos siempre
//! están de acuerdo en la costura, y [`ColumnCache`] reproduce bit a bit a
//! [`sample_column`] evaluando una sola vez los 49 puntos del chunk.

mod biome;
mod curves;

use biome::select_biome;
use curves::surface_height_q16;
use grisu_noise::{Fbm2, ONE, lerp, ridge, warp2};

/// Bits del lado de las celdas de la rejilla de columnas: 4 bloques.
pub const GRID_SHIFT: i32 = 2;
/// Altura máxima de la superficie, en bloques.
pub const MAX_SURFACE: i32 = 640;
/// Altura mínima de la superficie, en bloques.
pub const MIN_SURFACE: i32 = -64;
/// Nivel del mar; en W1 es dato, no genera bloques de agua.
pub const WATER_LEVEL: i32 = -16;
/// Cota del salto de `surface_y` entre columnas vecinas, en bloques.
pub const MAX_COLUMN_STEP: i32 = 24;

/// Sal de la capa de continentalidad.
const SALT_CONTINENTALITY: u64 = 0x11C0_0000_0000_0001;
/// Sal de la capa de erosión.
const SALT_EROSION: u64 = 0x11C0_0000_0000_0002;
/// Sal de la capa de picos y valles.
const SALT_PEAKS_VALLEYS: u64 = 0x11C0_0000_0000_0003;
/// Sal de la capa de temperatura.
const SALT_TEMPERATURE: u64 = 0x11C0_0000_0000_0004;
/// Sal de la capa de humedad.
const SALT_HUMIDITY: u64 = 0x11C0_0000_0000_0005;
/// Sal de la deformación de dominio en X de la continentalidad.
const SALT_WARP_X: u64 = 0x11C0_0000_0000_0006;
/// Sal de la deformación de dominio en Z de la continentalidad.
const SALT_WARP_Y: u64 = 0x11C0_0000_0000_0007;
/// Sal de la capa de ríos.
const SALT_RIVER: u64 = 0x11C0_0000_0000_0008;

/// Bioma de la superficie de una columna, elegido por cercanía al centroide.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceBiome {
    Cordillera = 0,
    NeedleForest = 1,
    Highland = 2,
    RedDesert = 3,
    Badlands = 4,
    RiverValley = 5,
    Tundra = 6,
    InlandSea = 7,
}

/// Resumen de una columna del mundo: los cinco parámetros climáticos en Q16
/// (picos y valles en `[0, ONE]`, el resto en `[-ONE, ONE]`), alturas en
/// bloques y bioma.
#[derive(Clone, Copy, Debug)]
pub struct ColumnParams {
    pub continentality: i64,
    pub erosion: i64,
    pub peaks_valleys: i64,
    pub temperature: i64,
    pub humidity: i64,
    /// Bloque de superficie: el sólido más alto de la columna.
    pub surface_y: i32,
    /// Nivel del mar de la columna (siempre [`WATER_LEVEL`]).
    pub water_level: i32,
    pub biome: SurfaceBiome,
}

/// Valor de un punto de la rejilla y acumulador de la interpolación:
/// `v` lleva, en este orden, continentalidad, erosión, picos, temperatura,
/// humedad y altura (todo en Q16; la altura ya lleva ríos y recortes).
#[derive(Clone, Copy, Debug)]
struct GridPoint {
    v: [i64; 6],
}

/// Fracción de la posición dentro de su celda de rejilla, en Q16.
#[inline]
fn cell_fraction(pos: i32) -> i64 {
    i64::from(pos & ((1 << GRID_SHIFT) - 1)) << (16 - GRID_SHIFT)
}

/// Interpolación bilineal de la celda en el orden canónico
/// `lerp(lerp(q00, q10, fx), lerp(q01, q11, fx), fz)`. Como
/// [`sample_column`] y [`ColumnCache::column`] pasan por aquí con las
/// esquinas en el mismo orden y las mismas fracciones, los dos caminos dan
/// el mismo resultado bit a bit.
fn interpolate_cell(
    q00: &GridPoint,
    q10: &GridPoint,
    q01: &GridPoint,
    q11: &GridPoint,
    fx: i64,
    fz: i64,
) -> GridPoint {
    let mix = |a: i64, b: i64, c: i64, d: i64| lerp(lerp(a, b, fx), lerp(c, d, fx), fz);
    let mut v = [0i64; 6];
    for (i, o) in v.iter_mut().enumerate() {
        *o = mix(q00.v[i], q10.v[i], q01.v[i], q11.v[i]);
    }
    GridPoint { v }
}

/// Convierte el punto interpolado en los parámetros públicos: redondeo de
/// la altura al bloque más cercano y bioma por centroides.
fn finish_column(p: GridPoint) -> ColumnParams {
    ColumnParams {
        continentality: p.v[0],
        erosion: p.v[1],
        peaks_valleys: p.v[2],
        temperature: p.v[3],
        humidity: p.v[4],
        surface_y: ((p.v[5] + (1 << 15)) >> 16) as i32,
        water_level: WATER_LEVEL,
        biome: select_biome(p.v[3], p.v[4], p.v[0]),
    }
}

/// Las capas de ruido de la columna, todas derivadas de la semilla.
struct NoiseStack {
    continentality: Fbm2,
    warp_x: Fbm2,
    warp_y: Fbm2,
    erosion: Fbm2,
    peaks: Fbm2,
    temperature: Fbm2,
    humidity: Fbm2,
    river: Fbm2,
}

impl NoiseStack {
    fn new(seed: u64) -> Self {
        Self {
            continentality: Fbm2::new(seed, SALT_CONTINENTALITY, 12, 4),
            warp_x: Fbm2::new(seed, SALT_WARP_X, 8, 2),
            warp_y: Fbm2::new(seed, SALT_WARP_Y, 8, 2),
            erosion: Fbm2::new(seed, SALT_EROSION, 11, 3),
            peaks: Fbm2::new(seed, SALT_PEAKS_VALLEYS, 9, 3),
            temperature: Fbm2::new(seed, SALT_TEMPERATURE, 12, 3),
            humidity: Fbm2::new(seed, SALT_HUMIDITY, 10, 3),
            river: Fbm2::new(seed, SALT_RIVER, 13, 2),
        }
    }

    /// Evalúa el punto `(gx, gz)` de la rejilla (celdas de 4 bloques).
    fn point(&self, gx: i64, gz: i64) -> GridPoint {
        let (bx, bz) = (gx << GRID_SHIFT, gz << GRID_SHIFT);
        let (wx, wz) = warp2(bx << 16, bz << 16, &self.warp_x, &self.warp_y, 8 * ONE);
        let cont = self.continentality.sample_q16(wx, wz);
        let ero = self.erosion.sample_block(bx, bz);
        let peaks = ridge(self.peaks.sample_block(bx, bz));
        let river = ridge(self.river.sample_block(bx, bz));
        GridPoint {
            v: [
                cont,
                ero,
                peaks,
                self.temperature.sample_block(bx, bz),
                self.humidity.sample_block(bx, bz),
                surface_height_q16(cont, ero, peaks, river),
            ],
        }
    }
}

/// Parámetros de la columna del mundo en `(x, z)`, en bloques globales:
/// función pura, el valor no depende del chunk que contenga la columna.
#[must_use]
pub fn sample_column(seed: u64, x: i32, z: i32) -> ColumnParams {
    let stack = NoiseStack::new(seed);
    let gx = i64::from(x >> GRID_SHIFT);
    let gz = i64::from(z >> GRID_SHIFT);
    let q00 = stack.point(gx, gz);
    let q10 = stack.point(gx + 1, gz);
    let q01 = stack.point(gx, gz + 1);
    let q11 = stack.point(gx + 1, gz + 1);
    finish_column(interpolate_cell(
        &q00,
        &q10,
        &q01,
        &q11,
        cell_fraction(x),
        cell_fraction(z),
    ))
}

/// Rejilla 7×7 de puntos para un chunk: locales `{-4, 0, 4, ..., 20}` cada
/// 4 bloques, de modo que [`ColumnCache::column`] responde en `-1..=16`
/// con margen para las pendientes de los vecinos.
pub(crate) struct ColumnCache {
    /// Esquina mínima del chunk, en bloques.
    origin_x: i32,
    origin_z: i32,
    /// Índice global del punto (0, 0): una celda antes del origen.
    base_gx: i32,
    base_gz: i32,
    /// Puntos por filas: `points[iz][ix]`.
    points: [[GridPoint; 7]; 7],
}

impl ColumnCache {
    /// Evalúa los 49 puntos globales que cubren el chunk con margen.
    pub(crate) fn evaluate(seed: u64, chunk_x: i32, chunk_z: i32) -> Self {
        let stack = NoiseStack::new(seed);
        let origin_x = chunk_x << 4;
        let origin_z = chunk_z << 4;
        let base_gx = (origin_x >> GRID_SHIFT) - 1;
        let base_gz = (origin_z >> GRID_SHIFT) - 1;
        let points = std::array::from_fn(|iz| {
            std::array::from_fn(|ix| {
                let gx = i64::from(base_gx + ix as i32);
                let gz = i64::from(base_gz + iz as i32);
                stack.point(gx, gz)
            })
        });
        Self {
            origin_x,
            origin_z,
            base_gx,
            base_gz,
            points,
        }
    }

    /// Parámetros de la columna en coordenadas locales `-1..=16`: recorre
    /// exactamente la misma aritmética que [`sample_column`].
    pub(crate) fn column(&self, local_x: i32, local_z: i32) -> ColumnParams {
        let x = self.origin_x + local_x;
        let z = self.origin_z + local_z;
        let ix = ((x >> GRID_SHIFT) - self.base_gx) as usize;
        let iz = ((z >> GRID_SHIFT) - self.base_gz) as usize;
        finish_column(interpolate_cell(
            &self.points[iz][ix],
            &self.points[iz][ix + 1],
            &self.points[iz + 1][ix],
            &self.points[iz + 1][ix + 1],
            cell_fraction(x),
            cell_fraction(z),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Igualdad campo a campo a través del formato de depuración.
    fn same_params(a: &ColumnParams, b: &ColumnParams) -> bool {
        format!("{a:?}") == format!("{b:?}")
    }

    /// Todos los campos acotados: clima en `[-ONE, ONE]`, superficie dentro
    /// del dominio y nivel del mar fijo.
    fn parameters_in_range(p: &ColumnParams) -> bool {
        let clima = [
            p.continentality,
            p.erosion,
            p.peaks_valleys,
            p.temperature,
            p.humidity,
        ];
        clima.iter().all(|&v| (-ONE..=ONE).contains(&v))
            && (MIN_SURFACE..=MAX_SURFACE).contains(&p.surface_y)
            && p.water_level == WATER_LEVEL
    }

    #[test]
    fn parameters_stay_in_range() {
        for seed in [0, 7, 42, u64::MAX] {
            for z in (-48..48).step_by(3) {
                for x in (-48..48).step_by(3) {
                    assert!(parameters_in_range(&sample_column(seed, x, z)), "({x},{z})");
                }
            }
        }
    }

    #[test]
    fn neighbouring_columns_stay_within_the_step_bound() {
        const EDGE: i32 = 192;
        for seed in [42, 2026] {
            let mut previa: Vec<i32> = (-EDGE..=EDGE)
                .map(|x| sample_column(seed, x, -EDGE).surface_y)
                .collect();
            for z in (-EDGE + 1)..=EDGE {
                let fila: Vec<i32> = (-EDGE..=EDGE)
                    .map(|x| sample_column(seed, x, z).surface_y)
                    .collect();
                for s in fila.windows(2) {
                    assert!(
                        (s[1] - s[0]).abs() <= MAX_COLUMN_STEP,
                        "salto X z={z}: {s:?}"
                    );
                }
                for ((&a, &b), x) in previa.iter().zip(&fila).zip(-EDGE..=EDGE) {
                    assert!(
                        (b - a).abs() <= MAX_COLUMN_STEP,
                        "salto Z ({x},{z}): {a} a {b}"
                    );
                }
                previa = fila;
            }
        }
    }

    #[test]
    fn is_deterministic() {
        for (x, z) in [(0, 0), (-5, 9), (123, -456), (7_499_999, -7_499_999)] {
            let (a, b) = (sample_column(42, x, z), sample_column(42, x, z));
            assert!(same_params(&a, &b), "dos llamadas distintas en ({x},{z})");
        }
    }

    #[test]
    fn cache_matches_sample_column() {
        for (cx, cz) in [(0, 0), (-1, -1), (3, -7), (-468_750, 468_749)] {
            let cache = ColumnCache::evaluate(42, cx, cz);
            for lz in -1..=16 {
                for lx in -1..=16 {
                    let a = cache.column(lx, lz);
                    let b = sample_column(42, cx * 16 + lx, cz * 16 + lz);
                    assert!(same_params(&a, &b), "chunk ({cx},{cz}) local ({lx},{lz})");
                }
            }
        }
    }

    #[test]
    fn global_sweep_has_at_least_five_biomes() {
        let mut vistos = [false; 8];
        for i in 0..64i32 {
            for j in 0..64i32 {
                vistos[sample_column(42, i * 256, j * 256).biome as usize] = true;
            }
        }
        let distintos = vistos.iter().fold(0, |n, &v| n + usize::from(v));
        assert!(distintos >= 5, "solo {distintos} biomas en la barrida");
    }
}
