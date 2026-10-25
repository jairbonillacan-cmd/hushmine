//! Generador del mundo grande (W1): une columna, densidad y litología.
//!
//! [`block_at`] es la referencia: crudeza con
//! [`supported_solid`](crate::density::supported_solid), columna con
//! [`sample_column`](crate::column::sample_column) y roca con
//! [`solid_block`](crate::lithology::solid_block). [`generate_chunk`] es
//! la vía rápida y da EXACTAMENTE los mismos bloques (`INV-W02`):
//!
//! (a) esquinas y fracciones idénticas de la rejilla gruesa global (X/Z
//!     cada 4, Y cada 8; los orígenes de chunk son múltiplos de 16);
//! (b) atajos de zona por la cota `±AMP/SLOPE` de `crate::density`, también
//!     a nivel de chunk entero (cielo con `MAX_SURFACE`, lecho);
//! (c) `ColumnCache ≡ sample_column` (probado en `column`) y (d) pendiente
//!     anclada en la propia columna con su cruz de vecinas: los mismos
//!     parámetros, altura y pendiente en los dos caminos.
//!
//! Bóveda y lecho (`y ≤ MANTO_BOTTOM`): crudeza de referencia vía `density::raw_solid`.

use grisu_voxel::{AIR, BlockId, CHUNK_SIZE, Chunk, ChunkPos};

use crate::blocks::HARD_ROCK;
use crate::column::{ColumnCache, ColumnParams, MAX_SURFACE, sample_column};
use crate::density::{
    BAND_HALF, BEDROCK_Y, MANTO_BOTTOM, Zone, band_density, noise_corner, raw_solid,
    supported_solid, terrain_zone,
};
use crate::lithology::solid_block;

/// Lado del chunk como `i64`.
const SIDE: i64 = CHUNK_SIZE as i64;
/// Locales del cubo ampliado (margen 1): `-1..=16`, 18 valores por eje.
const WIDE: usize = 18;

/// Pendiente de la columna `(x, z)` de superficie `h`: máximo `|Δh|` contra
/// las 4 vecinas (acotado por `MAX_COLUMN_STEP`).
fn slope_of(seed: u64, x: i32, z: i32, h: i32) -> i32 {
    let step = |dx: i32, dz: i32| (h - sample_column(seed, x + dx, z + dz).surface_y).abs();
    step(1, 0).max(step(-1, 0)).max(step(0, 1)).max(step(0, -1))
}

/// Bloque del mundo grande en `(x, y, z)`: la referencia exacta contra la
/// que se compara `generate_chunk` (precondición de `density`: |x|, |z| ≤ 7 499 999).
#[must_use]
pub fn block_at(seed: u64, x: i64, y: i64, z: i64) -> BlockId {
    if !supported_solid(seed, x, y, z) {
        return AIR;
    }
    let (cx, cz) = (x as i32, z as i32);
    let params = sample_column(seed, cx, cz);
    let h = params.surface_y;
    solid_block(seed, x, y, z, &params, h, slope_of(seed, cx, cz, h))
}

/// Caché de esquinas gruesas del chunk: planos X/Z `{o−4 … o+20}` (7) e Y
/// `{o−8 … o+24}` (5; el plano superior lo pide la vecina +1 en Y): la
/// crudeza de banda de cualquier bloque del cubo ampliado sale de aquí.
struct BandCorners {
    ox: i64,
    oy: i64,
    oz: i64,
    /// Valores de las esquinas globales, indexado `[ix][iy][iz]`.
    v: [[[i64; 7]; 5]; 7],
}

impl BandCorners {
    /// Evalúa las 245 esquinas globales que cubren el cubo ampliado.
    fn evaluate(seed: u64, ox: i64, oy: i64, oz: i64) -> Self {
        let v = std::array::from_fn(|ix| {
            std::array::from_fn(|iy| {
                std::array::from_fn(|iz| {
                    let (cx, cy, cz) = (4 * ix as i64 - 4, 8 * iy as i64 - 8, 4 * iz as i64 - 4);
                    noise_corner(seed, ox + cx, oy + cy, oz + cz)
                })
            })
        });
        Self { ox, oy, oz, v }
    }

