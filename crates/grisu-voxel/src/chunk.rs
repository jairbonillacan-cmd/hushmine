use crate::{AIR, BlockId};

/// Lado de un chunk, en bloques.
pub const CHUNK_SIZE: usize = 16;
/// Bloques por chunk.
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Cubo de 16³ bloques, un byte por bloque (4 KB).
///
/// Orden en memoria: `x` varía más rápido, luego `z` y luego `y`, para que
/// una capa horizontal sea contigua.
#[derive(Clone, PartialEq, Eq)]
pub struct Chunk {
    blocks: Box<[BlockId; CHUNK_VOLUME]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::filled(AIR)
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let solid = self.blocks.iter().filter(|&&b| b != AIR).count();
        write!(f, "Chunk({solid} bloques sólidos)")
    }
}

impl Chunk {
    pub fn filled(block: BlockId) -> Self {
        Self {
            blocks: Box::new([block; CHUNK_VOLUME]),
        }
    }

    /// Índice lineal de una posición local. Las coordenadas deben ser < 16.
    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.blocks[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        self.blocks[Self::index(x, y, z)] = block;
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&b| b == AIR)
    }

    pub fn as_bytes(&self) -> &[BlockId] {
        &self.blocks[..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_is_4_kb_and_round_trips() {
        let mut chunk = Chunk::default();
        assert_eq!(chunk.as_bytes().len(), 4096);
        assert!(chunk.is_empty());
        chunk.set(15, 3, 7, 9);
        assert_eq!(chunk.get(15, 3, 7), 9);
        assert_eq!(chunk.get(3, 15, 7), AIR);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn layers_are_contiguous() {
        assert_eq!(Chunk::index(1, 0, 0), 1);
        assert_eq!(Chunk::index(0, 0, 1), 16);
        assert_eq!(Chunk::index(0, 1, 0), 256);
    }
}
