//! Guardado por regiones: archivos `.hreg` de 32³ chunks, atómicos.
//!
//! El mundo se reparte en regiones de 32³ chunks, cada una en su
//! `r.RX.RY.RZ.hreg`, y cada archivo guarda solo los chunks modificados
//! (los generados se recalculan), codificados «paleta + RLE».
//! [`RegionFile::save_atomic`] escribe primero un `.tmp` y renombra al
//! final: un corte a mitad de guardado nunca destruye el archivo anterior.

use std::collections::BTreeMap;
use std::path::Path;

use grisu_voxel::{CHUNK_VOLUME, Chunk, ChunkPos};

// ---------------------------------------------------------------------------
// Formato binario de un archivo `.hreg` (todo en little-endian):
//
//   magia:            b"HREG"                        (4 bytes)
//   rx, ry, rz:       i32 × 3, coordenadas de región (12 bytes)
//   worldgen_version: u32, versión del generador     (4 bytes)
//   chunk_count:      u32, chunks modificados        (4 bytes)
//   tabla:            chunk_count × { local_idx: u16, offset: u32, length: u32 }
//   blob:             los chunks concatenados; cada `offset` es relativo al
//                     inicio del blob y `length` el tamaño de ese chunk.
//
// Codificación de un chunk dentro del blob:
//   modo 1 (uniforme): byte `1` e `id: u16`; el chunk entero es ese bloque.
//   modo 2 (paleta + RLE): byte `2`, `n: u16` (1..=256), `n` ids `u16` en
//     orden de primera aparición y tramos `(index: u16, run: u16)` que suman
//     exactamente `CHUNK_VOLUME` bloques en el orden lineal de `as_bytes`.
//
// La tabla se escribe en orden de `local_idx` (recorrido ordenado de un
// `BTreeMap`): un mismo contenido produce siempre los mismos bytes.
// ---------------------------------------------------------------------------

/// Bits de desplazamiento de chunk a región (32³ chunks por archivo).
pub const REGION_SHIFT: u32 = 5;
/// Chunks por lado de región.
pub const REGION_SIZE: i32 = 1 << REGION_SHIFT;
/// Chunks que caben en una región (32³ = 32 768).
pub const REGION_CHUNKS: usize = (REGION_SIZE * REGION_SIZE * REGION_SIZE) as usize;

const LOCAL_MASK: i32 = REGION_SIZE - 1;
const MAGIC: &[u8] = b"HREG";
const MODE_UNIFORM: u8 = 1;
const MODE_PALETTE: u8 = 2;

/// Coordenada de región (chunk >> 5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionPos {
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
}

impl RegionPos {
    /// Región que contiene al chunk `pos`. El desplazamiento aritmético
    /// redondea hacia −∞, así que funciona con coordenadas negativas.
    pub fn from_chunk(pos: ChunkPos) -> Self {
        RegionPos {
            rx: pos.x >> REGION_SHIFT,
            ry: pos.y >> REGION_SHIFT,
            rz: pos.z >> REGION_SHIFT,
        }
    }

    /// Índice local del chunk dentro de su región (0..=32767). Igual que en
    /// los chunks, `x` varía más rápido, luego `z` y luego `y`:
    /// `lx + lz*32 + ly*1024`.
    pub fn local_index(pos: ChunkPos) -> u16 {
        let lx = pos.x & LOCAL_MASK;
        let lz = pos.z & LOCAL_MASK;
        let ly = pos.y & LOCAL_MASK;
        (lx + lz * REGION_SIZE + ly * REGION_SIZE * REGION_SIZE) as u16
    }