    /// Crudeza de banda del bloque global `(x, y, z)` con superficie `h`:
    /// la celda gruesa correcta y las fracciones globales, como `raw_solid`.
    fn solid(&self, x: i64, y: i64, z: i64, h: i32) -> bool {
        let (cx0, cy0, cz0) = (x & !3, y & !7, z & !3);
        // Restas múltiplo del paso y no negativas: `>>` es la división.
        let (ix, iy, iz) = (
            ((cx0 - (self.ox - 4)) >> 2) as usize,
            ((cy0 - (self.oy - 8)) >> 3) as usize,
            ((cz0 - (self.oz - 4)) >> 2) as usize,
        );
        let cell = std::array::from_fn(|a| {
            std::array::from_fn(|b| std::array::from_fn(|c| self.v[ix + a][iy + b][iz + c]))
        });
        band_density(&cell, (x & 3) << 14, (y & 7) << 13, (z & 3) << 14, h, y) > 0
    }
}

/// Crudeza (`raw_solid`) del cubo ampliado, indexado `[lx+1][ly+1][lz+1]`
/// con locales `-1..=16`: Bóveda/lecho vía `raw_solid`, frente con zona o caché.
fn raw_cube(
    seed: u64,
    ox: i64,
    oy: i64,
    oz: i64,
    params: &[[ColumnParams; WIDE]; WIDE],
    band: Option<&BandCorners>,
) -> [[[bool; WIDE]; WIDE]; WIDE] {
    let mut raw = [[[false; WIDE]; WIDE]; WIDE];
    for (lx, plane_x) in raw.iter_mut().enumerate() {
        for (ly, plane_y) in plane_x.iter_mut().enumerate() {
            for (lz, cell) in plane_y.iter_mut().enumerate() {
                let (x, y, z) = (ox + lx as i64 - 1, oy + ly as i64 - 1, oz + lz as i64 - 1);
                *cell = if y > MANTO_BOTTOM {
                    let h = params[lz][lx].surface_y;
                    match terrain_zone(h, y) {
                        Zone::Solid => true,
                        Zone::Air => false,
                        // El caché existe si alguna banda toca el cubo; si
                        // no, decide la referencia.
                        Zone::Band => {
                            band.map_or_else(|| raw_solid(seed, x, y, z), |b| b.solid(x, y, z, h))
                        }
                    }
                } else {
                    raw_solid(seed, x, y, z)
                };
            }
        }
    }
    raw
}

/// Escribe el chunk aplicando la litología a cada celda sólida. Con
/// `raw = None` el chunk es roca maciza; si no, cada celda exige ser sólida
/// y tener alguna de sus 6 vecinas sólidas (regla del soporte).
fn emit_chunk(
    seed: u64,
    ox: i64,
    oy: i64,
    oz: i64,
    params: &[[ColumnParams; WIDE]; WIDE],
    slope: &[[i32; 16]; 16],
    raw: Option<&[[[bool; WIDE]; WIDE]; WIDE]>,
) -> Chunk {
    let mut chunk = Chunk::default();
    for (lz, slope_row) in slope.iter().enumerate() {
        for ly in 0..16usize {
            for (lx, &sl) in slope_row.iter().enumerate() {
                let (gx, gy, gz) = (lx + 1, ly + 1, lz + 1);
                if let Some(g) = raw {
                    let neigh = g[gx + 1][gy][gz]
                        || g[gx - 1][gy][gz]
                        || g[gx][gy + 1][gz]
                        || g[gx][gy - 1][gz]
                        || g[gx][gy][gz + 1]
                        || g[gx][gy][gz - 1];
                    if !g[gx][gy][gz] || !neigh {
                        continue;
                    }
                }
                let (x, y, z) = (ox + lx as i64, oy + ly as i64, oz + lz as i64);
                let p = &params[gz][gx];
                let block = solid_block(seed, x, y, z, p, p.surface_y, sl);
                if block != AIR {
                    chunk.set(lx, ly, lz, block);
                }
            }
        }
    }
    chunk
}

