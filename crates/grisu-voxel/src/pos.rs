//! Posiciones de bloque y de chunk, con coordenadas negativas.
//!
//! Los bloques usan `i32` por eje (±2 147 483 647, de sobra para un mundo
//! de 15 M × 15 M). El chunk se obtiene con un desplazamiento aritmético,
//! que redondea hacia −∞: el bloque −1 está en el chunk −1, posición 15.

use crate::CHUNK_SIZE;

/// Bits de la posición local dentro de un chunk.
pub const CHUNK_SHIFT: u32 = 4;
const LOCAL_MASK: i32 = CHUNK_SIZE as i32 - 1;

/// Posición de un chunk de 16³ (coordenadas de bloque `>> 4`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Chunk que contiene el bloque `block`.
    pub const fn containing(block: [i32; 3]) -> Self {
        Self {
            x: block[0] >> CHUNK_SHIFT,
            y: block[1] >> CHUNK_SHIFT,
            z: block[2] >> CHUNK_SHIFT,
        }
    }

    /// Primer bloque (esquina mínima) del chunk.
    pub const fn origin(self) -> [i32; 3] {
        [
            self.x << CHUNK_SHIFT,
            self.y << CHUNK_SHIFT,
            self.z << CHUNK_SHIFT,
        ]
    }

    /// Posición de un bloque dentro de su chunk (`0..16` por eje).
    pub const fn local(block: [i32; 3]) -> [usize; 3] {
        [
            (block[0] & LOCAL_MASK) as usize,
            (block[1] & LOCAL_MASK) as usize,
            (block[2] & LOCAL_MASK) as usize,
        ]
    }

    pub const fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    pub const fn to_array(self) -> [i32; 3] {
        [self.x, self.y, self.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_blocks_round_down() {
        for (block, chunk, local) in [
            (0, 0, 0),
            (15, 0, 15),
            (16, 1, 0),
            (-1, -1, 15),
            (-16, -1, 0),
            (-17, -2, 15),
            (7_499_999, 468_749, 15),
            (-7_500_000, -468_750, 0),
        ] {
            let b = [block, block, block];
            let pos = ChunkPos::containing(b);
            assert_eq!(pos, ChunkPos::new(chunk, chunk, chunk), "bloque {block}");
            assert_eq!(ChunkPos::local(b), [local; 3], "bloque {block}");
            assert_eq!(pos.origin()[0] + local as i32, block);
        }
    }

    #[test]
    fn origin_and_containing_are_inverse() {
        for c in [-468_750, -2, -1, 0, 1, 468_749] {
            let pos = ChunkPos::new(c, -c, c);
            assert_eq!(ChunkPos::containing(pos.origin()), pos);
        }
    }
}
