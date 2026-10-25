//! Colores de la interfaz (lanzador, menú y HUD), tomados de la paleta del
//! proyecto (`tools/ui/ui_palette.py` y `tools/textures/palette.py`).
//!
//! Están en lineal porque las superficies son sRGB; el nombre de cada
//! constante es el de la paleta y el comentario da su hex sRGB. Reglas de uso
//! (ver `docs/ui/guia-interfaz.md`):
//!
//! - **Un único acento**: la lámpara de sodio, para la acción principal.
//! - **El latón marca la selección** (la ficha de 1931 junto a la fila).
//! - **El fósforo**, dentro de los monitores y para los mensajes buenos.
//! - **Texto pequeño** en `ENAMEL_CREAM` o `PAPER_AGED`; `STEEL_LIGHT` solo
//!   en texto grande, porque no llega a 4,5:1 sobre la pared.

/// Color RGBA lineal.
pub type Color = [f32; 4];

const fn opaque(rgb: [f32; 3]) -> Color {
    [rgb[0], rgb[1], rgb[2], 1.0]
}

/// Mismo color con otra opacidad.
pub const fn with_alpha(c: Color, a: f32) -> Color {
    [c[0], c[1], c[2], a]
}

/// `#0A0B0E`
pub const VOID_BLACK: Color = opaque([0.0030, 0.0033, 0.0044]);
/// `#14161B`: la pared de la caseta.
pub const SOOT_DARK: Color = opaque([0.0070, 0.0080, 0.0110]);
/// `#292E38`: líneas finas y bisel de los monitores.
pub const STEEL_DARK: Color = opaque([0.0222, 0.0273, 0.0395]);
/// `#6C778C`: texto grande secundario.
pub const STEEL_LIGHT: Color = opaque([0.1500, 0.1845, 0.2623]);
/// `#EDE6D3`: texto principal.
pub const ENAMEL_CREAM: Color = opaque([0.8469, 0.7913, 0.6514]);
/// `#D8CEB8`: texto pequeño secundario.
pub const PAPER_AGED: Color = opaque([0.6867, 0.6172, 0.4793]);
/// `#9E7B2F`: cuerpo de la ficha de latón.
pub const BRASS_MID: Color = opaque([0.3419, 0.1981, 0.0284]);
/// `#D9B052`: canto de la ficha y foco del teclado.
pub const BRASS_BRIGHT: Color = opaque([0.6939, 0.43427, 0.0844]);
/// `#FAAE3C`: la lámpara de sodio, el único acento.
pub const SODIUM_WARM: Color = opaque([0.9560, 0.4233, 0.0452]);
/// `#1C1917`: tinta sobre el sodio.
pub const INK_STAMP: Color = opaque([0.0116, 0.0097, 0.0086]);
/// `#07120A`: fondo de los monitores.
pub const CRT_BG: Color = opaque([0.0021, 0.0060, 0.0030]);
/// `#1E6F36`
pub const PHOSPHOR_DIM: Color = opaque([0.0130, 0.1590, 0.0369]);
/// `#34B85A`: estado correcto.
pub const PHOSPHOR_MID: Color = opaque([0.0343, 0.4793, 0.1022]);
/// `#68F08E`: lectura principal de los monitores.
pub const PHOSPHOR_BRIGHT: Color = opaque([0.1384, 0.8714, 0.2705]);
/// `#F28A2E`: avisos y errores (7:1 sobre la pared, legible en pequeño).
pub const LAVA_ORANGE: Color = opaque([0.8879, 0.2542, 0.0273]);

/// Pared de la caseta: casi opaca para que el texto no compita con el fondo.
pub const WALL: Color = with_alpha(SOOT_DARK, 0.94);
/// Velo sobre el mundo: la noche en el monte.
pub const NIGHT_VEIL: Color = with_alpha(VOID_BLACK, 0.6);

/// Ficha de latón de 1931 que marca la fila elegida: una placa octogonal de
/// 12 px con el agujero por donde colgaba de su alcayata.
pub fn draw_tally(batch: &mut grisu_render::UiBatch, x: f32, center_y: f32) {
    let s = 12.0;
    let y = (center_y - s * 0.5).round();
    batch.rect(x + 2.0, y, s - 4.0, s, BRASS_MID);
    batch.rect(x, y + 2.0, s, s - 4.0, BRASS_MID);
    batch.rect(x + 2.0, y, s - 4.0, 2.0, BRASS_BRIGHT);
    batch.rect(x + s * 0.5 - 1.0, y + 3.0, 2.0, 2.0, SOOT_DARK);
}

/// Interpola en línea recta entre dos colores.
pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Luminancia relativa WCAG de un color lineal.
pub fn luminance(c: Color) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Razón de contraste WCAG entre dos colores lineales.
pub fn contrast(a: Color, b: Color) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_text_colours_pass_wcag_aa_on_the_wall() {
        for (name, colour) in [
            ("enamel_cream", ENAMEL_CREAM),
            ("paper_aged", PAPER_AGED),
            ("phosphor_mid", PHOSPHOR_MID),
            ("lava_orange", LAVA_ORANGE),
        ] {
            let ratio = contrast(colour, SOOT_DARK);
            assert!(ratio >= 4.5, "{name}: {ratio:.2}:1");
        }
        // El gris acero solo vale para texto grande (3:1).
        assert!(contrast(STEEL_LIGHT, SOOT_DARK) >= 3.0);
        // La tinta sobre el sodio del botón principal.
        assert!(contrast(INK_STAMP, SODIUM_WARM) >= 7.0);
        // El fósforo sobre el fondo del monitor.
        assert!(contrast(PHOSPHOR_BRIGHT, CRT_BG) >= 7.0);
    }

    #[test]
    fn lerp_color_interpolates_each_channel() {
        let mid = lerp_color([0.0, 0.0, 0.0, 1.0], [1.0, 2.0, 4.0, 1.0], 0.5);
        assert_eq!(mid, [0.5, 1.0, 2.0, 1.0]);
        assert_eq!(lerp_color([0.0; 4], [1.0; 4], 2.0), [1.0; 4]);
        assert_eq!(lerp_color([0.0; 4], [1.0; 4], -1.0), [0.0; 4]);
    }
}
