//! Mundo abierto: chunks cargados bajo demanda alrededor de un punto, en
//! cualquier coordenada (también negativas), con un borde opcional.
//!
//! Es la base del mundo grande (W0). La generación, el guardado y la carga
//! en hilos llegarán en W1 y W2; aquí solo está el almacenamiento.

use std::collections::{BTreeMap, BTreeSet};

use crate::{AIR, BlockId, CHUNK_SIZE, Chunk, ChunkPos, ChunkedVoxels, Voxels, VoxelsMut};

#[derive(Clone, Debug, Default)]
pub struct SparseWorld {
    chunks: BTreeMap<ChunkPos, Chunk>,
    dirty: BTreeSet<ChunkPos>,
    /// Los bloques con `|x|` o `|z|` mayor o igual que este valor forman
    /// una pared indestructible e invisible para la colisión.
    border: Option<i32>,
}

impl SparseWorld {
    /// Mundo sin borde.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mundo con un borde horizontal en `±border` bloques.
    pub fn with_border(border: i32) -> Self {
        Self {
            border: Some(border),
            ..Self::default()
        }
    }

    pub fn border(&self) -> Option<i32> {
        self.border
    }

    /// ¿Está el bloque fuera del borde del mundo?
    pub fn beyond_border(&self, x: i32, z: i32) -> bool {
        self.border
            .is_some_and(|b| x >= b || x < -b || z >= b || z < -b)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// Carga (o sustituye) un chunk y marca como sucios él y sus vecinos.
    pub fn insert_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
        self.mark_neighbours(pos);
    }

    /// Descarga un chunk. Se marca como sucio para que el render quite su
    /// malla, igual que sus vecinos (sus caras de borde cambian).
    pub fn remove_chunk(&mut self, pos: ChunkPos) -> Option<Chunk> {
        let chunk = self.chunks.remove(&pos)?;
        self.mark_neighbours(pos);
        Some(chunk)
    }

    /// Genera con `generate` los chunks que falten a `radius` (horizontal) y
    /// `vertical` chunks de `center`. Devuelve cuántos se cargaron.
    pub fn ensure_loaded(
        &mut self,
        center: ChunkPos,
        radius: i32,
        vertical: i32,
        mut generate: impl FnMut(ChunkPos) -> Chunk,
    ) -> usize {
        let mut loaded = 0;
        for dy in -vertical..=vertical {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let pos = center.offset(dx, dy, dz);
                    if !self.chunks.contains_key(&pos) {
                        let chunk = generate(pos);
                        self.insert_chunk(pos, chunk);
                        loaded += 1;
                    }
                }
            }
        }
        loaded
    }

    /// Descarga los chunks a más de `radius` (horizontal) o `vertical`
    /// chunks de `center`. Devuelve cuántos se descargaron.
    pub fn unload_outside(&mut self, center: ChunkPos, radius: i32, vertical: i32) -> usize {
        let far: Vec<ChunkPos> = self
            .chunks
            .keys()
            .filter(|p| {
                (p.x - center.x).abs() > radius
                    || (p.z - center.z).abs() > radius
                    || (p.y - center.y).abs() > vertical
            })
            .copied()
            .collect();
        for pos in &far {
            self.remove_chunk(*pos);
        }
        far.len()
    }

    fn mark_neighbours(&mut self, pos: ChunkPos) {
        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    self.dirty.insert(pos.offset(dx, dy, dz));
                }
            }
        }
    }
}

impl Voxels for SparseWorld {
    fn get(&self, x: i32, y: i32, z: i32) -> BlockId {
        let block = [x, y, z];
        match self.chunks.get(&ChunkPos::containing(block)) {
            Some(chunk) => {
                let [lx, ly, lz] = ChunkPos::local(block);
                chunk.get(lx, ly, lz)
            }
            None => AIR,
        }
    }

    fn blocks_movement(&self, x: i32, y: i32, z: i32) -> bool {
        self.beyond_border(x, z) || self.is_solid(x, y, z)
    }

    fn chunk_is_empty(&self, chunk: ChunkPos) -> bool {
        self.chunks.get(&chunk).is_none_or(Chunk::is_empty)
    }
}