    /// Inversa de [`RegionPos::local_index`]: chunk global del índice local.
    /// Devuelve `None` si `local_idx` ≥ [`REGION_CHUNKS`] (un `u16` llega
    /// hasta 65535 y hay que validarlo).
    pub fn chunk_pos(self, local_idx: u16) -> Option<ChunkPos> {
        if local_idx as usize >= REGION_CHUNKS {
            return None;
        }
        let idx = local_idx as i32;
        let ly = idx / (REGION_SIZE * REGION_SIZE);
        let rem = idx % (REGION_SIZE * REGION_SIZE);
        let lz = rem / REGION_SIZE;
        let lx = rem % REGION_SIZE;
        Some(ChunkPos::new(
            self.rx * REGION_SIZE + lx,
            self.ry * REGION_SIZE + ly,
            self.rz * REGION_SIZE + lz,
        ))
    }

    /// Nombre del archivo de la región dentro de su directorio.
    pub fn file_name(self) -> String {
        format!("r.{}.{}.{}.hreg", self.rx, self.ry, self.rz)
    }
}

/// Un archivo de región no se pudo leer o no tiene un formato válido.
#[derive(Debug)]
pub enum RegionError {
    Io(std::io::Error),
    Corrupt(&'static str),
}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionError::Io(err) => write!(f, "error de E/S: {err}"),
            RegionError::Corrupt(reason) => write!(f, "región corrupta: {reason}"),
        }
    }
}

impl std::error::Error for RegionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RegionError::Io(err) => Some(err),
            RegionError::Corrupt(_) => None,
        }
    }
}

impl From<std::io::Error> for RegionError {
    fn from(err: std::io::Error) -> Self {
        RegionError::Io(err)
    }
}

/// Un archivo de región en memoria: solo los chunks modificados.
#[derive(Clone, Debug)]
pub struct RegionFile {
    pos: RegionPos,
    worldgen_version: u32,
    chunks: BTreeMap<u16, Chunk>,
}

impl RegionFile {
    /// Región vacía, sin ningún chunk modificado.
    pub fn new(pos: RegionPos, worldgen_version: u32) -> Self {
        RegionFile {
            pos,
            worldgen_version,
            chunks: BTreeMap::new(),
        }
    }

    pub fn region_pos(&self) -> RegionPos {
        self.pos
    }

    pub fn worldgen_version(&self) -> u32 {
        self.worldgen_version
    }

    /// Guarda el contenido nuevo de un chunk de la región. Devuelve error si
    /// el índice local no cabe en la región.
    pub fn mark_modified(&mut self, local_idx: u16, chunk: Chunk) -> Result<(), RegionError> {
        if local_idx as usize >= REGION_CHUNKS {
            return Err(RegionError::Corrupt("índice local fuera de rango"));
        }
        self.chunks.insert(local_idx, chunk);
        Ok(())
    }

    /// El chunk modificado con ese índice local, si se guardó.
    pub fn chunk(&self, local_idx: u16) -> Option<&Chunk> {
        self.chunks.get(&local_idx)
    }

    /// Índices locales guardados, de menor a mayor.
    pub fn modified_indices(&self) -> Vec<u16> {
        self.chunks.keys().copied().collect()
    }

    /// Escribe la región en `dir` de forma atómica: todo el contenido va a
    /// `nombre.tmp` y solo al final se renombra al nombre definitivo.
    pub fn save_atomic(&self, dir: &Path) -> Result<(), RegionError> {
        std::fs::create_dir_all(dir)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&self.pos.rx.to_le_bytes());
        bytes.extend_from_slice(&self.pos.ry.to_le_bytes());
        bytes.extend_from_slice(&self.pos.rz.to_le_bytes());
        bytes.extend_from_slice(&self.worldgen_version.to_le_bytes());
        bytes.extend_from_slice(&(self.chunks.len() as u32).to_le_bytes());

        // Primero el blob (así se conocen los offsets) y después la tabla.
        let mut blob = Vec::new();
        let mut table: Vec<(u16, u32, u32)> = Vec::new();
        for (&local_idx, chunk) in &self.chunks {
            let offset = blob.len() as u32;
            encode_chunk(chunk, &mut blob);
            let length = (blob.len() as u32) - offset;
            table.push((local_idx, offset, length));
        }
        for &(local_idx, offset, length) in &table {
            bytes.extend_from_slice(&local_idx.to_le_bytes());
            bytes.extend_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(&length.to_le_bytes());
        }
        bytes.extend_from_slice(&blob);

