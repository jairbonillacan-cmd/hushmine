//! Interfaz de acceso al terreno.
//!
//! Mallas, raycast, colisión y render solo ven estos traits, no cómo se
//! guardan los bloques. Así el mapa finito del modo Turno ([`VoxelWorld`])
//! y el mundo abierto ([`SparseWorld`]) comparten todo el motor.
//!
//! [`VoxelWorld`]: crate::VoxelWorld
//! [`SparseWorld`]: crate::SparseWorld

use crate::{AIR, BlockId, ChunkPos};

/// Lectura de bloques en coordenadas del mundo (pueden ser negativas).
pub trait Voxels {
    /// Bloque en una posición; aire si no hay nada cargado ahí.
    fn get(&self, x: i32, y: i32, z: i32) -> BlockId;

    fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        self.get(x, y, z) != AIR
    }

    /// Solidez para la colisión. Por defecto, la de los bloques; un mundo
    /// puede añadir paredes invisibles en sus límites.
    fn blocks_movement(&self, x: i32, y: i32, z: i32) -> bool {
        self.is_solid(x, y, z)
    }

    /// ¿Es el chunk todo aire? Sirve para saltarse trabajo; por defecto no
    /// lo sabe.
    fn chunk_is_empty(&self, _chunk: ChunkPos) -> bool {
        false
    }
}

/// Escritura de bloques.
pub trait VoxelsMut: Voxels {
    /// Cambia un bloque y devuelve el anterior, o `None` si la posición no
    /// admite cambios (fuera de los límites del mundo).
    fn set(&mut self, x: i32, y: i32, z: i32, block: BlockId) -> Option<BlockId>;

    /// Rellena una caja (límites inclusivos).
    fn fill(&mut self, min: [i32; 3], max: [i32; 3], block: BlockId) {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                for x in min[0]..=max[0] {
                    self.set(x, y, z, block);
                }
            }
        }
    }
}

/// Mundo organizado en chunks, con seguimiento de los que hay que volver
/// a mallar.
pub trait ChunkedVoxels: VoxelsMut {
    /// Chunks cargados, en orden estable.
    fn loaded_chunks(&self) -> Vec<ChunkPos>;

    fn is_loaded(&self, chunk: ChunkPos) -> bool;

    /// Chunks modificados desde la última llamada (y los vecinos cuya malla
    /// depende de ellos). Los marca como limpios.
    fn take_dirty_chunks(&mut self) -> Vec<ChunkPos>;

    fn has_dirty_chunks(&self) -> bool;
}
