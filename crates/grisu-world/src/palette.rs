//! Paleta de bloques para el mundo grande.
//!
//! Sección 3 de `docs/mundo/01-escala-y-motor.md`: los chunks siguen siendo
//! de 16³ bloques, pero sus ids son de 16 bits y se guardan en la
//! representación más pequeña posible: un `Uniform` de 2 bytes cuando todo el
//! chunk es el mismo bloque, una `Palette` de 1/2/4/8 bits por posición
//! mientras haya 256 bloques distintos o menos, y los 4096 ids directos
//! (8 KB) cuando la paleta ya no cabe.

use grisu_voxel::{AIR, BlockId, CHUNK_SIZE, CHUNK_VOLUME, Chunk};

/// Id de bloque ancho para el mundo grande (0..=17 reservados a la Convención A).
pub type WideBlockId = u16;

/// Entradas que admite una paleta de 8 bits.
const MAX_PALETTE_ENTRIES: usize = 256;

/// Chunk de 16³ con paleta (sección 3 de docs/mundo/01-escala-y-motor.md).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PalettedChunk {
    /// Todo el chunk es un mismo bloque: 2 bytes.
    Uniform(WideBlockId),
    /// Paleta de 1, 2, 4 u 8 bits según el número de bloques distintos (2..=256).
    ///
    /// Los índices de paleta se empaquetan LSB-first en `packed`. Como `bits`
    /// divide a 64, un índice nunca cruza una palabra: con
    /// `per_word = 64 / bits`, el valor de la posición lineal `i` (orden de
    /// [`Chunk::index`]) vive en `packed[i / per_word]` con desplazamiento
    /// `(i % per_word) * bits`, y `packed.len()` es exactamente
    /// `CHUNK_VOLUME / per_word` (64, 128, 256 o 512 palabras). `entries`
    /// está en orden de primera aparición y mide de 2 a `2^bits`.
    Palette {
        bits: u8,
        entries: Vec<WideBlockId>,
        packed: Vec<u64>,
    },
    /// Más de 256 bloques distintos: 4096 ids directos (8 KB).
    Direct(Box<[WideBlockId; CHUNK_VOLUME]>),
}

impl PalettedChunk {
    /// Chunk enteramente de un mismo bloque (variante `Uniform`).
    pub fn filled(id: WideBlockId) -> Self {
        PalettedChunk::Uniform(id)
    }

    /// Id del bloque en `(x, y, z)`.
    ///
    /// Las coordenadas fuera de `0..16` no están en este chunk: se leen
    /// como 0 (aire) en lugar de entrar en pánico.
    pub fn get(&self, x: usize, y: usize, z: usize) -> WideBlockId {
        match linear_index(x, y, z) {
            Some(i) => self.get_linear(i),
            None => 0,
        }
    }

    /// Escribe `id` en `(x, y, z)` promocionando la representación al usarla.
    ///
    /// - `Uniform(a)` con `id == a` no hace nada; con otro id pasa a una
    ///   `Palette` de 1 bit con índice 0 para `a` y 1 para `id`.
    /// - `Palette`: si `id` ya está en `entries` escribe su índice; si no
    ///   está y queda hueco, lo añade y escribe el nuevo índice; si la
    ///   paleta está llena, reconstruye el chunk completo (sube `bits` de
    ///   1→2→4→8 o pasa a `Direct` cuando supera 256 distintos).
    /// - `Direct`: escritura directa.
    ///
    /// Las coordenadas fuera de `0..16` se ignoran.
    pub fn set(&mut self, x: usize, y: usize, z: usize, id: WideBlockId) {
        let Some(i) = linear_index(x, y, z) else {
            return;
        };
        match self {
            PalettedChunk::Uniform(a) if *a == id => {}
            PalettedChunk::Uniform(a) => {
                // Todo el chunk era `a` (índice 0); el id nuevo es el índice 1.
                let mut packed = vec![0u64; CHUNK_VOLUME / per_word(1)];
                write_packed(&mut packed, 1, i, 1);
                *self = PalettedChunk::Palette {
                    bits: 1,
                    entries: vec![*a, id],
                    packed,
                };
            }
            PalettedChunk::Palette {
                bits,
                entries,
                packed,
            } => {
                let found = entries.iter().position(|&e| e == id);
                let fits = entries.len() < capacity(*bits);
                match (found, fits) {
                    (Some(index), _) => write_packed(packed, *bits, i, index),
                    (None, true) => {
                        entries.push(id);
                        write_packed(packed, *bits, i, entries.len() - 1);
                    }
                    (None, false) => {
                        // Paleta llena: volcar, escribir y reconstruir al mínimo.
                        let mut ids = self.linear_ids();
                        if let Some(slot) = ids.get_mut(i) {
                            *slot = id;
                        }
                        *self = PalettedChunk::from_linear_ids(&ids);
                    }
                }
            }
            PalettedChunk::Direct(ids) => {
                if let Some(slot) = ids.get_mut(i) {
                    *slot = id;
                }
            }
        }
    }

    /// Recalcula los ids distintos y reduce a la representación mínima:
    /// `Uniform` con uno, `Palette` con 2..=256 y `Direct` a partir de 257.
    /// Una `Uniform` ya es mínima y no cambia.
    pub fn compact(&mut self) {
        if !matches!(self, PalettedChunk::Uniform(_)) {
            let ids = self.linear_ids();
            *self = PalettedChunk::from_linear_ids(&ids);
        }
    }

