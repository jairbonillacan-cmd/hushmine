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

    /// Construye un chunk a partir de sus 4096 bytes en orden de memoria.
    pub fn from_bytes(blocks: [BlockId; CHUNK_VOLUME]) -> Self {
        Self {
            blocks: Box::new(blocks),
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

    /// Codifica el chunk para enviarlo: un byte de modo y después los 4096
    /// bloques tal cual (`0`) o en tramos `(bloque, longitud - 1)` (`1`), lo
    /// que ocupe menos.
    pub fn encode(&self) -> Vec<u8> {
        let mut rle = vec![ENCODING_RLE];
        let mut iter = self.blocks.iter().copied().peekable();
        while let Some(block) = iter.next() {
            let mut run = 1usize;
            while run < 256 && iter.peek() == Some(&block) {
                iter.next();
                run += 1;
            }
            rle.extend_from_slice(&[block, (run - 1) as u8]);
        }
        if rle.len() < CHUNK_VOLUME + 1 {
            return rle;
        }
        let mut raw = Vec::with_capacity(CHUNK_VOLUME + 1);
        raw.push(ENCODING_RAW);
        raw.extend_from_slice(&self.blocks[..]);
        raw
    }

    /// Decodifica lo que produjo [`Chunk::encode`]. Nunca hace `panic!`.
    pub fn decode(data: &[u8]) -> Result<Self, ChunkDecodeError> {
        let (&mode, body) = data.split_first().ok_or(ChunkDecodeError)?;
        let mut chunk = Self::default();
        match mode {
            ENCODING_RAW if body.len() == CHUNK_VOLUME => chunk.blocks.copy_from_slice(body),
            ENCODING_RLE if body.len().is_multiple_of(2) => {
                let mut i = 0;
                for &[block, len] in body.as_chunks::<2>().0 {
                    let run = usize::from(len) + 1;
                    let end = i + run;
                    if end > CHUNK_VOLUME {
                        return Err(ChunkDecodeError);
                    }
                    chunk.blocks[i..end].fill(block);
                    i = end;
                }
                if i != CHUNK_VOLUME {
                    return Err(ChunkDecodeError);
                }
            }
            _ => return Err(ChunkDecodeError),
        }
        Ok(chunk)
    }
}

const ENCODING_RAW: u8 = 0;
const ENCODING_RLE: u8 = 1;

/// Los datos de un chunk recibido no son válidos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkDecodeError;

impl std::fmt::Display for ChunkDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "datos de chunk inválidos")
    }
}

impl std::error::Error for ChunkDecodeError {}

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
    fn encoding_round_trips_and_picks_the_smaller_form() {
        let empty = Chunk::default();
        let bytes = empty.encode();
        assert_eq!(bytes.len(), 1 + 2 * 16);
        assert_eq!(Chunk::decode(&bytes), Ok(empty));

        let mut noisy = Chunk::default();
        for i in 0..CHUNK_VOLUME {
            noisy.blocks[i] = (i * 7 % 5) as u8;
        }
        let bytes = noisy.encode();
        assert_eq!(bytes.len(), 1 + CHUNK_VOLUME);
        assert_eq!(Chunk::decode(&bytes), Ok(noisy));
    }

    #[test]
    fn corrupt_encodings_are_errors() {
        assert!(Chunk::decode(&[]).is_err());
        assert!(Chunk::decode(&[0, 1, 2]).is_err());
        assert!(Chunk::decode(&[1, 5]).is_err());
        assert!(Chunk::decode(&[1, 5, 255]).is_err());
        assert!(Chunk::decode(&[9]).is_err());
        let too_long: Vec<u8> = std::iter::once(1).chain([3, 255].repeat(17)).collect();
        assert!(Chunk::decode(&too_long).is_err());
    }

    #[test]
    fn layers_are_contiguous() {
        assert_eq!(Chunk::index(1, 0, 0), 1);
        assert_eq!(Chunk::index(0, 0, 1), 16);
        assert_eq!(Chunk::index(0, 1, 0), 256);
    }
}