/// Genera el chunk `pos` del mundo grande con la semilla `seed`.
///
/// Función pura (`INV-W01`), idéntica bloque a bloque a [`block_at`]
/// (`INV-W02`), con coordenadas globales en `i64` (`INV-W03`).
#[must_use]
pub fn generate_chunk(seed: u64, pos: ChunkPos) -> Chunk {
    let o = pos.origin();
    let (ox, oy, oz) = (i64::from(o[0]), i64::from(o[1]), i64::from(o[2]));

    // Atajo lecho: crudeza cierta y litología HARD_ROCK, todo el chunk.
    if oy + SIDE - 1 <= BEDROCK_Y {
        return Chunk::filled(HARD_ROCK);
    }
    // Atajo cielo: por encima de `h + BAND_HALF` jamás hay sólido (cota de
    // `density`), y `h ≤ MAX_SURFACE`.
    if oy > i64::from(MAX_SURFACE) + BAND_HALF {
        return Chunk::default();
    }

    // Columnas locales −1..=16: litología, crudeza de vecinas y pendientes.
    let cache = ColumnCache::evaluate(seed, pos.x, pos.z);
    let params: [[ColumnParams; WIDE]; WIDE] = std::array::from_fn(|lz| {
        std::array::from_fn(|lx| cache.column(lx as i32 - 1, lz as i32 - 1))
    });

    // Pendientes por columna local y cotas de altura de las bandas.
    let mut slope = [[0i32; 16]; 16];
    let (mut min_h, mut max_h) = (i32::MAX, i32::MIN);
    for row in &params {
        for p in row {
            min_h = min_h.min(p.surface_y);
            max_h = max_h.max(p.surface_y);
        }
    }
    for lz in 0..16 {
        for lx in 0..16 {
            // Ancla en la propia columna (lx, lz): igual que `slope_of`.
            let h = params[lz + 1][lx + 1].surface_y;
            let jump = |ix: usize, iz: usize| (h - params[iz][ix].surface_y).abs();
            slope[lz][lx] = jump(lx, lz + 1)
                .max(jump(lx + 2, lz + 1))
                .max(jump(lx + 1, lz))
                .max(jump(lx + 1, lz + 2));
        }
    }

    // Clasificación vertical del cubo ampliado `[oy−1, oy+16]`: los atajos
    // de chunk entero solo valen si TODO el cubo es frente de superficie.
    let (bottom, top) = (oy - 1, oy + SIDE);
    let terrain_only = bottom > MANTO_BOTTOM;
    // Suelo del tramo de terreno (por encima de la Bóveda) del cubo.
    let floor_y = bottom.max(MANTO_BOTTOM + 1);
    // Toda banda por debajo del cubo (la más alta, la de `max_h`): aire.
    let all_air = i64::from(max_h) + BAND_HALF < floor_y;
    // Toda banda por encima (la más baja, la de `min_h`): roca maciza.
    let all_solid = i64::from(min_h) - BAND_HALF > top;
    if terrain_only && all_air {
        return Chunk::default();
    }
    if terrain_only && all_solid {
        return emit_chunk(seed, ox, oy, oz, &params, &slope, None);
    }

    // ¿Toca alguna banda el tramo de terreno del cubo? Si no, las zonas
    // deciden sin Fbm3.
    let band_needed =
        i64::from(max_h) + BAND_HALF >= floor_y && i64::from(min_h) - BAND_HALF <= top;
    let band = band_needed.then(|| BandCorners::evaluate(seed, ox, oy, oz));
    let raw = raw_cube(seed, ox, oy, oz, &params, band.as_ref());
    emit_chunk(seed, ox, oy, oz, &params, &slope, Some(&raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{DIAMOND, DIRT, GOLD, LAVA, ORE, ROCK};
    use crate::column::SurfaceBiome;

    /// Compara `block_at` y `generate_chunk` bloque a bloque (paso `step`).
    fn matches_reference(seed: u64, pos: ChunkPos, step: usize) {
        let chunk = generate_chunk(seed, pos);
        let o = pos.origin();
        for lz in (0..CHUNK_SIZE).step_by(step) {
            for ly in (0..CHUNK_SIZE).step_by(step) {
                for lx in (0..CHUNK_SIZE).step_by(step) {
                    let g = [o[0] + lx as i32, o[1] + ly as i32, o[2] + lz as i32];
                    let want = block_at(seed, i64::from(g[0]), i64::from(g[1]), i64::from(g[2]));
                    assert_eq!(chunk.get(lx, ly, lz), want, "{pos:?} ({lx},{ly},{lz})");
                }
            }
        }
    }

    /// Chunk cuyo rango Y contiene la superficie de la columna central de
    /// `(ccx, ccz)`: la banda del frente cruza el chunk con sólido y aire.
    fn surface_chunk(seed: u64, ccx: i32, ccz: i32) -> ChunkPos {
        let h = sample_column(seed, ccx * 16 + 8, ccz * 16 + 8).surface_y;
        ChunkPos::new(ccx, h >> 4, ccz)
    }

    #[test]
    fn block_at_matches_generate_chunk() {
        // Superficie en tres climas distintos, negativas, frontera del
        // manto, manto, Bóveda y frontera del lecho.
        for pos in [
            ChunkPos::new(0, 0, 0),
            surface_chunk(42, 0, 0),
            surface_chunk(42, 300, 300),
            surface_chunk(42, -300, -200),
            ChunkPos::new(-1, -1, 0),
            ChunkPos::new(2, -40, 3),
            ChunkPos::new(2, -41, 3),
            ChunkPos::new(0, -20, 0),
            ChunkPos::new(1, -100, -1),
            ChunkPos::new(-3, -256, 2),
        ] {
            matches_reference(42, pos, 1);
        }
        matches_reference(7, ChunkPos::new(0, 0, 0), 1);
        // Lejanos con chunk_y = −1: muestreo con paso 2 por presupuesto.
        matches_reference(42, ChunkPos::new(468_740, -1, -468_740), 2);
        matches_reference(42, ChunkPos::new(-468_740, -1, 468_740), 2);
    }

    #[test]
    fn bedrock_chunk_is_all_hard_rock() {
        // −257·16 = −4112: el chunk entero queda bajo el lecho.
        for &b in generate_chunk(42, ChunkPos::new(0, -257, 0)).as_bytes() {
            assert_eq!(b, HARD_ROCK);
        }
    }

    #[test]
    fn sky_above_the_bound_is_empty() {
        // 41·16 = 656 ≥ 640 + 8 + 1 = 649: nunca hay sólido tan alto.
        assert!(generate_chunk(42, ChunkPos::new(0, 41, 0)).is_empty());
        assert!(generate_chunk(42, ChunkPos::new(-468_740, 41, 468_740)).is_empty());
    }

    #[test]
    fn ids_follow_the_convention() {
        let ids = [AIR, DIRT, ROCK, HARD_ROCK, ORE, GOLD, DIAMOND, LAVA];
        assert_eq!(ids, [0, 1, 2, 3, 4, 5, 6, 7]);
        for &(x, y, z) in &[(0i32, 0i32, 0i32), (0, -16, 0), (1, -100, 1)] {
            for &b in generate_chunk(42, ChunkPos::new(x, y, z)).as_bytes() {
                assert!(b <= LAVA, "id {b} en ({x},{y},{z})");
            }
        }
    }

    #[test]
    fn ores_show_up_but_stay_rare() {
        // Corteza (cy −15) y manto (cy −16) bajo el origen, 3×3 chunks por capa.
        let mut count = [0usize; 8];
        for cy in [-16i32, -15] {
            for cz in -1..=1 {
                for cx in -1..=1 {
                    for &b in generate_chunk(42, ChunkPos::new(cx, cy, cz)).as_bytes() {
                        count[b as usize] += 1;
                    }
                }
            }
        }
        assert!(count[ORE as usize] > 0 && count[GOLD as usize] > 0 && count[DIAMOND as usize] > 0);
        assert!(count[ORE as usize] > count[GOLD as usize], "mena ≤ oro");
        // Lava: cada chunk del manto es UNA celda de 16³ y la bolsa sale en
        // 1/64; se muestrean centros de celda y se confirma con el chunk.
        let pocket = (-40..=-17i32)
            .flat_map(|iy| (-10..10i32).map(move |iz| (iy, iz)))
            .flat_map(|(iy, iz)| {
                (-10..10i32).map(move |ix| [ix * 16 + 8, iy * 16 + 8, iz * 16 + 8])
            })
            .find(|&c| block_at(42, i64::from(c[0]), i64::from(c[1]), i64::from(c[2])) == LAVA);
        let c = pocket.expect("sin bolsas de lava en 9600 celdas del manto");
        let chunk = generate_chunk(42, ChunkPos::containing(c));
        assert!(
            chunk.as_bytes().contains(&LAVA),
            "bolsa {c:?} fuera del chunk"
        );
    }

    #[test]
    fn surface_soil_is_dirt_or_rock() {
        // Primera capa sólida: tierra fuera de la Cordillera en profundidad
        // 0 y nunca a profundidad ≥ 4.
        let mut dirt_tops = 0;
        for dz in (-24..=24i32).step_by(2) {
            for dx in (-24..=24i32).step_by(2) {
                let p = sample_column(42, dx, dz);
                let band = i64::from(p.surface_y) - BAND_HALF..=i64::from(p.surface_y) + BAND_HALF;
                for y in band.rev() {
                    let b = block_at(42, i64::from(dx), y, i64::from(dz));
                    if b == AIR {
                        continue;
                    }
                    let depth = i64::from(p.surface_y) - y;
                    if b == DIRT {
                        assert!(depth < 4, "tierra a profundidad {depth} ({dx},{y},{dz})");
                    }
                    if p.biome == SurfaceBiome::Cordillera {
                        assert_ne!(b, DIRT, "tierra en Cordillera ({dx},{y},{dz})");
                    } else if depth == 0 {
                        assert_eq!(b, DIRT, "suelo {b} en ({dx},{y},{dz})");
                        dirt_tops += 1;
                    }
                    break;
                }
            }
        }
        assert!(dirt_tops > 0, "ninguna columna asoma tierra en superficie");
    }
}
