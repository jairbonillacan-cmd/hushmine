//! HUSHMINE: generación del mundo de prueba del hito W0.
//!
//! [`generate_chunk`] es una función pura `(semilla, posición) → chunk`, sin
//! estado compartido entre chunks: todo el cálculo usa las coordenadas
//! globales de cada bloque, así que dos chunks vecinos siempre están de
//! acuerdo en la costura y el orden de generación no importa
//! (`INV-W02`). No hay `f32` ni `f64`: el relieve sale de un hash entero de
//! una rejilla interpolado en punto fijo Q16, igual en cualquier plataforma
//! (`INV-W01`).
//!
//! Este crate no depende de las reglas del juego: usa los identificadores de
//! `hush_sim::BlockKind` como constantes [`u8`] (módulo [`blocks`]) porque
//! para el motor un bloque es un `BlockId`. El mundo es de prueba y sirve
//! para ejercer el acceso por chunks en coordenadas negativas y extremas:
//! superficie cerca de `y = 0`, capas de tierra, roca y roca dura, y mena,
//! oro y diamante por hash de bloque.
//!
//! Las invariantes de [`docs/mundo/04-verificacion-e-hitos.md`] están en
//! `tests/`: determinismo (`INV-W01`), costuras (`INV-W02`) y coordenadas
//! extremas (`INV-W03`), más una medición de rendimiento ignorada.

mod blocks;
mod generate;
mod hash;
mod relief;

pub use blocks::{DIAMOND, DIRT, GOLD, HARD_ROCK, ORE, ROCK};
pub use generate::{block_at, generate_chunk};
pub use relief::{AMPLITUDE, CELL_SHIFT, surface_height};

use grisu_voxel::{ChunkPos, SparseWorld};

/// Carga con [`generate_chunk`] los chunks que falten a `radius` chunks
/// horizontales y `vertical` chunks verticales de `center`. Devuelve cuántos
/// se generaron.
///
/// ```
/// use grisu_voxel::{ChunkPos, SparseWorld};
/// use hush_worldgen::load_around;
///
/// let mut world = SparseWorld::new();
/// let cargados = load_around(&mut world, 42, ChunkPos::new(-468_750, -1, 468_749), 1, 1);
/// assert_eq!(cargados, 3 * 3 * 3);
/// assert_eq!(world.len(), 27);
/// ```
pub fn load_around(
    world: &mut SparseWorld,
    seed: u64,
    center: ChunkPos,
    radius: i32,
    vertical: i32,
) -> usize {
    world.ensure_loaded(center, radius, vertical, |pos| generate_chunk(seed, pos))
}

#[cfg(test)]
mod tests {
    use grisu_voxel::{ChunkPos, Voxels};

    use super::*;

    #[test]
    fn load_around_only_generates_missing_chunks() {
        let mut world = SparseWorld::new();
        let center = ChunkPos::new(-2, -1, 3);
        assert_eq!(load_around(&mut world, 7, center, 2, 1), 5 * 5 * 3);
        // Ya está todo: no se genera nada más.
        assert_eq!(load_around(&mut world, 7, center, 2, 1), 0);
        // Al moverse una columna solo faltan 15 chunks.
        assert_eq!(
            load_around(&mut world, 7, center.offset(1, 0, 0), 2, 1),
            5 * 3
        );
        // Y el chunk cargado es el mismo que generaría la función pura.
        let pos = ChunkPos::new(-2, -1, 3);
        assert_eq!(world.chunk(pos), Some(&generate_chunk(7, pos)));
        // La superficie quedó a la vista: a 8 bloques bajo el origen siempre
        // hay sólido (la superficie ronda y = 0) y en y = 15 nunca.
        assert!(world.is_solid(-32, -8, 48));
        assert!(!world.is_solid(-32, 15, 48));
    }
}
