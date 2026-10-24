use crate::{AIR, BlockId, CHUNK_SIZE, Chunk};

/// Tamaño máximo de un mundo por eje, en bloques. Las caras empaquetadas
/// guardan cada coordenada en 7 bits.
pub const MAX_WORLD_SIZE: u32 = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// Cada eje debe ser múltiplo de 16 y estar entre 16 y 128.
    InvalidSize([u32; 3]),
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSize(size) => write!(
                f,
                "tamaño de mundo inválido {size:?}: cada eje debe ser múltiplo de \
                 {CHUNK_SIZE} y como máximo {MAX_WORLD_SIZE}"
            ),
        }
    }
}

impl std::error::Error for WorldError {}

/// Mundo finito formado por chunks de 16³.
///
/// Fuera de los límites todo es aire: así las caras exteriores del mapa se
/// dibujan y los rayos salen sin chocar. Las reglas del juego deciden si los
/// bordes se pueden cavar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoxelWorld {
    /// Tamaño en bloques.
    size: [u32; 3],
    /// Tamaño en chunks.
    chunk_dims: [usize; 3],
    chunks: Vec<Chunk>,
    /// Chunks cuya malla hay que regenerar.
    dirty: Vec<bool>,
}

impl VoxelWorld {
    /// Crea un mundo vacío. Todos los chunks empiezan marcados como sucios.
    pub fn new(size: [u32; 3]) -> Result<Self, WorldError> {
        let valid = size
            .iter()
            .all(|&s| s > 0 && s <= MAX_WORLD_SIZE && (s as usize).is_multiple_of(CHUNK_SIZE));
        if !valid {
            return Err(WorldError::InvalidSize(size));
        }
        let chunk_dims = size.map(|s| s as usize / CHUNK_SIZE);
        let count = chunk_dims.iter().product();
        Ok(Self {
            size,
            chunk_dims,
            chunks: vec![Chunk::default(); count],
            dirty: vec![true; count],
        })
    }

    pub fn size(&self) -> [u32; 3] {
        self.size
    }

    pub fn chunk_dims(&self) -> [usize; 3] {
        self.chunk_dims
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        let [sx, sy, sz] = self.size;
        x >= 0 && y >= 0 && z >= 0 && (x as u32) < sx && (y as u32) < sy && (z as u32) < sz
    }

    /// Índice de chunk a partir de coordenadas de chunk.
    pub fn chunk_index(&self, cx: usize, cy: usize, cz: usize) -> usize {
        let [dx, _, dz] = self.chunk_dims;
        cx + cz * dx + cy * dx * dz
    }

    /// Coordenadas de chunk a partir de su índice.
    pub fn chunk_coords(&self, index: usize) -> [usize; 3] {
        let [dx, _, dz] = self.chunk_dims;
        [index % dx, index / (dx * dz), (index / dx) % dz]
    }

    pub fn chunk(&self, index: usize) -> &Chunk {
        &self.chunks[index]
    }

