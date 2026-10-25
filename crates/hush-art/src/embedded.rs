//! Texturas reales de los bloques, incrustadas en el binario.
//!
//! Cada bloque sólido (1-17) ocupa una capa del texture array: la capa `i`
//! es el bloque `i + 1`. La correspondencia con `manifest.ron` es el
//! fotograma 0 de la lava y la variante `copper` en los bloques con equipo
//! (el motor aún no guarda el equipo de cada celda).

use std::fmt;

use grisu_render::TextureData;
use hush_sim::BlockKind;

/// PNG incrustado en el binario: nombre de archivo y sus bytes.
type EmbeddedPng = (&'static str, &'static [u8]);

/// PNG de `assets/textures/blocks` incrustado con `include_bytes!`.
macro_rules! png {
    ($file:literal) => {
        (
            $file,
            include_bytes!(concat!("../../../assets/textures/blocks/", $file)),
        )
    };
}

/// Una textura por bloque sólido, en orden de identificador.
static TEXTURES: [(BlockKind, EmbeddedPng); 17] = [
    (BlockKind::Dirt, png!("dirt.png")),
    (BlockKind::Rock, png!("rock.png")),
    (BlockKind::HardRock, png!("hard_rock.png")),
    (BlockKind::Ore, png!("ore.png")),
    (BlockKind::Gold, png!("gold.png")),
    (BlockKind::Diamond, png!("diamond.png")),
    (BlockKind::Lava, png!("lava_0.png")),
    (BlockKind::TeamBlock, png!("team_block_copper.png")),
    (BlockKind::ForceField, png!("force_field_copper.png")),
    (BlockKind::Ladder, png!("ladder.png")),
    (BlockKind::JumpPad, png!("jump_pad.png")),
    (BlockKind::Electric, png!("electric.png")),
    (BlockKind::Rail, png!("rail.png")),
    (BlockKind::Beacon, png!("beacon_copper.png")),
    (BlockKind::Explosive, png!("explosive.png")),
    (BlockKind::BucketStation, png!("bucket_station_copper.png")),
    (BlockKind::BucketPost, png!("bucket_post_copper.png")),
];

/// Capa del fotograma 0 de la lava dentro del texture array (bloque 7).
/// Debe coincidir con `LAVA_LAYER` en `voxel.wgsl`.
pub const LAVA_LAYER: u32 = 6;
/// Fotogramas de la lava en el array: el 0 en su capa y los 1-3 extra.
pub const LAVA_FRAMES: u32 = 4;

/// Fotogramas 1-3 de la lava: capas extra al final del texture array para
/// que el shader las anime con el tiempo.
static LAVA_EXTRA_FRAMES: [EmbeddedPng; 3] =
    [png!("lava_1.png"), png!("lava_2.png"), png!("lava_3.png")];

/// Error al decodificar una textura de bloque incrustada o al leer el
/// manifiesto de capas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockTextureError {
    /// El PNG incrustado no se puede decodificar.
    Undecodable {
        /// Archivo PNG implicado.
        file: &'static str,
    },
    /// El PNG decodificado no guarda el tamaño común de las capas: cuadrado,
    /// potencia de dos, de lado 16 o 32.
    WrongSize {
        /// Archivo PNG implicado.
        file: &'static str,
        /// Ancho encontrado.
        width: u32,
        /// Alto encontrado.
        height: u32,
    },
    /// El manifiesto de capas tiene un formato inválido o sus capas no son
    /// consecutivas desde 0.
    InvalidManifest {
        /// Qué está mal y dónde.
        reason: String,
    },
    /// El manifiesto declara una capa cuyo PNG no está incrustado.
    MissingTexture {
        /// Nombre del bloque sin textura incrustada.
        block_name: String,
    },
}

impl fmt::Display for BlockTextureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undecodable { file } => {
                write!(f, "no se pudo decodificar el PNG incrustado '{file}'")
            }
            Self::WrongSize {
                file,
                width,
                height,
            } => write!(
                f,
                "'{file}' mide {width}×{height}; se esperaba una textura \
                 cuadrada de lado 16 o 32, igual que las demás capas"
            ),
            Self::InvalidManifest { reason } => {
                write!(f, "manifiesto de capas inválido: {reason}")
            }
            Self::MissingTexture { block_name } => {
                write!(f, "el manifiesto declara '{block_name}' sin PNG incrustado")
            }
        }
    }
}

impl std::error::Error for BlockTextureError {}

/// Texturas de todos los bloques salvo el aire, en orden de identificador,
/// más los fotogramas 1-3 de la lava como capas extra al final.
///
/// Decodifica los PNG incrustados en tiempo de compilación; no lee el disco
/// en ejecución. Todas las capas comparten el tamaño de la primera (cuadrado,
/// potencia de dos, de lado 16 o 32). Falla (sin `panic!`) si algún PNG no se
/// puede decodificar o no guarda ese tamaño.
pub fn block_textures() -> Result<Vec<TextureData>, BlockTextureError> {
    let pngs = TEXTURES
        .iter()
        .map(|&(_, png)| png)
        .chain(LAVA_EXTRA_FRAMES.iter().copied());
    let mut layers = Vec::with_capacity(TEXTURES.len() + LAVA_EXTRA_FRAMES.len());
    let mut side = None;
    for (file, bytes) in pngs {
        decode_layer(&mut layers, &mut side, file, bytes)?;
    }
    Ok(layers)
}

/// Archivo PNG de `assets/textures/blocks` que usa cada bloque (`None` para
/// el aire, que no se dibuja).
pub fn texture_file(block: BlockKind) -> Option<&'static str> {
    TEXTURES
        .iter()
        .find(|(kind, _)| *kind == block)
        .map(|(_, png)| png.0)
}

