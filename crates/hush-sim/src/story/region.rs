//! Persistencia en formato de región propietario `.hush`.
//!
//! Cada archivo agrupa una rejilla de hasta 32×32 columnas de chunks (los
//! mundos actuales de hasta 128×128 bloques caben en una sola región). Los
//! chunks se codifican con tramos de repetición (RLE) de un byte por vóxel;
//! el formato reserva un byte de códec para comprimir con `zstd` en el
//! futuro sin romper la compatibilidad. Todo determinista: guardar y cargar
//! no cambia el contenido ni la huella del mundo.

use std::io::{self, Read, Write};
use std::path::Path;

use grisu_voxel::{CHUNK_SIZE, Chunk, ChunkDecodeError, VoxelWorld};

/// Mágica del formato.
const MAGIC: &[u8; 8] = b"HUSHRG01";
/// Versión actual del formato.
const VERSION: u16 = 1;
/// Códec RLE (el 1 quedará para zstd).
const CODEC_RLE: u8 = 0;
/// Tamaño de la rejilla de la región, en columnas de chunk.
pub const REGION_CHUNKS: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegionError {
    Io(io::ErrorKind, String),
    /// El archivo no empieza con la mágica correcta.
    BadMagic,
    UnsupportedVersion(u16),
    UnsupportedCodec(u8),
    /// El tamaño declarado no encaja con los datos.
    Malformed(String),
    /// El mundo reconstruido no es válido.
    World(grisu_voxel::WorldError),
    Chunk(ChunkDecodeError),
}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(kind, msg) => write!(f, "error de E/S ({kind:?}): {msg}"),
            Self::BadMagic => write!(f, "no es un archivo de región .hush"),
            Self::UnsupportedVersion(v) => write!(f, "versión de región no soportada: {v}"),
            Self::UnsupportedCodec(c) => write!(f, "códec de región no soportado: {c}"),
            Self::Malformed(msg) => write!(f, "región malformada: {msg}"),
            Self::World(e) => write!(f, "mundo inválido: {e}"),
            Self::Chunk(e) => write!(f, "chunk inválido: {e}"),
        }
    }
}

impl std::error::Error for RegionError {}

fn io_err(e: io::Error) -> RegionError {
    RegionError::Io(e.kind(), e.to_string())
}

/// Codifica un chunk con tramos de repetición: pares `(bloque, longitud)`
/// con longitudes de 1 a 255.
pub fn encode_chunk_rle(chunk: &Chunk) -> Vec<u8> {
    let bytes = chunk.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 8 + 8);
    let mut run_block = bytes[0];
    let mut run_len: u8 = 1;
    for &b in &bytes[1..] {
        if b == run_block && run_len < 255 {
            run_len += 1;
        } else {
            out.extend_from_slice(&[run_block, run_len]);
            run_block = b;
            run_len = 1;
        }
    }
    out.extend_from_slice(&[run_block, run_len]);
    out
}

/// Decodifica un chunk RLE. Devuelve `None` si los datos no suman 4096.
pub fn decode_chunk_rle(data: &[u8]) -> Option<Chunk> {
    if !data.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = [0u8; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    let mut at = 0usize;
    for pair in data.as_chunks::<2>().0 {
        let run = pair[1] as usize;
        if at + run > bytes.len() {
            return None;
        }
        bytes[at..at + run].fill(pair[0]);
        at += run;
    }
    (at == bytes.len()).then(|| Chunk::from_bytes(bytes))
}

/// Serializa un mundo completo al formato de región.
pub fn encode_world(world: &VoxelWorld) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.push(CODEC_RLE);
    for axis in world.size() {
        out.extend_from_slice(&axis.to_le_bytes());
    }
    // Un bloque de longitud por chunk, con índice creciente.
    let mut chunks = Vec::with_capacity(world.chunk_count());
    for index in 0..world.chunk_count() {
        let encoded = encode_chunk_rle(world.chunk(index));
        chunks.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        chunks.extend_from_slice(&encoded);
    }
    out.extend_from_slice(&(world.chunk_count() as u32).to_le_bytes());
    out.extend(chunks);
    out
}