        let name = self.pos.file_name();
        let tmp = dir.join(format!("{name}.tmp"));
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, dir.join(name))?;
        Ok(())
    }

    /// Lee y valida un archivo `.hreg` completo. Ante cualquier byte de
    /// menos, una magia distinta, una tabla imposible o tramos que no sumen
    /// 4096 devuelve `Err`; nunca entra en pánico.
    pub fn load(path: &Path) -> Result<RegionFile, RegionError> {
        let data = std::fs::read(path)?;
        let (magic, mut cursor) = split(&data, MAGIC.len()).ok_or(corrupt("cabecera truncada"))?;
        if magic != MAGIC {
            return Err(corrupt("la magia no es HREG"));
        }
        let rx = next_i32(&mut cursor).ok_or(corrupt("cabecera truncada"))?;
        let ry = next_i32(&mut cursor).ok_or(corrupt("cabecera truncada"))?;
        let rz = next_i32(&mut cursor).ok_or(corrupt("cabecera truncada"))?;
        let worldgen_version = next_u32(&mut cursor).ok_or(corrupt("cabecera truncada"))?;
        let chunk_count = next_u32(&mut cursor).ok_or(corrupt("cabecera truncada"))?;
        if chunk_count as usize > REGION_CHUNKS {
            return Err(corrupt("más chunks de los que caben en una región"));
        }

        let mut table: Vec<(u16, u32, u32)> = Vec::new();
        for _ in 0..chunk_count {
            let local_idx = next_u16(&mut cursor).ok_or(corrupt("tabla truncada"))?;
            let offset = next_u32(&mut cursor).ok_or(corrupt("tabla truncada"))?;
            let length = next_u32(&mut cursor).ok_or(corrupt("tabla truncada"))?;
            if local_idx as usize >= REGION_CHUNKS {
                return Err(corrupt("índice local fuera de rango"));
            }
            table.push((local_idx, offset, length));
        }

        // El resto del archivo es el blob; cada entrada debe caer dentro.
        let blob = cursor;
        let mut chunks = BTreeMap::new();
        for &(local_idx, offset, length) in &table {
            let start = offset as usize;
            let end = start
                .checked_add(length as usize)
                .ok_or(corrupt("offset desbordado"))?;
            let encoded = blob
                .get(start..end)
                .ok_or(corrupt("el chunk se sale del blob"))?;
            let (chunk, used) = decode_chunk(encoded)?;
            if used != encoded.len() {
                return Err(corrupt("bytes sobrantes al final del chunk"));
            }
            chunks.insert(local_idx, chunk);
        }

        Ok(RegionFile {
            pos: RegionPos { rx, ry, rz },
            worldgen_version,
            chunks,
        })
    }
}

fn corrupt(reason: &'static str) -> RegionError {
    RegionError::Corrupt(reason)
}

/// Parte `data` en sus primeros `n` bytes y el resto, o `None` si faltan.
fn split(data: &[u8], n: usize) -> Option<(&[u8], &[u8])> {
    if data.len() < n {
        None
    } else {
        Some(data.split_at(n))
    }
}

/// Lee `N` bytes little-endian y avanza el cursor, o `None` si faltan.
fn next_le<const N: usize>(cursor: &mut &[u8]) -> Option<[u8; N]> {
    let (head, rest) = split(cursor, N)?;
    let bytes: [u8; N] = head.try_into().ok()?;
    *cursor = rest;
    Some(bytes)
}

fn next_u16(cursor: &mut &[u8]) -> Option<u16> {
    Some(u16::from_le_bytes(next_le(cursor)?))
}

fn next_u32(cursor: &mut &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(next_le(cursor)?))
}

fn next_i32(cursor: &mut &[u8]) -> Option<i32> {
    Some(i32::from_le_bytes(next_le(cursor)?))
}