/// Decodifica un PNG incrustado a RGBA y lo añade como capa del array. La
/// primera capa fija el lado común; las siguientes deben medir lo mismo.
fn decode_layer(
    layers: &mut Vec<TextureData>,
    side: &mut Option<u32>,
    file: &'static str,
    bytes: &'static [u8],
) -> Result<(), BlockTextureError> {
    let wrong_size = |width, height| BlockTextureError::WrongSize {
        file,
        width,
        height,
    };
    let rgba = image::load_from_memory(bytes)
        .map_err(|_| BlockTextureError::Undecodable { file })?
        .to_rgba8();
    let (width, height) = rgba.dimensions();
    let expected = match *side {
        Some(side) => side,
        None => {
            if width != height || !matches!(width, 16 | 32) {
                return Err(wrong_size(width, height));
            }
            *side = Some(width);
            width
        }
    };
    if width != expected || height != expected {
        return Err(wrong_size(width, height));
    }
    layers.push(TextureData {
        width,
        height,
        rgba: rgba.into_raw(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TeamVariant;
    use crate::textured_blocks;

    /// Todas las capas comparten el tamaño de la primera: cuadrado, potencia
    /// de dos, de lado 16 o 32 (los PNG actuales miden 16).
    #[test]
    fn all_layers_share_one_square_size_plus_lava_frames() {
        let textures = block_textures().expect("decodificar los PNG incrustados");
        // Una capa por bloque sólido (17) y los fotogramas extra de la lava.
        assert_eq!(textured_blocks().len(), 17);
        assert_eq!(textures.len(), 17 + LAVA_EXTRA_FRAMES.len());
        let (width, height) = (textures[0].width, textures[0].height);
        assert_eq!(width, height);
        assert!(matches!(width, 16 | 32));
        for t in &textures {
            assert_eq!((t.width, t.height), (width, height));
            assert_eq!(t.rgba.len(), (width * height * 4) as usize);
        }
    }

    /// El fotograma 0 ocupa la capa de la lava (`LAVA_LAYER`) y los 1-3 van
    /// al final del array; los cuatro difieren entre sí.
    #[test]
    fn lava_frames_append_four_distinct_layers() {
        let textures = block_textures().expect("decodificar los PNG incrustados");
        let mut frames: Vec<&TextureData> =
            vec![&textures[usize::try_from(LAVA_LAYER).expect("capa de la lava")]];
        frames.extend(&textures[textures.len() - LAVA_EXTRA_FRAMES.len()..]);
        assert_eq!(frames.len(), LAVA_FRAMES as usize);
        for i in 0..frames.len() {
            for j in i + 1..frames.len() {
                assert_ne!(
                    frames[i].rgba, frames[j].rgba,
                    "los fotogramas {i} y {j} de la lava son idénticos"
                );
            }
        }
    }

    #[test]
    fn layers_follow_block_identifiers() {
        assert_eq!(TEXTURES.len(), textured_blocks().len());
        for (index, (block, _)) in TEXTURES.iter().enumerate() {
            assert_eq!(block, &textured_blocks()[index]);
            assert_eq!(usize::from(block.id()), index + 1);
        }
    }

    /// Para cada bloque, el archivo elegido debe ser el que el manifiesto
    /// declara con `frame: 0` y variante cobre (o sin variante).
    #[test]
    fn chosen_files_match_manifest() {
        let entries = crate::parse_manifest().expect("parsear el manifiesto");
        for &block in textured_blocks() {
            let id = block.id();
            let of_block: Vec<_> = entries.iter().filter(|e| e.block_id == id).collect();
            assert!(
                !of_block.is_empty(),
                "block_id {id} no aparece en el manifiesto"
            );

            let chosen = of_block
                .iter()
                .find(|e| e.frame == 0 && e.variant != Some(TeamVariant::Cobalt))
                .unwrap_or_else(|| panic!("block_id {id} sin fotograma 0 en cobre"));
            let has_frames = of_block.iter().any(|e| e.frame != 0);
            let expected = match chosen.variant {
                Some(TeamVariant::Copper) => format!("{}_copper.png", chosen.block_name),
                _ if has_frames => format!("{}_0.png", chosen.block_name),
                _ => format!("{}.png", chosen.block_name),
            };
            assert_eq!(texture_file(block), Some(expected.as_str()));
        }
    }

    #[test]
    fn valuable_blocks_are_distinguishable() {
        let textures = block_textures().expect("decodificar los PNG incrustados");
        let layer = |b: BlockKind| &textures[usize::from(b.id()) - 1];
        let gold = mean(layer(BlockKind::Gold));
        let ore = mean(layer(BlockKind::Ore));
        let diamond = mean(layer(BlockKind::Diamond));
        for pair in [(gold, ore), (gold, diamond), (ore, diamond)] {
            let distance: u32 = (0..3).map(|k| pair.0[k].abs_diff(pair.1[k])).sum();
            assert!(
                distance > 12,
                "{:?} y {:?} se parecen demasiado",
                pair.0,
                pair.1
            );
        }
        assert!(
            gold[0] > gold[2] + 20,
            "el oro debería verse amarillo: {gold:?}"
        );
        assert!(
            diamond[2] > diamond[0] + 20,
            "el diamante debería verse azulado: {diamond:?}"
        );
    }

    /// Color medio de una capa, a partir de sus propias dimensiones.
    fn mean(t: &TextureData) -> [u32; 3] {
        let pixels = t.width * t.height;
        let mut sum = [0u32; 3];
        for p in t.rgba.chunks(4) {
            for i in 0..3 {
                sum[i] += u32::from(p[i]);
            }
        }
        sum.map(|s| s / pixels)
    }
}
