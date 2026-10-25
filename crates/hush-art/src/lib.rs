//! Texturas de 16×16 de los bloques de HUSHMINE.
//!
//! Las texturas reales son los PNG de `assets/textures/blocks` (generados
//! por `tools/textures/build_all.py`), incrustados en el binario: la capa
//! `i` corresponde al bloque `i + 1`. Los bloques del modo historia
//! (18..=28) tienen PNG propio pero se añaden como capas extra tras los
//! fotogramas de la lava (ver [`all_block_textures`] y
//! [`STORY_LAYER_BASE`]). El generador provisional por código se conserva
//! en [`procedural_block_textures`] por si hace falta para pruebas.

mod embedded;
mod procedural;

pub use embedded::{
    BlockTextureError, LAVA_FRAMES, LAVA_LAYER, block_textures, story_block_textures, texture_file,
};
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

/// Bloques del modo historia (18..=28), en orden de identificador.
pub fn story_blocks() -> &'static [BlockKind] {
    &BlockKind::ALL[18..]
}

/// Primera capa de los bloques del modo historia en el array de
/// [`all_block_textures`]: 17 bloques con PNG más los fotogramas 1-3 de la
/// lava. Debe coincidir con `STORY_LAYER_BASE` en `voxel.wgsl`.
pub const STORY_LAYER_BASE: u32 = 20;

/// Texturas de todos los bloques listas para el texture array del terreno:
/// los PNG incrustados (bloques 1..=17 y fotogramas extra de la lava) y, al
/// final, los PNG de los bloques del modo historia (18..=28).
pub fn all_block_textures() -> Result<Vec<grisu_render::TextureData>, BlockTextureError> {
    let mut layers = block_textures()?;
    layers.extend(story_block_textures()?);
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
    fn story_layers_are_the_embedded_pngs() {
        let textures = all_block_textures().unwrap();
        let story = &textures[STORY_LAYER_BASE as usize..];
        assert_eq!(story.len(), story_blocks().len());
        for (layer, &block) in story.iter().zip(story_blocks()) {
            let fallback = procedural_block_texture(block);
            assert_ne!(
                layer.rgba, fallback.rgba,
                "la capa de {block:?} debería ser el PNG real, no la textura procedural",
            );
        }
    }

    #[test]
    fn emission_matches_story_document() {
        let table = emission_table();
        assert_eq!(table[usize::from(BlockKind::Torch.id())], 14);
        assert_eq!(table[usize::from(BlockKind::Lava.id())], 15);
    }
}
