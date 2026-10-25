//! HUSHMINE: generador determinista del mundo grande (W1).
//!
//! [`generate_chunk`] es una función pura `(semilla, posición) → chunk`
//! que une las tres fases del mundo grande sobre el ruido entero en Q16
//! de `grisu-noise` (fases de `docs/mundo/02-generacion.md`): columna
//! ([`sample_column`]: clima, superficie y bioma), densidad (crudeza con
//! salientes, Bóveda y lecho) y litología (el [`BlockId`] de cada celda
//! sólida, [`WORLDGEN_VERSION`]). Sin estado compartido entre chunks:
//! dos chunks vecinos siempre están de acuerdo en la costura y el orden
//! de generación no importa (`INV-W02`); sin `f32` ni reloj
//! (`INV-W01`). Cuevas, fluidos y estructuras llegarán en W2+.
//!
//! Este crate no depende de las reglas del juego: usa los identificadores
//! de la Convención A (`hush_sim::BlockKind`) duplicados como constantes
//! `u8` (módulo [`blocks`]) porque para el motor un bloque es un
//! `BlockId`.
//!
//! Las invariantes de `docs/mundo/04-verificacion-e-hitos.md` están en
//! `tests/`: determinismo (`INV-W01`), costuras (`INV-W02`) y coordenadas
//! extremas (`INV-W03`), más una medición de rendimiento ignorada.
//!
//! [`BlockId`]: grisu_voxel::BlockId

mod blocks;
mod column;
mod density;
mod generate;
mod lithology;
mod relief;

pub use blocks::{DIAMOND, DIRT, GOLD, HARD_ROCK, LAVA, ORE, ROCK};
// `MAX_COLUMN_STEP` se reexporta para que la cota documentada de la fase
// de columna sea API pública (y no quede muerta en el build sin pruebas).
pub use column::{ColumnParams, MAX_COLUMN_STEP, SurfaceBiome, sample_column};
pub use generate::{block_at, generate_chunk};
pub use lithology::WORLDGEN_VERSION;
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
    use grisu_voxel::{AIR, ChunkPos, Voxels};

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
        // Agnóstico del terreno: lo cargado matches_reference con la referencia
        // bloque a bloque (dentro y fuera de la roca) y el manto de esa
        // columna (y = −300) es sólido.
        assert_eq!(world.get(-32, -8, 48), block_at(7, -32, -8, 48));
        assert_eq!(world.get(-32, 15, 48), block_at(7, -32, 15, 48));
        assert_ne!(block_at(7, -32, -300, 48), AIR, "el manto no es sólido");
    }
}
