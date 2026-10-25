//! El generador del mundo de prueba: columna a columna, sin estado.
//!
//! [`generate_chunk`] no lee ni escribe nada fuera de sus parámetros: las
//! capas se calculan por columna (relieve) y por bloque (menas) a partir de
//! las coordenadas globales, de modo que el resultado no depende del orden
//! de generación ni de los chunks vecinos.

use grisu_voxel::{AIR, BlockId, CHUNK_SIZE, Chunk, ChunkPos};

use crate::{blocks, hash::hash3d, relief};

/// Sal de la capa de menas.
const ORE_SALT: u64 = 0x0E5E_0000_0000_0006;

/// Las menas salen de una sola tirada `hash3d % ORE_SPAN` por bloque: mena
/// en 16/4096 (1/256), oro en 4/4096 y diamante en 1/4096, cada vez más
/// raro y más hondo.
const ORE_SPAN: u64 = 4096;
const ORE_ROLL: u64 = 16;
const GOLD_ROLL: u64 = 16 + 4;
const DIAMOND_ROLL: u64 = 16 + 4 + 1;

/// Profundidad mínima bajo la superficie para el oro y el diamante.
const GOLD_MIN_DEPTH: i64 = 10;
const DIAMOND_MIN_DEPTH: i64 = 24;

/// Bloque del mundo de prueba en la posición global `(x, y, z)`.
///
/// Función pura de la semilla y las coordenadas: por encima de la
/// superficie hay aire, debajo tierra; luego roca (dura a partir de
/// [`relief::hard_rock_depth`]) con mena, oro y diamante por hash de
/// bloque, más frecuentes cuanto más hondo.
pub fn block_at(seed: u64, x: i64, y: i64, z: i64) -> BlockId {
    let surface = i64::from(relief::surface_height_at(seed, x, z));
    let depth = surface - y;
    column_block(seed, x, y, z, depth)
}

/// Bloque de una columna ya resuelta (superficie y profundidad conocidas).
#[inline]
fn column_block(seed: u64, x: i64, y: i64, z: i64, depth: i64) -> BlockId {
    if depth < 0 {
        return AIR;
    }
    if depth < relief::dirt_depth(seed, x, z) {
        return blocks::DIRT;
    }
    // Una sola tirada por bloque con rangos disjuntos: mena en toda la
    // roca, oro hondo y diamante más hondo. Si la tirada es de oro o
    // diamante pero la profundidad no llega, el bloque queda como roca.
    let roll = hash3d(x, y, z, seed ^ ORE_SALT) % ORE_SPAN;
    if roll < ORE_ROLL {
        blocks::ORE
    } else if roll < GOLD_ROLL && depth >= GOLD_MIN_DEPTH {
        blocks::GOLD
    } else if roll < DIAMOND_ROLL && depth >= DIAMOND_MIN_DEPTH {
        blocks::DIAMOND
    } else if depth >= relief::hard_rock_depth(seed, x, z) {
        blocks::HARD_ROCK
    } else {
        blocks::ROCK
    }
}

/// Genera el chunk `pos` del mundo de prueba con la semilla `seed`.
///
/// Función pura, sin estado compartido entre chunks (`INV-W01`): el mismo
/// par `(semilla, posición)` da siempre el mismo chunk byte a byte, en
/// cualquier orden y en cualquier plataforma. Las coordenadas globales se
/// calculan en `i64`, así que no hay desbordamientos ni en coordenadas
/// negativas ni en los extremos del mundo (`INV-W03`).
pub fn generate_chunk(seed: u64, pos: ChunkPos) -> Chunk {
    let mut chunk = Chunk::default();
    let origin = [
        i64::from(pos.x) * CHUNK_SIZE as i64,
        i64::from(pos.y) * CHUNK_SIZE as i64,
        i64::from(pos.z) * CHUNK_SIZE as i64,
    ];
    for z in 0..CHUNK_SIZE as i64 {
        for x in 0..CHUNK_SIZE as i64 {
            let (wx, wz) = (origin[0] + x, origin[2] + z);
            let surface = i64::from(relief::surface_height_at(seed, wx, wz));
            for y in 0..CHUNK_SIZE as i64 {
                let wy = origin[1] + y;
                let depth = surface - wy;
                // El aire es el valor por defecto: no hace falta escribirlo.
                let block = column_block(seed, wx, wy, wz, depth);
                if block != AIR {
                    chunk.set(x as usize, y as usize, z as usize, block);
                }
            }
        }
    }
    chunk
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_and_ores_are_where_they_should() {
        // Columna con superficie conocida: buscamos una con altura 0.
        let (mut x, mut z) = (0, 0);
        'outer: for gz in -8..8 {
            for gx in -8..8 {
                if relief::surface_height_at(42, i64::from(gx), i64::from(gz)) == 0 {
                    (x, z) = (gx, gz);
                    break 'outer;
                }
            }
        }
        assert_eq!(relief::surface_height_at(42, i64::from(x), i64::from(z)), 0);
        // Aire por encima, tierra en la superficie y roca debajo.
        assert_eq!(block_at(42, i64::from(x), 1, i64::from(z)), AIR);
        assert_eq!(block_at(42, i64::from(x), 0, i64::from(z)), blocks::DIRT);
        let dirt = relief::dirt_depth(42, i64::from(x), i64::from(z));
        let below_dirt = block_at(42, i64::from(x), -dirt, i64::from(z));
        assert!(
            below_dirt == blocks::ROCK || below_dirt == blocks::ORE,
            "bajo la tierra hay {below_dirt}"
        );
        // Muy hondo solo queda roca dura con menas.
        let deep = block_at(42, i64::from(x), -200, i64::from(z));
        assert!(
            deep == blocks::HARD_ROCK
                || deep == blocks::ORE
                || deep == blocks::GOLD
                || deep == blocks::DIAMOND,
            "a 200 bloques hay {deep}"
        );
    }

    #[test]
    fn ores_exist_but_are_rare() {
        // Cuenta sobre un bloque de 4×4×4 chunks bajo la superficie.
        let mut counts = std::collections::BTreeMap::new();
        for cy in -4..0 {
            for cz in -2..2 {
                for cx in -2..2 {
                    for &b in generate_chunk(42, ChunkPos::new(cx, cy, cz)).as_bytes() {
                        *counts.entry(b).or_insert(0usize) += 1;
                    }
                }
            }
        }
        let total: usize = counts.values().sum();
        let of = |block: BlockId| counts.get(&block).copied().unwrap_or(0);
        assert!(of(blocks::ORE) > 0, "sin mena en {total} bloques");
        assert!(of(blocks::GOLD) > 0, "sin oro en {total} bloques");
        assert!(of(blocks::DIAMOND) > 0, "sin diamante en {total} bloques");
        // La mena es bastante más abundante que el oro (que exige
        // profundidad) y este más que el diamante.
        assert!(of(blocks::ORE) > 3 * of(blocks::GOLD));
        assert!(of(blocks::GOLD) > 2 * of(blocks::DIAMOND));
    }
}
