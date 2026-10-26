//! Texturas y luz de los bloques de HUSHMINE.
//!
//! Las texturas reales son los PNG de `assets/textures/blocks`, incrustados
//! en el binario; el manifiesto de capas (`manifest.ron`) gobierna el tamaño
//! y el orden del texture array. Los bloques del modo Historia (18 en
//! adelante) aún no tienen PNG: se generan por código y se añaden como capas
//! extra al final del array (ver [`all_block_textures`]). La emisión de luz
//! tiene versión escalar en [`emission_table`] (la que usa el juego) y
//! versión en color en [`emission_rgb_table`] (preparada, aún sin conectar a
//! la luz). El generador provisional completo se conserva en
//! [`procedural_block_textures`] por si hace falta para pruebas.
//!
//! También guarda la paleta de la interfaz ([`ui_palette`]), que comparten
//! el lanzador, el menú y el HUD.

mod embedded;
mod light_color;
mod manifest;
mod procedural;
pub mod ui_palette;

pub use embedded::{BlockTextureError, LAVA_FRAMES, LAVA_LAYER, block_textures, texture_file};
pub use light_color::{RgbLight, emission_rgb_table};
pub use manifest::{LoadedAtlas, ManifestLayer, TeamVariant, load_manifest_atlas, parse_manifest};
pub use procedural::{procedural_block_texture, procedural_block_textures};

use hush_sim::BlockKind;

/// Bloques con textura: todos menos el aire, en orden de identificador.
pub fn textured_blocks() -> &'static [BlockKind] {
    &BlockKind::ALL[1..]
}

/// Bloques con PNG real incrustado (1..=17), en orden de identificador.
pub fn embedded_blocks() -> &'static [BlockKind] {
    &BlockKind::ALL[1..18]
}

/// Bloques del modo historia (18..=27), en orden de identificador.
pub fn story_blocks() -> &'static [BlockKind] {
    &BlockKind::ALL[18..]
}

/// Primera capa de los bloques del modo historia en el array de
/// [`all_block_textures`]: 17 bloques con PNG más los fotogramas 1-3 de la
/// lava. Debe coincidir con `STORY_LAYER_BASE` en `voxel.wgsl`.
pub const STORY_LAYER_BASE: u32 = 20;

/// Texturas de todos los bloques listas para el texture array del terreno:
/// los PNG incrustados (bloques 1..=17 y fotogramas extra de la lava) y,
/// al final, los bloques del modo historia generados por código.
pub fn all_block_textures() -> Result<Vec<grisu_render::TextureData>, BlockTextureError> {
    let mut layers = block_textures()?;
    layers.extend(story_blocks().iter().map(|&b| procedural_block_texture(b)));
    Ok(layers)
}

/// Luz que emite cada bloque (0-15), indexada por su valor. Niveles del
/// modo historia: antorcha 14, lava 15.
pub fn emission_table() -> [u8; 256] {
    let mut table = [0; 256];
    table[usize::from(BlockKind::Lava.id())] = 15;
    table[usize::from(BlockKind::Torch.id())] = 14;
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_block_textures_covers_every_solid_block() {
        let textures = all_block_textures().unwrap();
        // 17 PNG + 3 fotogramas de lava + 11 bloques de historia.
        assert_eq!(textures.len(), 31);
        assert!(
            textures
                .iter()
                .all(|t| t.width == 16 && t.rgba.len() == 1024)
        );
    }

    #[test]
    fn story_layers_start_after_lava_frames() {
        // 17 bloques con textura propia (el aire no cuenta) + 3 fotogramas.
        assert_eq!(
            STORY_LAYER_BASE,
            (BlockKind::ALL.len() - story_blocks().len() - 1) as u32 + LAVA_FRAMES - 1
        );
    }

    #[test]
    fn emission_matches_story_document() {
        let table = emission_table();
        assert_eq!(table[usize::from(BlockKind::Torch.id())], 14);
        assert_eq!(table[usize::from(BlockKind::Lava.id())], 15);
    }
}
