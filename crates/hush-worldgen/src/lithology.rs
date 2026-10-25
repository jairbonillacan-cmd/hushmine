//! Litología del mundo grande (W1, fase 3): el [`BlockId`] de cada celda
//! sólida. [`solid_block`] decide SOLO el tipo de roca (la solidez la decide
//! `crate::density`) y es una función pura de (semilla, coordenadas, columna
//! y pendiente). El orden de decisión importa y es exactamente este:
//!
//! 1. Lecho (`y ≤ BEDROCK_Y`): `HARD_ROCK` continuo e indestructible.
//! 2. `depth < 0`: `AIR` (defensivo: no debería llamarse con aire).
//! 3. Suelo de 1–4 capas según pendiente y bioma: `DIRT`, o `ROCK` en la
//!    Cordillera.
//! 4. Raíces (`y ≤ ROOTS_TOP`): `HARD_ROCK` con diamante y oro a 1/128.
//! 5. Bóveda (`y ≤ MANTO_BOTTOM`): `HARD_ROCK`.
//! 6. Manto (`y ≤ CORTEX_BOTTOM`): estratos de 64 bloques ondulados con
//!    bolsas de lava (`INV-W05`) y vetas de oro (1/256) y diamante (1/512).
//! 7. Corteza y roca sobre el nivel 0: `ROCK` con mena (1/64) y oro hondo
//!    (1/256 a partir de 12 bloques).
//!
//! Solo enteros y Q16, sin `f32` ni reloj (`INV-W01`, `INV-W03`).

use grisu_noise::{Noise2, hash2d, hash3d};
use grisu_voxel::{AIR, BlockId};

use crate::blocks::{DIAMOND, DIRT, GOLD, HARD_ROCK, LAVA, ORE, ROCK};
use crate::column::{ColumnParams, SurfaceBiome};
use crate::density::{BEDROCK_Y, MANTO_BOTTOM, ROOTS_TOP};

/// Suelo de la corteza: por debajo de −256 empieza el manto (cotas del
/// diseño en `docs/mundo/01-escala-y-motor.md`).
pub(crate) const CORTEX_BOTTOM: i64 = -256;

/// Versión del generador del mundo grande (`INV-W11`): congela los resúmenes
/// de los chunks de referencia. Cualquier cambio del generador —aunque sea
/// un umbral o una sal de este módulo— exige subir la versión.
pub const WORLDGEN_VERSION: u32 = 1;

/// Sal del desfase de estratos del manto.
const SALT_STRATA: u64 = 0x33E0_0000_0000_0001;
/// Sal de la tirada de mena de la corteza.
const SALT_ORE: u64 = 0x33E0_0000_0000_0002;
/// Sal de la tirada de oro del manto.
const SALT_GOLD: u64 = 0x33E0_0000_0000_0003;
/// Sal de la tirada de diamante de las Raíces.
const SALT_DIAMOND: u64 = 0x33E0_0000_0000_0004;
/// Sal de las bolsas de lava del manto.
const SALT_LAVA: u64 = 0x33E0_0000_0000_0005;
/// Sal del grosor de suelo.
const SALT_SOIL: u64 = 0x33E0_0000_0000_0006;

/// Desfase máximo de los estratos del manto, en bloques. La corteza usará 8
/// en W3, cuando haya rocas sedimentarias distinguibles.
const STRATA_AMP: i64 = 12;
/// Período vertical de los estratos del manto, en bloques.
const STRATA_PERIOD: i64 = 64;
/// Grosor de la banda de roca dura dentro del período: 40 de cada 64.
const STRATA_HARD: i64 = 40;
/// Tirada de las Raíces: diamante en 3/384 y oro en los 3/384 siguientes.
const ROOTS_SPAN: u64 = 384;
const ROOTS_DIAMOND: u64 = 3;
const ROOTS_GOLD: u64 = 6;
/// Tirada del manto: oro en 8/2048 y diamante en los 4/2048 siguientes.
const MANTLE_SPAN: u64 = 2048;
const MANTLE_GOLD: u64 = 8;
const MANTLE_DIAMOND: u64 = 12;
/// Bolsas de lava: 1 de cada 64 celdas de 16³ del manto.
const LAVA_SPAN: u64 = 64;
/// Tirada de la corteza: mena en 16/1024 y oro hondo en los 4 siguientes.
const CRUST_SPAN: u64 = 1024;
const CRUST_ORE: u64 = 16;
const CRUST_GOLD: u64 = 20;
/// Profundidad mínima bajo la superficie para el oro de la corteza.
const GOLD_MIN_DEPTH: i64 = 12;