    /// Representación mínima de un chunk de `grisu-voxel`.
    ///
    /// Los ids de origen son `u8`, así que nunca necesita `Direct`.
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let ids: Vec<WideBlockId> = chunk
            .as_bytes()
            .iter()
            .copied()
            .map(WideBlockId::from)
            .collect();
        PalettedChunk::from_linear_ids(&ids)
    }

    /// Convierte a un [`Chunk`] de `grisu-voxel` (un byte por bloque).
    ///
    /// Es con pérdida: los ids mayores que 255 no caben en un `u8` y quedan
    /// como `AIR`. Las celdas que quedan en aire no se escriben, porque el
    /// chunk recién creado ya está todo a aire.
    pub fn to_chunk_lossy(&self) -> Chunk {
        let mut chunk = Chunk::default();
        for i in 0..CHUNK_VOLUME {
            let id = self.get_linear(i);
            let narrow: BlockId = if id <= WideBlockId::from(u8::MAX) {
                id as BlockId
            } else {
                AIR
            };
            if narrow != AIR {
                let x = i % CHUNK_SIZE;
                let z = (i / CHUNK_SIZE) % CHUNK_SIZE;
                let y = i / (CHUNK_SIZE * CHUNK_SIZE);
                chunk.set(x, y, z, narrow);
            }
        }
        chunk
    }

    /// Id en la posición lineal `i` (orden de `Chunk::index`).
    fn get_linear(&self, i: usize) -> WideBlockId {
        match self {
            PalettedChunk::Uniform(id) => *id,
            PalettedChunk::Palette {
                bits,
                entries,
                packed,
            } => {
                let index = read_packed(packed, *bits, i);
                entries.get(index).copied().unwrap_or(0)
            }
            PalettedChunk::Direct(ids) => ids.get(i).copied().unwrap_or(0),
        }
    }

    /// Vuelca los 4096 ids a un array lineal (orden de `Chunk::index`).
    fn linear_ids(&self) -> [WideBlockId; CHUNK_VOLUME] {
        match self {
            PalettedChunk::Uniform(id) => [*id; CHUNK_VOLUME],
            PalettedChunk::Direct(ids) => **ids,
            PalettedChunk::Palette { .. } => {
                let mut ids = [0; CHUNK_VOLUME];
                for (i, slot) in ids.iter_mut().enumerate() {
                    *slot = self.get_linear(i);
                }
                ids
            }
        }
    }

    /// Reconstruye la representación mínima desde ids lineales, con
    /// `entries` en orden de primera aparición.
    fn from_linear_ids(ids: &[WideBlockId]) -> Self {
        let mut entries: Vec<WideBlockId> = Vec::new();
        let mut indices: Vec<usize> = Vec::with_capacity(ids.len());
        for &id in ids {
            let index = match entries.iter().position(|&e| e == id) {
                Some(existing) => existing,
                None => {
                    entries.push(id);
                    entries.len() - 1
                }
            };
            indices.push(index);
        }
        if let [only] = entries.as_slice() {
            return PalettedChunk::Uniform(*only);
        }
        if entries.len() > MAX_PALETTE_ENTRIES {
            let mut direct = [0; CHUNK_VOLUME];
            for (slot, &id) in direct.iter_mut().zip(ids.iter()) {
                *slot = id;
            }
            return PalettedChunk::Direct(Box::new(direct));
        }
        let bits = bits_for(entries.len());
        let mut packed = vec![0u64; CHUNK_VOLUME / per_word(bits)];
        for (i, &index) in indices.iter().enumerate() {
            write_packed(&mut packed, bits, i, index);
        }
        PalettedChunk::Palette {
            bits,
            entries,
            packed,
        }
    }
}

/// Posición lineal validada: `Some(Chunk::index(x, y, z))` cuando las tres
/// coordenadas están en `0..16` y `None` cuando no.
fn linear_index(x: usize, y: usize, z: usize) -> Option<usize> {
    if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
        Some(Chunk::index(x, y, z))
    } else {
        None
    }
}

/// Índices de paleta que caben en una palabra de 64 bits.
fn per_word(bits: u8) -> usize {
    64 / usize::from(bits)
}

/// Entradas que admite una paleta de `bits`: `2^bits`.
fn capacity(bits: u8) -> usize {
    1usize << bits
}

/// Bits mínimos (1, 2, 4 u 8) para `distinct` bloques distintos (2..=256).
fn bits_for(distinct: usize) -> u8 {
    if distinct <= 2 {
        1
    } else if distinct <= 4 {
        2
    } else if distinct <= 16 {
        4
    } else {
        8
    }
}

/// Lee el índice de paleta de la posición lineal `i` del empaquetado.
fn read_packed(packed: &[u64], bits: u8, i: usize) -> usize {
    let slots = per_word(bits);
    let shift = (i % slots) * usize::from(bits);
    let mask = capacity(bits) as u64 - 1;
    let word = packed.get(i / slots).copied().unwrap_or(0);
    ((word >> shift) & mask) as usize
}

/// Escribe el índice de paleta `value` en la posición lineal `i`.
fn write_packed(packed: &mut [u64], bits: u8, i: usize, value: usize) {
    let slots = per_word(bits);
    let shift = (i % slots) * usize::from(bits);
    let mask = capacity(bits) as u64 - 1;
    if let Some(word) = packed.get_mut(i / slots) {
        *word = (*word & !(mask << shift)) | ((value as u64 & mask) << shift);
    }
}
