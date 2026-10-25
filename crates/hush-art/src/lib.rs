//! Texturas de 16×16 de los bloques de HUSHMINE.
//!
//! Las texturas reales son los PNG de `assets/textures/blocks`, incrustados
//! en el binario: la capa `i` corresponde al bloque `i + 1`. El generador
//! provisional por código se conserva en [`procedural_block_textures`] por
//! si hace falta para pruebas.
//!
//! También guarda la paleta de la interfaz ([`ui_palette`]), que comparten
//! el lanzador, el menú y el HUD.

mod embedded;
mod procedural;
pub mod ui_palette;

pub use embedded::{BlockTextureError, LAVA_FRAMES, LAVA_LAYER, block_textures, texture_file};
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