/// Tipo de roca de la celda sólida `(x, y, z)`: la llama el generador (y
/// `block_at`) para celdas que `crate::density` ya ha declarado sólidas.
/// `surface_y` es la superficie de ESTA columna (la que manda para la
/// profundidad; `params` aporta el bioma) y `slope` es el máximo salto
/// `|Δh|` hacia las 4 columnas vecinas, calculado por el llamador con la
/// misma aritmética. Función pura: determinista por (semilla, coordenadas).
#[must_use]
pub(crate) fn solid_block(
    seed: u64,
    x: i64,
    y: i64,
    z: i64,
    params: &ColumnParams,
    surface_y: i32,
    slope: i32,
) -> BlockId {
    // Lecho: continuo e indestructible, sin tiradas encima.
    if y <= BEDROCK_Y {
        return HARD_ROCK;
    }
    // Profundidad bajo la superficie; por encima solo puede haber aire.
    let depth = i64::from(surface_y) - y;
    if depth < 0 {
        return AIR;
    }
    // Suelo: la Cordillera no acumula tierra.
    if depth < soil_depth(seed, x, z, slope) {
        return if params.biome == SurfaceBiome::Cordillera {
            ROCK
        } else {
            DIRT
        };
    }
    // Raíces: roca dura con diamante y oro equiprobables, rangos disjuntos
    // de una sola tirada.
    if y <= ROOTS_TOP {
        let roll = hash3d(x, y, z, seed ^ SALT_DIAMOND) % ROOTS_SPAN;
        if roll < ROOTS_DIAMOND {
            return DIAMOND;
        }
        if roll < ROOTS_GOLD {
            return GOLD;
        }
        return HARD_ROCK;
    }
    // Bóveda: roca dura limpia.
    if y <= MANTO_BOTTOM {
        return HARD_ROCK;
    }
    // Manto: estratos ondulados con bolsas de lava y vetas.
    if y <= CORTEX_BOTTOM {
        return mantle_block(seed, x, y, z);
    }
    // Corteza y roca sobre el nivel 0.
    crust_block(seed, x, y, z, depth)
}

/// Grosor del suelo de la columna `(x, z)`: 3–4 bloques en llano
/// (`slope ≤ 2`), 2 en ladera (`slope ≤ 5`) y 1 en paredón.
fn soil_depth(seed: u64, x: i64, z: i64, slope: i32) -> i64 {
    if slope <= 2 {
        3 + (hash2d(x, z, seed ^ SALT_SOIL) % 2) as i64
    } else if slope <= 5 {
        2
    } else {
        1
    }
}

/// Capa del estrato de período [`STRATA_PERIOD`] a la que cae `y` con el
/// desfase `off` (en bloques): resto euclídeo, siempre en `0..64`.
fn stratum_layer(y: i64, off: i64) -> i64 {
    (y + off).rem_euclid(STRATA_PERIOD)
}

/// Desfase vertical de los estratos del manto en la columna `(x, z)`, en
/// bloques dentro de ±[`STRATA_AMP`]: el ruido de gradiente de período 32
/// bloques está en Q16, así que el paso a bloques es `(ruido · amp) >> 16`.
/// Suave de columna a columna y con retícula global: sin costuras.
fn stratum_offset(seed: u64, x: i64, z: i64) -> i64 {
    (Noise2::new(seed, SALT_STRATA, 5).sample_block(x, z) * STRATA_AMP) >> 16
}

/// Interior de una bolsa de lava del manto (`INV-W05`): la celda de 16³
/// `(x >> 4, y >> 4, z >> 4)` es bolsa en 1/64 de los casos, pero solo
/// lleva `LAVA` su interior 12³ (coordenada local en `2..14`): el anillo
/// restante es roca sólida —también entre bolsas vecinas— y a esa
/// profundidad la densidad es siempre sólida.
fn lava_pocket(seed: u64, x: i64, y: i64, z: i64) -> bool {
    if !hash3d(x >> 4, y >> 4, z >> 4, seed ^ SALT_LAVA).is_multiple_of(LAVA_SPAN) {
        return false;
    }
    let (lx, ly, lz) = (x & 15, y & 15, z & 15);
    (2..14).contains(&lx) && (2..14).contains(&ly) && (2..14).contains(&lz)
}