    /// Sustituye un chunk entero (al recibir el mapa por red). Marca como
    /// sucios el chunk y sus vecinos. Devuelve `false` si el índice no existe.
    pub fn replace_chunk(&mut self, index: usize, chunk: Chunk) -> bool {
        if index >= self.chunks.len() {
            return false;
        }
        self.chunks[index] = chunk;
        // Un bloque de cada chunk vecino (caras, aristas y esquinas).
        let s = CHUNK_SIZE as i32;
        let [cx, cy, cz] = self.chunk_coords(index).map(|c| c as i32 * s);
        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    self.mark_dirty_at(cx + dx * s, cy + dy * s, cz + dz * s);
                }
            }
        }
        true
    }

    /// Bloque en una posición del mundo; aire fuera de los límites.
    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockId {
        if !self.in_bounds(x, y, z) {
            return AIR;
        }
        let (x, y, z) = (x as usize, y as usize, z as usize);
        let s = CHUNK_SIZE;
        let chunk = self.chunk_index(x / s, y / s, z / s);
        self.chunks[chunk].get(x % s, y % s, z % s)
    }

    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        self.get(x, y, z) != AIR
    }

    /// Solidez para la colisión: fuera del mundo, los lados y el fondo son
    /// paredes y solo el cielo está abierto. Así nadie sale del mapa.
    pub fn blocks_movement(&self, x: i32, y: i32, z: i32) -> bool {
        let [sx, _, sz] = self.size;
        let outside_sides = x < 0 || z < 0 || x as u32 >= sx || z as u32 >= sz;
        y < 0 || outside_sides || self.is_solid(x, y, z)
    }

    /// Cambia un bloque y marca como sucios su chunk y los vecinos que
    /// comparten la cara. Devuelve el bloque anterior, o `None` si la
    /// posición está fuera del mundo.
    pub fn set(&mut self, x: i32, y: i32, z: i32, block: BlockId) -> Option<BlockId> {
        if !self.in_bounds(x, y, z) {
            return None;
        }
        let (ux, uy, uz) = (x as usize, y as usize, z as usize);
        let s = CHUNK_SIZE;
        let chunk = self.chunk_index(ux / s, uy / s, uz / s);
        let old = self.chunks[chunk].get(ux % s, uy % s, uz % s);
        if old == block {
            return Some(old);
        }
        self.chunks[chunk].set(ux % s, uy % s, uz % s, block);
        self.mark_dirty_around(x, y, z);
        Some(old)
    }

    /// Marca como sucios los chunks cuya malla depende de la celda: el suyo y
    /// los que comparten cara, arista o esquina (la oclusión ambiental y la
    /// luz leen los vecinos).
    pub fn mark_dirty_around(&mut self, x: i32, y: i32, z: i32) {
        for dy in -1..=1 {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    self.mark_dirty_at(x + dx, y + dy, z + dz);
                }
            }
        }
    }

    /// ¿Hay algún chunk pendiente de regenerar?
    pub fn has_dirty(&self) -> bool {
        self.dirty.contains(&true)
    }

    fn mark_dirty_at(&mut self, x: i32, y: i32, z: i32) {
        if self.in_bounds(x, y, z) {
            let s = CHUNK_SIZE;
            let index = self.chunk_index(x as usize / s, y as usize / s, z as usize / s);
            self.dirty[index] = true;
        }
    }

    /// Rellena una caja (límites inclusivos y recortados al mundo).
    pub fn fill(&mut self, min: [i32; 3], max: [i32; 3], block: BlockId) {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                for x in min[0]..=max[0] {
                    self.set(x, y, z, block);
                }
            }
        }
    }

    /// Devuelve los índices de los chunks sucios y los marca como limpios.
    pub fn take_dirty(&mut self) -> Vec<usize> {
        let dirty = self
            .dirty
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| d.then_some(i))
            .collect();
        self.dirty.fill(false);
        dirty
    }

    pub fn mark_all_dirty(&mut self) {
        self.dirty.fill(true);
    }

    /// Huella FNV-1a de todos los bloques, para comparar mundos entre
    /// servidor, cliente y plataformas.
    pub fn content_hash(&self) -> u64 {
        let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
        for chunk in &self.chunks {
            for &b in chunk.as_bytes() {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3);
            }
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> VoxelWorld {
        VoxelWorld::new([64, 32, 48]).unwrap()
    }

    #[test]
    fn rejects_invalid_sizes() {
        assert!(VoxelWorld::new([0, 16, 16]).is_err());
        assert!(VoxelWorld::new([20, 16, 16]).is_err());
        assert!(VoxelWorld::new([144, 16, 16]).is_err());
        assert!(VoxelWorld::new([128, 64, 96]).is_ok());
    }

    #[test]
    fn get_set_and_out_of_bounds() {
        let mut w = world();
        assert_eq!(w.set(63, 31, 47, 5), Some(AIR));
        assert_eq!(w.get(63, 31, 47), 5);
        assert_eq!(w.set(64, 0, 0, 5), None);
        assert_eq!(w.get(-1, 0, 0), AIR);
        assert!(!w.is_solid(0, 0, 0));
    }

    #[test]
    fn chunk_coords_round_trip() {
        let w = world();
        assert_eq!(w.chunk_count(), 4 * 2 * 3);
        for i in 0..w.chunk_count() {
            let [x, y, z] = w.chunk_coords(i);
            assert_eq!(w.chunk_index(x, y, z), i);
        }
    }

    #[test]
    fn editing_marks_own_and_neighbour_chunks() {
        let mut w = world();
        w.take_dirty();
        // Interior de un chunk: solo ese chunk.
        w.set(8, 8, 8, 1);
        assert_eq!(w.take_dirty(), vec![w.chunk_index(0, 0, 0)]);
        // Borde x = 16: también el chunk de la izquierda.
        w.set(16, 8, 8, 1);
        let mut dirty = w.take_dirty();
        dirty.sort_unstable();
        assert_eq!(dirty, vec![w.chunk_index(0, 0, 0), w.chunk_index(1, 0, 0)]);
        // Sin cambio real, nada sucio.
        w.set(16, 8, 8, 1);
        assert!(w.take_dirty().is_empty());
    }

    #[test]
    fn hash_changes_with_content() {
        let mut w = world();
        let empty = w.content_hash();
        w.set(1, 2, 3, 4);
        assert_ne!(w.content_hash(), empty);
        w.set(1, 2, 3, AIR);
        assert_eq!(w.content_hash(), empty);
    }
}
