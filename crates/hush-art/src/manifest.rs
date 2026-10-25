//! Manifiesto de capas del texture array de bloques: lee
//! `assets/textures/blocks/manifest.ron` (incrustado en el binario) y
//! decodifica, en el orden exacto de sus capas, los PNG que declara.

use grisu_render::TextureData;
use hush_sim::BlockKind;

use crate::BlockTextureError;

/// Contenido de `manifest.ron`, incrustado en tiempo de compilación.
const MANIFEST_RON: &str = include_str!("../../../assets/textures/blocks/manifest.ron");

/// PNG incrustado: nombre de archivo y sus bytes.
type EmbeddedPng = (&'static str, &'static [u8]);

/// Tabla de PNG incrustados con `include_bytes!` desde `assets/textures/blocks`.
macro_rules! embedded {
    [$($file:literal),* $(,)?] => {
        [$(($file, include_bytes!(concat!("../../../assets/textures/blocks/", $file)))),*]
    };
}

/// Los 25 PNG del manifiesto, en el mismo orden que sus capas (0..=24).
static EMBEDDED: [EmbeddedPng; 25] = embedded![
    "dirt.png",
    "rock.png",
    "hard_rock.png",
    "ore.png",
    "gold.png",
    "diamond.png",
    "lava_0.png",
    "lava_1.png",
    "lava_2.png",
    "lava_3.png",
    "team_block_copper.png",
    "team_block_cobalt.png",
    "force_field_copper.png",
    "force_field_cobalt.png",
    "ladder.png",
    "jump_pad.png",
    "electric.png",
    "rail.png",
    "beacon_copper.png",
    "beacon_cobalt.png",
    "explosive.png",
    "bucket_station_copper.png",
    "bucket_station_cobalt.png",
    "bucket_post_copper.png",
    "bucket_post_cobalt.png",
];

/// Variante de equipo de los bloques con dos texturas.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TeamVariant {
    Copper,
    Cobalt,
}

impl TeamVariant {
    /// Interpreta el texto de la variante en el manifiesto.
    fn parse(text: &str) -> Option<Self> {
        match text {
            "copper" => Some(Self::Copper),
            "cobalt" => Some(Self::Cobalt),
            _ => None,
        }
    }

    /// Sufijo de archivo que usa cada variante (`team_block_copper.png`).
    fn suffix(self) -> &'static str {
        match self {
            Self::Copper => "copper",
            Self::Cobalt => "cobalt",
        }
    }
}

/// Una capa declarada en el manifiesto.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ManifestLayer {
    /// Índice de la capa en el texture array (consecutivo desde 0).
    pub layer: u32,
    /// Identificador estable del bloque.
    pub block_id: u8,
    /// Nombre del bloque en `snake_case` (p. ej. `team_block`).
    pub block_name: String,
    /// Variante de equipo, si el bloque tiene dos texturas.
    pub variant: Option<TeamVariant>,
    /// Fotograma de animación dentro del bloque (la lava usa 0..=3).
    pub frame: u8,
}

/// Atlas cargado desde el manifiesto: capas y texturas emparejadas por índice.
pub struct LoadedAtlas {
    /// Lado común de todas las texturas (16 o 32).
    pub tile_size: u32,
    /// Capas del manifiesto, en su orden.
    pub layers: Vec<ManifestLayer>,
    /// Textura RGBA de cada capa, en el mismo orden que `layers`.
    pub textures: Vec<TextureData>,
}

/// Parsea el manifiesto incrustado; las capas deben ser consecutivas desde 0.
pub fn parse_manifest() -> Result<Vec<ManifestLayer>, BlockTextureError> {
    parse_layers(MANIFEST_RON)
}

/// Carga el atlas del manifiesto: decodifica los PNG incrustados en el orden
/// exacto de las capas. El tamaño de tile sale de la primera textura (cuadrada,
/// potencia de dos, de lado 16 o 32) y todas las capas deben medir lo mismo.
pub fn load_manifest_atlas() -> Result<LoadedAtlas, BlockTextureError> {
    let wrong_size = |file, width, height| BlockTextureError::WrongSize {
        file,
        width,
        height,
    };
    let layers = parse_manifest()?;
    let mut tile_size = None;
    let mut textures = Vec::with_capacity(layers.len());
    for (index, layer) in layers.iter().enumerate() {
        let Some(&(file, bytes)) = EMBEDDED.get(index) else {
            return Err(BlockTextureError::MissingTexture {
                block_name: layer.block_name.clone(),
            });
        };
        let framed = layers
            .iter()
            .any(|other| other.block_name == layer.block_name && other.frame != 0);
        if expected_file(layer, framed) != file {
            return Err(BlockTextureError::MissingTexture {
                block_name: layer.block_name.clone(),
            });
        }
        let rgba = image::load_from_memory(bytes)
            .map_err(|_| BlockTextureError::Undecodable { file })?
            .to_rgba8();
        let (width, height) = rgba.dimensions();
        let side = match tile_size {
            Some(side) => side,
            None => {
                if width != height || !width.is_power_of_two() || !matches!(width, 16 | 32) {
                    return Err(wrong_size(file, width, height));
                }
                tile_size = Some(width);
                width
            }
        };
        if width != side || height != side {
            return Err(wrong_size(file, width, height));
        }
        textures.push(TextureData {
            width,
            height,
            rgba: rgba.into_raw(),
        });
    }
    let Some(tile_size) = tile_size else {
        // `parse_layers` ya rechaza los manifiestos sin capas.
        return Err(BlockTextureError::InvalidManifest {
            reason: "el manifiesto no declara capas".to_owned(),
        });
    };
    Ok(LoadedAtlas {
        tile_size,
        layers,
        textures,
    })
}