/// Reconstruye el mundo a partir de una región serializada.
pub fn decode_world(data: &[u8]) -> Result<VoxelWorld, RegionError> {
    if data.len() < 8 + 2 + 1 + 12 + 4 || &data[0..8] != MAGIC {
        return Err(RegionError::BadMagic);
    }
    let mut at = 8;
    let version = u16::from_le_bytes([data[at], data[at + 1]]);
    at += 2;
    if version != VERSION {
        return Err(RegionError::UnsupportedVersion(version));
    }
    let codec = data[at];
    at += 1;
    if codec != CODEC_RLE {
        return Err(RegionError::UnsupportedCodec(codec));
    }
    let mut size = [0u32; 3];
    for axis in &mut size {
        *axis = u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
        at += 4;
    }
    let count = u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    at += 4;
    let mut world = VoxelWorld::new(size).map_err(RegionError::World)?;
    if count as usize != world.chunk_count() {
        return Err(RegionError::Malformed(format!(
            "el archivo declara {count} chunks y el tamaño pide {}",
            world.chunk_count()
        )));
    }
    for index in 0..count as usize {
        if at + 4 > data.len() {
            return Err(RegionError::Malformed("fin de archivo inesperado".into()));
        }
        let len = u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as usize;
        at += 4;
        if at + len > data.len() {
            return Err(RegionError::Malformed("chunk truncado".into()));
        }
        let chunk = decode_chunk_rle(&data[at..at + len])
            .ok_or_else(|| RegionError::Malformed(format!("RLE inválido en chunk {index}")))?;
        if !world.replace_chunk(index, chunk) {
            return Err(RegionError::Malformed(format!(
                "chunk {index} fuera de rango"
            )));
        }
        at += len;
    }
    Ok(world)
}

/// Guarda el mundo en un archivo `.hush`.
pub fn save_world(world: &VoxelWorld, path: &Path) -> Result<(), RegionError> {
    let mut file = std::fs::File::create(path).map_err(io_err)?;
    file.write_all(&encode_world(world)).map_err(io_err)?;
    file.flush().map_err(io_err)?;
    Ok(())
}

/// Carga un mundo desde un archivo `.hush`.
pub fn load_world(path: &Path) -> Result<VoxelWorld, RegionError> {
    let mut file = std::fs::File::open(path).map_err(io_err)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(io_err)?;
    decode_world(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;

    #[test]
    fn rle_round_trips_homogeneous_and_mixed_chunks() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([0, 0, 0], [15, 15, 15], BlockKind::Rock.id());
        let encoded = encode_chunk_rle(w.chunk(0));
        // Homogéneo: 17 tramos como mucho.
        assert!(encoded.len() < 40, "RLE poco eficiente: {}", encoded.len());
        assert_eq!(decode_chunk_rle(&encoded).unwrap(), *w.chunk(0));

        // Ajedrez: peor caso, un par por celda.
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let b = if (x + y + z) % 2 == 0 { 1 } else { 2 };
                    w.set(x, y, z, b);
                }
            }
        }
        let encoded = encode_chunk_rle(w.chunk(0));
        // Peor caso: un par por celda (alguna carrera de 2 en los bordes).
        assert!(encoded.len() <= CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 2);
        assert!(encoded.len() > CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        assert_eq!(decode_chunk_rle(&encoded).unwrap(), *w.chunk(0));
    }

    #[test]
    fn rle_rejects_bad_lengths() {
        assert_eq!(decode_chunk_rle(&[1]), None);
        assert_eq!(decode_chunk_rle(&[1, 255, 2, 255]), None); // no llega a 4096
        // Un byte de más: sobra.
        assert_eq!(decode_chunk_rle(&[1, 255, 2, 255, 3, 2]), None);
    }

    #[test]
    fn world_round_trip_preserves_hash_and_size() {
        let mut w = VoxelWorld::new([32, 32, 32]).unwrap();
        w.fill([0, 0, 0], [31, 5, 31], BlockKind::Rock.id());
        w.set(7, 7, 7, BlockKind::Diamond.id());
        w.set(8, 7, 7, BlockKind::Air.id());
        let encoded = encode_world(&w);
        let decoded = decode_world(&encoded).unwrap();
        assert_eq!(decoded.size(), w.size());
        assert_eq!(decoded.content_hash(), w.content_hash());
        assert_eq!(decoded, w);
    }

    #[test]
    fn malformed_files_are_rejected_without_panicking() {
        assert_eq!(decode_world(b"basura"), Err(RegionError::BadMagic));
        let mut data = encode_world(&VoxelWorld::new([16, 16, 16]).unwrap());
        // Versión imposible.
        data[8] = 9;
        data[9] = 9;
        assert!(matches!(
            decode_world(&data),
            Err(RegionError::UnsupportedVersion(_))
        ));
        // Códec futuro.
        let mut data = encode_world(&VoxelWorld::new([16, 16, 16]).unwrap());
        data[10] = 1;
        assert!(matches!(
            decode_world(&data),
            Err(RegionError::UnsupportedCodec(1))
        ));
        // Truncado.
        let data = encode_world(&VoxelWorld::new([16, 16, 16]).unwrap());
        assert!(matches!(
            decode_world(&data[..data.len() - 3]),
            Err(RegionError::Malformed(_))
        ));
    }

    #[test]
    fn save_and_load_through_disk() {
        let dir = std::env::temp_dir().join(format!("hush-region-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("prueba.hush");
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([2, 2, 2], [5, 5, 5], BlockKind::Gold.id());
        save_world(&w, &path).unwrap();
        let loaded = load_world(&path).unwrap();
        assert_eq!(loaded.content_hash(), w.content_hash());
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
