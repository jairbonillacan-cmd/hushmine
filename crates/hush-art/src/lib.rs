//! Texturas y luz de los bloques de HUSHMINE.
//!
//! Las texturas reales son los PNG de `assets/textures/blocks`, incrustados
//! en el binario; el manifiesto de capas (`manifest.ron`) gobierna el tamaño
//! y el orden del texture array. La emisión de luz tiene versión escalar en
//! [`emission_table`] y versión en color en [`emission_rgb_table`]. El
//! generador provisional por código se conserva en
//! [`procedural_block_textures`] por si hace falta para pruebas.

mod embedded;
mod light_color;
mod manifest;
mod procedural;

pub use embedded::{BlockTextureError, LAVA_FRAMES, LAVA_LAYER, block_textures, texture_file};
pub use light_color::{RgbLight, emission_rgb_table};
pub use manifest::{LoadedAtlas, ManifestLayer, TeamVariant, load_manifest_atlas, parse_manifest};
pub use procedural::{procedural_block_texture, procedural_block_textures};

use hush_sim::BlockKind;

/// Bloques con textura: todos menos el aire, en orden de identificador.
pub fn textured_blocks() -> &'static [BlockKind] {
    &BlockKind::ALL[1..]
}

/// Luz que emite cada bloque (0-15), indexada por su valor.
pub fn emission_table() -> [u8; 256] {
    let mut table = [0; 256];
    table[usize::from(BlockKind::Lava.id())] = 13;
    table
}