/// Los ids del archivo son `u16` por si el rango crece algún día, pero hoy
/// el motor guarda un byte por bloque: fuera de rango es corrupción.
fn block_from_id(id: u16) -> Result<u8, RegionError> {
    u8::try_from(id).map_err(|_| corrupt("id de bloque fuera de rango"))
}

/// Añade a `out` la codificación del chunk: modo uniforme si solo hay un
/// bloque distinto y modo paleta + RLE en otro caso.
fn encode_chunk(chunk: &Chunk, out: &mut Vec<u8>) {
    let blocks = chunk.as_bytes();

    // Paleta en orden de primera aparición. Sin HashSet: máximo 256 ids y la
    // búsqueda lineal mantiene el orden (y el archivo, determinista).
    let mut palette: Vec<u16> = Vec::new();
    for &block in blocks {
        let id = u16::from(block);
        if !palette.contains(&id) {
            palette.push(id);
        }
    }

    if palette.len() == 1 {
        out.push(MODE_UNIFORM);
        if let Some(&id) = palette.first() {
            out.extend_from_slice(&id.to_le_bytes());
        }
        return;
    }

    out.push(MODE_PALETTE);
    out.extend_from_slice(&(palette.len() as u16).to_le_bytes());
    for &id in &palette {
        out.extend_from_slice(&id.to_le_bytes());
    }

    // Tramos sobre el orden lineal del chunk: índice de paleta y longitud
    // de cada tanda de bloques iguales consecutivos.
    let mut iter = blocks.iter().copied().peekable();
    while let Some(block) = iter.next() {
        let id = u16::from(block);
        let index = palette.iter().position(|&p| p == id).unwrap_or(0);
        let mut run: usize = 1;
        while iter.peek() == Some(&block) {
            iter.next();
            run += 1;
        }
        out.extend_from_slice(&(index as u16).to_le_bytes());
        out.extend_from_slice(&(run as u16).to_le_bytes());
    }
}

/// Decodifica un chunk del blob y devuelve cuántos bytes ha consumido.
/// Valida tamaños de paleta, índices, tramos vacíos y que los tramos sumen
/// exactamente `CHUNK_VOLUME`.
fn decode_chunk(data: &[u8]) -> Result<(Chunk, usize), RegionError> {
    let (&mode, mut cursor) = data.split_first().ok_or(corrupt("chunk vacío"))?;
    match mode {
        MODE_UNIFORM => {
            let id = next_u16(&mut cursor).ok_or(corrupt("chunk uniforme truncado"))?;
            let block = block_from_id(id)?;
            Ok((Chunk::filled(block), data.len() - cursor.len()))
        }
        MODE_PALETTE => {
            let n = next_u16(&mut cursor).ok_or(corrupt("tamaño de paleta truncado"))? as usize;
            if n == 0 || n > 256 {
                return Err(corrupt("tamaño de paleta inválido"));
            }
            let mut palette: Vec<u8> = Vec::with_capacity(n);
            for _ in 0..n {
                let id = next_u16(&mut cursor).ok_or(corrupt("paleta truncada"))?;
                palette.push(block_from_id(id)?);
            }

            let mut chunk = Chunk::default();
            let mut filled = 0usize;
            while filled < CHUNK_VOLUME {
                let index = next_u16(&mut cursor).ok_or(corrupt("tramos truncados"))?;
                let run = next_u16(&mut cursor).ok_or(corrupt("tramos truncados"))? as usize;
                if run == 0 {
                    return Err(corrupt("tramo de longitud cero"));
                }
                let end = filled.checked_add(run).ok_or(corrupt("tramo desbordado"))?;
                if end > CHUNK_VOLUME {
                    return Err(corrupt("los tramos suman más de 4096"));
                }
                let block = match palette.get(index as usize) {
                    Some(&block) => block,
                    None => return Err(corrupt("índice de paleta fuera de rango")),
                };
                for i in filled..end {
                    chunk.set(i % 16, i / 256, (i % 256) / 16, block);
                }
                filled = end;
            }
            Ok((chunk, data.len() - cursor.len()))
        }
        _ => Err(corrupt("modo de chunk desconocido")),
    }
}