impl LoadedAtlas {
    /// Capa del texture array para un bloque con su variante de equipo y
    /// fotograma. La lava acepta fotogramas 0..=3; los bloques con variantes
    /// aceptan `Copper`/`Cobalt` con fotograma 0 (sin variante, cobre, que es
    /// el valor por defecto del motor); el resto solo `(None, 0)`.
    pub fn layer_for(
        &self,
        block: BlockKind,
        variant: Option<TeamVariant>,
        frame: u8,
    ) -> Option<u32> {
        let wanted = match block {
            BlockKind::Lava => {
                if variant.is_some() || frame > 3 {
                    return None;
                }
                (None, frame)
            }
            BlockKind::TeamBlock
            | BlockKind::ForceField
            | BlockKind::Beacon
            | BlockKind::BucketStation
            | BlockKind::BucketPost => {
                if frame != 0 {
                    return None;
                }
                (Some(variant.unwrap_or(TeamVariant::Copper)), 0)
            }
            _ => {
                if variant.is_some() || frame != 0 {
                    return None;
                }
                (None, 0)
            }
        };
        self.layers
            .iter()
            .find(|layer| {
                layer.block_id == block.id() && layer.variant == wanted.0 && layer.frame == wanted.1
            })
            .map(|layer| layer.layer)
    }
}

/// Nombre de archivo que el generador asigna a una capa: `nombre_equipo.png`
/// si tiene variantes, `nombre_frame.png` si tiene fotogramas, o `nombre.png`.
fn expected_file(layer: &ManifestLayer, framed: bool) -> String {
    match layer.variant {
        Some(variant) => format!("{}_{}.png", layer.block_name, variant.suffix()),
        None if framed => format!("{}_{}.png", layer.block_name, layer.frame),
        None => format!("{}.png", layer.block_name),
    }
}

/// Interpreta el campo `variant`: `None` o `Some("copper")`/`Some("cobalt")`.
fn parse_variant(value: &str) -> Option<Option<TeamVariant>> {
    if value == "None" {
        return Some(None);
    }
    let name = value
        .strip_prefix("Some(")
        .and_then(|rest| rest.strip_suffix(')'))
        .map(|inner| inner.trim_matches('"'))?;
    Some(Some(TeamVariant::parse(name)?))
}

