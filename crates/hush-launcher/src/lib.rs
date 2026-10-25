//! Biblioteca del launcher: estado, geometría y dibujo del panel, sin
//! ventana. El binario la usa con `grisu-platform`; `hush-cli` la usa para
//! las capturas de revisión.

pub mod draw;
pub mod history;
pub mod layout;
pub mod page_play;
pub mod panorama;
pub mod settings;
pub mod state;
#[cfg(test)]
pub(crate) mod testutil;
pub mod theme;
pub mod thumbnail;
pub mod versions;
