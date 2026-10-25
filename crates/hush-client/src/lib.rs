//! Partes del cliente que no necesitan ventana ni red: los controles, el
//! menú y su dibujo. Están en biblioteca para que `hush-cli` pueda capturar
//! el menú real sin abrir el juego.

pub mod controls;
pub mod hud;
pub mod menu;