/// Parsea el texto de un manifiesto a mano, sin dependencias nuevas.
fn parse_layers(text: &str) -> Result<Vec<ManifestLayer>, BlockTextureError> {
    let invalid = |reason: &str| BlockTextureError::InvalidManifest {
        reason: reason.to_owned(),
    };
    let mut layers = Vec::new();
    for (number, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") || line == "[" || line == "]" {
            continue;
        }
        let bad = |detail: &str| BlockTextureError::InvalidManifest {
            reason: format!("línea {}: {detail}", number + 1),
        };
        let body = line
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix("),").or_else(|| rest.strip_suffix(')')));
        let Some(body) = body else {
            return Err(bad(&format!("entrada mal cerrada: '{line}'")));
        };
        let mut layer = None;
        let mut block_id = None;
        let mut block_name = None;
        let mut variant = None;
        let mut frame = None;
        for field in body.split(',') {
            let Some((key, value)) = field.trim().split_once(':') else {
                return Err(bad(&format!("campo sin 'clave: valor': '{field}'")));
            };
            let value = value.trim();
            match key.trim() {
                "layer" => layer = Some(value.parse().map_err(|_| bad("'layer' no es u32"))?),
                "block_id" => {
                    block_id = Some(value.parse().map_err(|_| bad("'block_id' no es u8"))?);
                }
                "block_name" => {
                    block_name = Some(
                        value
                            .strip_prefix('"')
                            .and_then(|rest| rest.strip_suffix('"'))
                            .ok_or_else(|| bad("'block_name' debe ir entre comillas"))?
                            .to_owned(),
                    );
                }
                "variant" => {
                    variant =
                        Some(parse_variant(value).ok_or_else(|| bad("variante desconocida"))?);
                }
                "frame" => frame = Some(value.parse().map_err(|_| bad("'frame' no es u8"))?),
                _ => return Err(bad(&format!("campo desconocido: '{key}'"))),
            }
        }
        if let (Some(layer), Some(block_id), Some(block_name), Some(variant), Some(frame)) =
            (layer, block_id, block_name, variant, frame)
        {
            layers.push(ManifestLayer {
                layer,
                block_id,
                block_name,
                variant,
                frame,
            });
        } else {
            return Err(bad("falta algún campo"));
        }
    }
    if layers.is_empty() {
        return Err(invalid("el manifiesto no declara capas"));
    }
    for (index, layer) in layers.iter().enumerate() {
        if layer.layer != index as u32 {
            return Err(invalid("las capas deben ser consecutivas desde 0"));
        }
    }
    Ok(layers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_declares_25_consecutive_layers() {
        let layers = parse_manifest().expect("parsear el manifiesto");
        assert_eq!(layers.len(), 25);
        for (index, layer) in layers.iter().enumerate() {
            assert_eq!(layer.layer, index as u32);
        }
        assert_eq!(layers[0].block_name, "dirt");
        assert_eq!(layers[6].block_id, 7);
        assert_eq!(layers[6].block_name, "lava");
        assert_eq!(layers[6].frame, 0);
        assert_eq!(layers[11].block_name, "team_block");
        assert_eq!(layers[11].variant, Some(TeamVariant::Cobalt));
    }

    #[test]
    fn atlas_loads_25_layers_of_16() {
        let atlas = load_manifest_atlas().expect("cargar el atlas");
        assert_eq!(atlas.tile_size, 16);
        assert_eq!(atlas.layers.len(), 25);
        assert_eq!(atlas.textures.len(), 25);
        for texture in &atlas.textures {
            assert_eq!((texture.width, texture.height), (16, 16));
            assert_eq!(texture.rgba.len(), 16 * 16 * 4);
        }
    }

    #[test]
    fn layer_for_distinguishes_team_variants() {
        let atlas = load_manifest_atlas().expect("cargar el atlas");
        let cases = [
            (BlockKind::TeamBlock, 10, 11),
            (BlockKind::ForceField, 12, 13),
            (BlockKind::Beacon, 18, 19),
            (BlockKind::BucketStation, 21, 22),
            (BlockKind::BucketPost, 23, 24),
        ];
        let of = |block: BlockKind, variant| atlas.layer_for(block, variant, 0);
        for (block, copper, cobalt) in cases {
            assert_eq!(of(block, Some(TeamVariant::Copper)), Some(copper));
            assert_eq!(of(block, Some(TeamVariant::Cobalt)), Some(cobalt));
            // Sin variante, la capa cobre es el valor por defecto del motor.
            assert_eq!(of(block, None), Some(copper));
            assert_eq!(atlas.layer_for(block, Some(TeamVariant::Cobalt), 1), None);
        }
    }

    #[test]
    fn layer_for_resolves_lava_frames_and_plain_blocks() {
        let atlas = load_manifest_atlas().expect("cargar el atlas");
        for frame in 0..=3u8 {
            assert_eq!(
                atlas.layer_for(BlockKind::Lava, None, frame),
                Some(6 + u32::from(frame))
            );
        }
        assert_eq!(
            atlas.layer_for(BlockKind::Lava, Some(TeamVariant::Copper), 2),
            None
        );
        assert_eq!(atlas.layer_for(BlockKind::Lava, None, 4), None);
        assert_eq!(atlas.layer_for(BlockKind::Dirt, None, 0), Some(0));
        // Combinaciones inválidas en bloques sin variante.
        assert_eq!(
            atlas.layer_for(BlockKind::Dirt, Some(TeamVariant::Cobalt), 0),
            None
        );
        assert_eq!(atlas.layer_for(BlockKind::Dirt, None, 1), None);
        assert_eq!(atlas.layer_for(BlockKind::Air, None, 0), None);
    }

    #[test]
    fn malformed_manifests_are_rejected() {
        let valid = "(layer: 0, block_id: 1, block_name: \"dirt\", variant: None, frame: 0),";
        assert!(parse_layers(valid).is_ok());
        // Falta el campo `frame` y la entrada va mal cerrada.
        assert!(parse_layers("(layer: 0, block_id: 1, block_name: \"dirt\"),").is_err());
        assert!(parse_layers(valid.trim_end_matches("),")).is_err());
        // Variante desconocida y capas que no empiezan en 0.
        assert!(parse_layers(&valid.replace("None", "Some(\"gold\")")).is_err());
        assert!(parse_layers(&valid.replace("layer: 0", "layer: 1")).is_err());
    }
}
