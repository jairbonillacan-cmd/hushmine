//! Texturas reales de los bloques, incrustadas en el binario.
//!
//! Cada bloque sólido (1-17) ocupa una capa del texture array: la capa `i`
//! es el bloque `i + 1`. La correspondencia con `manifest.ron` es el
//! fotograma 0 de la lava y la variante `copper` en los bloques con equipo
//! (el motor aún no guarda el equipo de cada celda).

use std::fmt;

use grisu_render::TextureData;
use hush_sim::BlockKind;

/// Lado del PNG de los bloques, en píxeles.
const SIZE: u32 = 16;

/// Un PNG de bloque incrustado en el binario en tiempo de compilación.
struct EmbeddedTexture {
    /// Bloque al que pertenece la textura.
    block: BlockKind,
    /// Nombre del archivo dentro de `assets/textures/blocks`.
    file: &'static str,
    /// Bytes del PNG, incrustados con `include_bytes!`.
    bytes: &'static [u8],
}

/// Una textura por bloque sólido, en orden de identificador.
static TEXTURES: [EmbeddedTexture; 17] = [
    EmbeddedTexture {
        block: BlockKind::Dirt,
        file: "dirt.png",
        bytes: include_bytes!("../../../assets/textures/blocks/dirt.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Rock,
        file: "rock.png",
        bytes: include_bytes!("../../../assets/textures/blocks/rock.png"),
    },
    EmbeddedTexture {
        block: BlockKind::HardRock,
        file: "hard_rock.png",
        bytes: include_bytes!("../../../assets/textures/blocks/hard_rock.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Ore,
        file: "ore.png",
        bytes: include_bytes!("../../../assets/textures/blocks/ore.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Gold,
        file: "gold.png",
        bytes: include_bytes!("../../../assets/textures/blocks/gold.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Diamond,
        file: "diamond.png",
        bytes: include_bytes!("../../../assets/textures/blocks/diamond.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Lava,
        file: "lava_0.png",
        bytes: include_bytes!("../../../assets/textures/blocks/lava_0.png"),
    },
    EmbeddedTexture {
        block: BlockKind::TeamBlock,
        file: "team_block_copper.png",
        bytes: include_bytes!("../../../assets/textures/blocks/team_block_copper.png"),
    },
    EmbeddedTexture {
        block: BlockKind::ForceField,
        file: "force_field_copper.png",
        bytes: include_bytes!("../../../assets/textures/blocks/force_field_copper.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Ladder,
        file: "ladder.png",
        bytes: include_bytes!("../../../assets/textures/blocks/ladder.png"),
    },
    EmbeddedTexture {
        block: BlockKind::JumpPad,
        file: "jump_pad.png",
        bytes: include_bytes!("../../../assets/textures/blocks/jump_pad.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Electric,
        file: "electric.png",
        bytes: include_bytes!("../../../assets/textures/blocks/electric.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Rail,
        file: "rail.png",
        bytes: include_bytes!("../../../assets/textures/blocks/rail.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Beacon,
        file: "beacon_copper.png",
        bytes: include_bytes!("../../../assets/textures/blocks/beacon_copper.png"),
    },
    EmbeddedTexture {
        block: BlockKind::Explosive,
        file: "explosive.png",
        bytes: include_bytes!("../../../assets/textures/blocks/explosive.png"),
    },
    EmbeddedTexture {
        block: BlockKind::BucketStation,
        file: "bucket_station_copper.png",
        bytes: include_bytes!("../../../assets/textures/blocks/bucket_station_copper.png"),
    },
    EmbeddedTexture {
        block: BlockKind::BucketPost,
        file: "bucket_post_copper.png",
        bytes: include_bytes!("../../../assets/textures/blocks/bucket_post_copper.png"),
    },
];

/// Capa del fotograma 0 de la lava dentro del texture array (bloque 7).
/// Debe coincidir con `LAVA_LAYER` en `voxel.wgsl`.
pub const LAVA_LAYER: u32 = 6;
/// Fotogramas de la lava en el array: el 0 en su capa y los 1-3 extra.
pub const LAVA_FRAMES: u32 = 4;

/// Fotogramas 1-3 de la lava: capas extra al final del texture array para
/// que el shader las anime con el tiempo.
static LAVA_EXTRA_FRAMES: [(&str, &[u8]); 3] = [
    (
        "lava_1.png",
        include_bytes!("../../../assets/textures/blocks/lava_1.png"),
    ),
    (
        "lava_2.png",
        include_bytes!("../../../assets/textures/blocks/lava_2.png"),
    ),
    (
        "lava_3.png",
        include_bytes!("../../../assets/textures/blocks/lava_3.png"),
    ),
];

/// Error al decodificar una textura de bloque incrustada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockTextureError {
    /// El PNG incrustado no se puede decodificar.
    Undecodable {
        /// Archivo PNG implicado.
        file: &'static str,
    },
    /// El PNG decodificado no mide 16×16.
    WrongSize {
        /// Archivo PNG implicado.
        file: &'static str,
        /// Ancho encontrado.
        width: u32,
        /// Alto encontrado.
        height: u32,
    },
}

impl fmt::Display for BlockTextureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Undecodable { file } => {
                write!(f, "no se pudo decodificar el PNG incrustado '{file}'")
            }
            Self::WrongSize {
                file,
                width,
                height,
            } => {
                write!(
                    f,
                    "'{file}' mide {width}×{height}, se esperaba {SIZE}×{SIZE}"
                )
            }
        }
    }
}

impl std::error::Error for BlockTextureError {}

/// Texturas de todos los bloques salvo el aire, en orden de identificador,
/// más los fotogramas 1-3 de la lava como capas extra al final.
///
/// Decodifica los PNG incrustados en tiempo de compilación; no lee el disco
/// en ejecución. Falla (sin `panic!`) si algún PNG no se puede decodificar o
/// no mide 16×16.
pub fn block_textures() -> Result<Vec<TextureData>, BlockTextureError> {
    let mut layers: Vec<TextureData> = TEXTURES
        .iter()
        .map(|t| decode(t.file, t.bytes))
        .collect::<Result<_, _>>()?;
    layers.extend(
        LAVA_EXTRA_FRAMES
            .iter()
            .map(|&(file, bytes)| decode(file, bytes))
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(layers)
}

/// Archivo PNG de `assets/textures/blocks` que usa cada bloque (`None` para
/// el aire, que no se dibuja).
pub fn texture_file(block: BlockKind) -> Option<&'static str> {
    TEXTURES.iter().find(|t| t.block == block).map(|t| t.file)
}

/// Decodifica un PNG incrustado a [`TextureData`] RGBA de 16×16.
fn decode(file: &'static str, bytes: &'static [u8]) -> Result<TextureData, BlockTextureError> {
    let decoded =
        image::load_from_memory(bytes).map_err(|_| BlockTextureError::Undecodable { file })?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    if width != SIZE || height != SIZE {
        return Err(BlockTextureError::WrongSize {
            file,
            width,
            height,
        });
    }
    Ok(TextureData {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::textured_blocks;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Entrada de `manifest.ron`, parseada con un análisis de texto simple.
    struct ManifestEntry {
        block_id: u8,
        block_name: String,
        variant: Option<String>,
        frame: u8,
    }

    fn manifest_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/textures/blocks/manifest.ron")
    }

    /// Parsea las líneas `(layer: N, block_id: N, block_name: "...", variant:
    /// ..., frame: N),` del manifiesto, sin crates nuevos.
    fn parse_manifest() -> Vec<ManifestEntry> {
        let text = fs::read_to_string(manifest_path()).expect("leer manifest.ron");
        let mut entries = Vec::new();
        for line in text.lines().map(str::trim) {
            if !line.starts_with('(') {
                continue;
            }
            let mut block_id = None;
            let mut block_name = None;
            let mut variant = None;
            let mut frame = None;
            for field in line
                .trim_start_matches('(')
                .trim_end_matches([')', ','])
                .split(',')
            {
                let (key, value) = field.split_once(':').expect("campo 'clave: valor'");
                let value = value.trim();
                match key.trim() {
                    "block_id" => block_id = value.parse().ok(),
                    "block_name" => block_name = Some(value.trim_matches('"').to_owned()),
                    // `None` o `Some("copper")`.
                    "variant" => {
                        variant = value
                            .strip_prefix("Some(")
                            .map(|v| v.trim_end_matches(')').trim_matches('"').to_owned());
                    }
                    "frame" => frame = value.parse().ok(),
                    _ => {}
                }
            }
            let (Some(block_id), Some(block_name), Some(frame)) = (block_id, block_name, frame)
            else {
                panic!("entrada de manifiesto incompleta: {line}");
            };
            entries.push(ManifestEntry {
                block_id,
                block_name,
                variant,
                frame,
            });
        }
        assert!(!entries.is_empty(), "el manifiesto no aportó entradas");
        entries
    }

    fn mean(t: &TextureData) -> [u32; 3] {
        let mut sum = [0u32; 3];
        for p in t.rgba.chunks(4) {
            for i in 0..3 {
                sum[i] += u32::from(p[i]);
            }
        }
        sum.map(|s| s / (SIZE * SIZE))
    }

    #[test]
    fn one_16x16_layer_per_block_plus_lava_frames() {
        let textures = block_textures().expect("decodificar los PNG incrustados");
        // Una capa por bloque sólido (17) y los fotogramas extra de la lava.
        assert_eq!(textured_blocks().len(), 17);
        assert_eq!(textures.len(), 17 + LAVA_EXTRA_FRAMES.len());
        for t in &textures {
            assert_eq!((t.width, t.height), (16, 16));
            assert_eq!(t.rgba.len(), (16 * 16 * 4) as usize);
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
        for (index, texture) in TEXTURES.iter().enumerate() {
            assert_eq!(texture.block, textured_blocks()[index]);
            assert_eq!(usize::from(texture.block.id()), index + 1);
        }
    }

    /// Para cada bloque, el archivo elegido debe ser el que el manifiesto
    /// declara con `frame: 0` y variante `copper` (o sin variante).
    #[test]
    fn chosen_files_match_manifest() {
        let entries = parse_manifest();
        for &block in textured_blocks() {
            let id = block.id();
            let of_block: Vec<_> = entries.iter().filter(|e| e.block_id == id).collect();
            assert!(
                !of_block.is_empty(),
                "block_id {id} no aparece en el manifiesto"
            );

            let chosen = of_block
                .iter()
                .find(|e| e.frame == 0 && e.variant.as_deref() != Some("cobalt"))
                .unwrap_or_else(|| panic!("block_id {id} sin fotograma 0 en cobre"));
            let has_frames = of_block.iter().any(|e| e.frame != 0);
            let expected = match chosen.variant.as_deref() {
                Some("copper") => format!("{}_copper.png", chosen.block_name),
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
}
