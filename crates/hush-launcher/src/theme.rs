//! Tipografía y métricas del lanzador; los colores vienen de
//! `hush_art::ui_palette`, compartidos con el menú y el HUD del juego.
//!
//! Dirección visual: la **caseta del Vigía** de `docs/ui/guia-interfaz.md`.
//! Una pared oscura a la izquierda con el rótulo y el menú, y a la derecha el
//! monte por la ventana (el panorama) con un monitor de fósforo verde que
//! muestra la partida que se va a jugar. Decisiones:
//!
//! - **Texto en minúsculas de frase**; las mayúsculas, solo en el rótulo.
//! - **Sin cajas alrededor de cada cosa**: jerarquía con tamaño, color y
//!   espacio. Las líneas finas separan filas; no hay bordes por botón.
//! - **Sin animaciones en bucle**: solo el fundido corto al cambiar de página.

pub use hush_art::ui_palette::*;

// Tamaños de texto (en píxeles de fuente; la fuente tiene 7 de alto). Se
// usan enteros para que los píxeles salgan nítidos.
/// Rótulo HUSHMINE.
pub const TEXT_TITLE: f32 = 6.0;
/// Encabezado de página.
pub const TEXT_HEADING: f32 = 4.0;
/// Entradas del menú y botón principal.
pub const TEXT_LARGE: f32 = 3.0;
/// Texto corriente y secundario (14 px).
pub const TEXT_BODY: f32 = 2.0;

/// Margen exterior de la pared y del contenido.
pub const GUTTER: f32 = 48.0;
/// Fracción del ancho que ocupa la pared de la caseta.
pub const WALL_FRACTION: f32 = 0.38;
/// Altura de una entrada del menú.
pub const NAV_ROW: f32 = 46.0;

/// Duración del fundido de entrada de una página, en segundos.
pub const FADE_TIME: f32 = 0.2;

/// Progreso del fundido en `[0, 1]`: 0 recién cambiada de página, 1 asentada.
pub fn fade_alpha(page_fade: f32) -> f32 {
    (page_fade / FADE_TIME).clamp(0.0, 1.0)
}

/// Velo que tapa el contenido mientras entra la página.
pub fn fade_cover(page_fade: f32) -> Color {
    with_alpha(SOOT_DARK, (1.0 - fade_alpha(page_fade)) * 0.6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_cover_starts_opaque_and_vanishes() {
        assert!((fade_cover(0.0)[3] - 0.6).abs() < 1e-6);
        assert_eq!(fade_cover(FADE_TIME)[3], 0.0);
        assert!((fade_cover(FADE_TIME * 0.5)[3] - 0.3).abs() < 1e-6);
        assert_eq!(fade_cover(FADE_TIME * 10.0)[3], 0.0);
    }
}