impl VoxelsMut for SparseWorld {
    fn set(&mut self, x: i32, y: i32, z: i32, block: BlockId) -> Option<BlockId> {
        if self.beyond_border(x, z) {
            return None;
        }
        let pos = ChunkPos::containing([x, y, z]);
        let [lx, ly, lz] = ChunkPos::local([x, y, z]);
        let chunk = self.chunks.entry(pos).or_default();
        let old = chunk.get(lx, ly, lz);
        if old != block {
            chunk.set(lx, ly, lz, block);
            // Igual que en `VoxelWorld`: el chunk y los vecinos que tocan la
            // celda (oclusión ambiental y luz leen los vecinos).
            let edge = |l: usize| match l {
                0 => -1..=0,
                l if l == CHUNK_SIZE - 1 => 0..=1,
                _ => 0..=0,
            };
            for dy in edge(ly) {
                for dz in edge(lz) {
                    for dx in edge(lx) {
                        self.dirty.insert(pos.offset(dx, dy, dz));
                    }
                }
            }
        }
        Some(old)
    }
}

impl ChunkedVoxels for SparseWorld {
    fn loaded_chunks(&self) -> Vec<ChunkPos> {
        self.chunks.keys().copied().collect()
    }

    fn is_loaded(&self, chunk: ChunkPos) -> bool {
        self.chunks.contains_key(&chunk)
    }

    fn take_dirty_chunks(&mut self) -> Vec<ChunkPos> {
        std::mem::take(&mut self.dirty).into_iter().collect()
    }

    fn has_dirty_chunks(&self) -> bool {
        !self.dirty.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAR: i32 = 7_499_999;

    #[test]
    fn get_and_set_in_negative_and_extreme_coordinates() {
        let mut w = SparseWorld::new();
        for &(x, y, z) in &[
            (-1, -1, -1),
            (-16, 0, -17),
            (FAR, -4096, -FAR),
            (-FAR, 1023, FAR),
        ] {
            assert_eq!(w.get(x, y, z), AIR);
            assert_eq!(w.set(x, y, z, 5), Some(AIR));
            assert_eq!(w.get(x, y, z), 5, "({x}, {y}, {z})");
        }
        // Cada bloque cayó en su propio chunk.
        assert_eq!(w.len(), 4);
        // Los vecinos inmediatos siguen vacíos.
        assert_eq!(w.get(-2, -1, -1), AIR);
        assert_eq!(w.get(0, -1, -1), AIR);
    }

    #[test]
    fn edits_on_chunk_edges_dirty_the_neighbours() {
        let mut w = SparseWorld::new();
        w.set(-1, 5, 5, 1);
        let dirty = w.take_dirty_chunks();
        assert!(dirty.contains(&ChunkPos::new(-1, 0, 0)));
        assert!(dirty.contains(&ChunkPos::new(0, 0, 0)));
        assert_eq!(dirty.len(), 2);
        w.set(-8, 5, 5, 2);
        assert_eq!(w.take_dirty_chunks(), vec![ChunkPos::new(-1, 0, 0)]);
        assert!(!w.has_dirty_chunks());
    }

    #[test]
    fn border_is_a_wall_and_cannot_be_edited() {
        let mut w = SparseWorld::with_border(7_500_000);
        assert!(w.blocks_movement(7_500_000, 0, 0));
        assert!(w.blocks_movement(0, 0, -7_500_001));
        assert!(!w.blocks_movement(FAR, 0, -7_500_000));
        assert_eq!(w.set(7_500_000, 0, 0, 1), None);
        assert_eq!(w.set(FAR, 0, 0, 1), Some(AIR));
    }

    #[test]
    fn loads_and_unloads_around_a_moving_centre() {
        let mut w = SparseWorld::new();
        let solid = |_: ChunkPos| Chunk::filled(2);
        let a = ChunkPos::new(-468_750, -1, 468_749);
        assert_eq!(w.ensure_loaded(a, 2, 1, solid), 5 * 5 * 3);
        assert_eq!(w.ensure_loaded(a, 2, 1, solid), 0);
        let b = a.offset(1, 0, 0);
        assert_eq!(w.ensure_loaded(b, 2, 1, solid), 5 * 3);
        assert_eq!(w.unload_outside(b, 2, 1), 5 * 3);
        assert_eq!(w.len(), 5 * 5 * 3);
        assert!(w.is_solid(b.origin()[0], b.origin()[1], b.origin()[2]));
    }
}