/// Bloque del manto: la lava manda sobre las vetas y las vetas sobre los
/// estratos (una sola tirada con rangos disjuntos: oro 8/2048, diamante
/// 4/2048).
fn mantle_block(seed: u64, x: i64, y: i64, z: i64) -> BlockId {
    if lava_pocket(seed, x, y, z) {
        return LAVA;
    }
    let roll = hash3d(x, y, z, seed ^ SALT_GOLD) % MANTLE_SPAN;
    if roll < MANTLE_GOLD {
        GOLD
    } else if roll < MANTLE_DIAMOND {
        DIAMOND
    } else if stratum_layer(y, stratum_offset(seed, x, z)) < STRATA_HARD {
        HARD_ROCK
    } else {
        ROCK
    }
}

/// Bloque de la corteza y de la roca sobre el nivel 0 bajo la superficie:
/// `ROCK` con mena (16/1024) y oro solo hondo (4/1024 desde 12 bloques). W1
/// no diferencia aún las rocas sedimentarias de la corteza; llegarán en W3
/// con su desfase de estratos de ±8.
fn crust_block(seed: u64, x: i64, y: i64, z: i64, depth: i64) -> BlockId {
    let roll = hash3d(x, y, z, seed ^ SALT_ORE) % CRUST_SPAN;
    if roll < CRUST_ORE {
        ORE
    } else if roll < CRUST_GOLD && depth >= GOLD_MIN_DEPTH {
        GOLD
    } else {
        ROCK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::WATER_LEVEL;

    /// Semilla fija de las pruebas del módulo.
    const SEED: u64 = 42;

    /// `ColumnParams` de prueba: aquí solo importan el bioma y la superficie.
    fn params(biome: SurfaceBiome, surface_y: i32) -> ColumnParams {
        ColumnParams {
            continentality: 0,
            erosion: 0,
            peaks_valleys: 0,
            temperature: 0,
            humidity: 0,
            surface_y,
            water_level: WATER_LEVEL,
            biome,
        }
    }

    #[test]
    fn bedrock_and_vault_are_always_hard_rock() {
        let p = params(SurfaceBiome::Tundra, 0);
        for seed in [0, 7, 42, u64::MAX] {
            // −1000 y −3000 caen en la Bóveda; el resto, en el lecho.
            for y in [BEDROCK_Y - 1, BEDROCK_Y, -5000, -8192, -1000, -3000] {
                for (x, z) in [(0, 0), (-17, 33), (7_499_999, -7_499_999)] {
                    assert_eq!(solid_block(seed, x, y, z, &p, 0, 0), HARD_ROCK);
                }
            }
        }
    }

    #[test]
    fn mantle_carries_gold_and_diamond() {
        let p = params(SurfaceBiome::Highland, 64);
        let (mut gold, mut diamond) = (0usize, 0usize);
        for x in 0..24i64 {
            for z in 0..24i64 {
                for y in (-639..=-256i64).step_by(2) {
                    match solid_block(SEED, x, y, z, &p, 64, 0) {
                        GOLD => gold += 1,
                        DIAMOND => diamond += 1,
                        _ => {}
                    }
                }
            }
        }
        assert!(gold > 0, "sin oro en el manto");
        assert!(diamond > 0, "sin diamante en el manto");
        assert!(gold > diamond, "oro {gold} ≤ diamante {diamond}");
    }

    #[test]
    fn lava_pockets_stay_inside_their_cell() {
        let p = params(SurfaceBiome::Highland, 64);
        // Celdas de 16³ con su interior completo en el manto: cy de −40
        // (interior −638..−627) a −17 (interior −270..−259).
        let mut pockets = 0;
        'celdas: for cx in -6..=6i64 {
            for cz in -6..=6i64 {
                for cy in -40..=-17i64 {
                    if !hash3d(cx, cy, cz, SEED ^ SALT_LAVA).is_multiple_of(LAVA_SPAN) {
                        continue;
                    }
                    pockets += 1;
                    for lx in 0..16i64 {
                        for ly in 0..16i64 {
                            for lz in 0..16i64 {
                                let (x, y, z) = ((cx << 4) + lx, (cy << 4) + ly, (cz << 4) + lz);
                                let b = solid_block(SEED, x, y, z, &p, 64, 0);
                                let interior = (2..14).contains(&lx)
                                    && (2..14).contains(&ly)
                                    && (2..14).contains(&lz);
                                assert_eq!(
                                    b == LAVA,
                                    interior,
                                    "({x},{y},{z}) lava fuera del interior de su celda"
                                );
                            }
                        }
                    }
                    if pockets >= 4 {
                        break 'celdas;
                    }
                }
            }
        }
        // 13·13·24 = 4056 celdas a 1/64: siempre hay bolsas (esperadas ≈ 63).
        assert!(pockets >= 1, "sin bolsas de lava en la muestra");
    }

    #[test]
    fn ore_outweighs_gold_in_crust_and_mantle() {
        let p = params(SurfaceBiome::Highland, 64);
        // Un sondeo por corteza y manto; la superficie 64 deja todo bajo
        // tierra y con profundidad ≥ 12. La mena solo sale en la corteza,
        // el oro en las dos; el oro del manto no llega a superarla.
        let (mut ore, mut gold) = (0usize, 0usize);
        for x in 0..24i64 {
            for z in 0..24i64 {
                for y in (-639..=-6i64).step_by(2) {
                    match solid_block(SEED, x, y, z, &p, 64, 0) {
                        ORE => ore += 1,
                        GOLD => gold += 1,
                        _ => {}
                    }
                }
            }
        }
        assert!(ore > gold, "mena {ore} ≤ oro {gold}");
    }

    #[test]
    fn soil_depends_on_biome_and_slope() {
        let forest = params(SurfaceBiome::NeedleForest, 32);
        // Llano: tierra en las primeras capas y roca (o mena) debajo.
        for depth in 0..3i64 {
            assert_eq!(solid_block(SEED, 8, 32 - depth, 8, &forest, 32, 1), DIRT);
        }
        let deep = solid_block(SEED, 8, 24, 8, &forest, 32, 1);
        assert!(deep == ROCK || deep == ORE, "bajo el suelo hay {deep}");
        // La Cordillera no acumula tierra.
        let range = params(SurfaceBiome::Cordillera, 32);
        assert_eq!(solid_block(SEED, 8, 32, 8, &range, 32, 1), ROCK);
        assert_eq!(solid_block(SEED, 8, 31, 8, &range, 32, 1), ROCK);
        // Pendiente alta: el suelo se afina a 1–2 capas.
        for slope in [3i32, 6, 40] {
            let p = params(SurfaceBiome::Highland, 32);
            assert_eq!(solid_block(SEED, 8, 32, 8, &p, 32, slope), DIRT);
            let below_soil = 32 - soil_depth(SEED, 8, 8, slope);
            assert_ne!(solid_block(SEED, 8, below_soil, 8, &p, 32, slope), DIRT);
        }
        // Roca sobre el nivel 0 (el interior de las montañas).
        let peak = params(SurfaceBiome::Highland, 100);
        assert_eq!(solid_block(SEED, 8, 100, 8, &peak, 100, 0), DIRT);
        let high = solid_block(SEED, 8, 94, 8, &peak, 100, 0);
        assert!(high == ROCK || high == ORE, "y = 94 da {high}");
        // El grosor por tramos de pendiente, directo.
        for x in 0..16i64 {
            for z in 0..16i64 {
                assert!((3..=4).contains(&soil_depth(SEED, x, z, 2)));
                assert_eq!(soil_depth(SEED, x, z, 5), 2);
                assert_eq!(soil_depth(SEED, x, z, 6), 1);
            }
        }
    }

    #[test]
    fn roots_are_hard_rock_with_diamond_and_gold() {
        let p = params(SurfaceBiome::Tundra, 64);
        let (mut diamond, mut gold) = (0usize, 0usize);
        for x in 0..12i64 {
            for z in 0..12i64 {
                for y in (-4000..=-3073i64).step_by(3) {
                    match solid_block(SEED, x, y, z, &p, 64, 0) {
                        DIAMOND => diamond += 1,
                        GOLD => gold += 1,
                        HARD_ROCK => {}
                        b => panic!("en las Raíces hay {b}"),
                    }
                }
            }
        }
        assert!(diamond > 0, "sin diamante en las Raíces");
        assert!(gold > 0, "sin oro en las Raíces");
    }

    #[test]
    fn is_deterministic_and_seed_moves_veins() {
        let p = params(SurfaceBiome::Highland, 64);
        for &(x, y, z) in &[
            (0i64, 0, 0),
            (17, -300, -5),
            (-100, -639, 100),
            (5, -4095, 9),
        ] {
            let a = solid_block(SEED, x, y, z, &p, 64, 0);
            assert_eq!(a, solid_block(SEED, x, y, z, &p, 64, 0), "({x},{y},{z})");
        }
        // Otra semilla remueve las vetas del manto.
        let mut moved = 0;
        for x in 0..8i64 {
            for z in 0..8i64 {
                for y in (-639..=-256i64).step_by(4) {
                    if solid_block(SEED, x, y, z, &p, 64, 0) != solid_block(43, x, y, z, &p, 64, 0)
                    {
                        moved += 1;
                    }
                }
            }
        }
        assert!(moved > 0, "la semilla no cambia ninguna veta");
        // Guardia de la versión congelada (`INV-W11`).
        assert_eq!(WORLDGEN_VERSION, 1);
    }
}
