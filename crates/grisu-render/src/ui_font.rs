//! Fuente de mapa de bits del HUD y los menús de Grisú.
//!
//! Generada por la revisión del lanzador a partir de la fuente 5×7 anterior:
//! conserva sus mayúsculas y cifras y añade minúsculas con ascendentes y
//! descendentes, tildes, diéresis y eñe compuestas sobre la letra base, y
//! anchos proporcionales. Las cifras son de ancho fijo para que los
//! contadores no bailen.
//!
//! Cada glifo ocupa una rejilla de 11 filas: 2 para la tilde de las
//! mayúsculas, 7 de altura de mayúscula (la línea base queda debajo) y 2 de
//! descendente. Las minúsculas tienen 5 filas de altura x, así que su tilde
//! cabe dentro de la altura de mayúscula.

/// Filas por encima de la altura de mayúscula (tildes de las mayúsculas).
pub const ABOVE: usize = 2;
/// Altura de mayúscula, en filas.
pub const CAP_HEIGHT: usize = 7;
/// Filas por debajo de la línea base (descendentes).
pub const BELOW: usize = 2;
const ROWS: usize = ABOVE + CAP_HEIGHT + BELOW;
/// Ancho del espacio, en columnas.
const SPACE_WIDTH: u8 = 3;
/// Ancho fijo de las cifras, en columnas.
const DIGIT_WIDTH: u8 = 5;

/// Glifo listo para dibujar: la fila 0 es la más alta (zona de tildes) y
/// el bit 7 de cada fila es la columna 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub rows: [u8; ROWS],
    /// Columnas que ocupa (sin el espacio entre letras).
    pub width: u8,
}

impl Glyph {
    /// ¿Está encendido el píxel de la fila `row` y la columna `col`?
    pub fn pixel(&self, row: usize, col: usize) -> bool {
        col < 8 && self.rows[row] & (0x80 >> col) != 0
    }
}

#[derive(Clone, Copy)]
enum Accent {
    Acute,
    Diaeresis,
    Tilde,
}

/// Devuelve el glifo de un carácter. Los que no existen salen como un
/// rectángulo hueco para que se noten en las capturas.
pub fn glyph(ch: char) -> Glyph {
    if ch == ' ' {
        return Glyph {
            rows: [0; ROWS],
            width: SPACE_WIDTH,
        };
    }
    if let Some((base, accent)) = composed(ch) {
        return with_accent(glyph(base), accent, base.is_uppercase());
    }
    let tabular = ch.is_ascii_digit();
    match body(ch) {
        Some(rows) => from_rows(&rows, tabular),
        None => from_rows(&MISSING, false),
    }
}

/// Glifo de reserva para caracteres sin dibujo.
const MISSING: [&str; 9] = [
    "####", "#..#", "#..#", "#..#", "#..#", "#..#", "####", ".....", ".....",
];

fn from_rows(body: &[&str; 9], tabular: bool) -> Glyph {
    let mut rows = [0u8; ROWS];
    let mut lo = usize::MAX;
    let mut hi = 0usize;
    for (r, line) in body.iter().enumerate() {
        for (c, b) in line.bytes().enumerate() {
            if b == b'#' {
                rows[ABOVE + r] |= 0x80 >> c;
                lo = lo.min(c);
                hi = hi.max(c);
            }
        }
    }
    if tabular {
        return Glyph {
            rows,
            width: DIGIT_WIDTH,
        };
    }
    if lo == usize::MAX {
        return Glyph { rows, width: 1 };
    }
    // Se recorta la columna vacía de la izquierda para que el ancho sea
    // proporcional al dibujo.
    for row in &mut rows {
        *row <<= lo;
    }
    Glyph {
        rows,
        width: (hi - lo + 1) as u8,
    }
}

fn with_accent(mut g: Glyph, accent: Accent, upper: bool) -> Glyph {
    // Tilde de mayúscula en las 2 filas de arriba; de minúscula, encima de la
    // altura x.
    let top = if upper { 0 } else { ABOVE };
    let center = usize::from(g.width.saturating_sub(1)) / 2;
    let mut set = |row: usize, col: usize| {
        if col < 8 {
            g.rows[top + row] |= 0x80 >> col;
        }
    };
    match accent {
        Accent::Acute => {
            set(0, center + 1);
            set(1, center);
        }
        Accent::Diaeresis => {
            set(1, center.saturating_sub(1));
            set(1, center + 1);
        }
        Accent::Tilde => {
            let left = center.saturating_sub(1);
            set(0, left + 1);
            set(0, left + 3);
            set(1, left);
            set(1, left + 2);
        }
    }
    let used = (0..8)
        .rev()
        .find(|&c| g.rows.iter().any(|r| r & (0x80 >> c) != 0))
        .map_or(1, |c| c + 1);
    g.width = g.width.max(used as u8);
    g
}

fn composed(ch: char) -> Option<(char, Accent)> {
    match ch {
        'á' => Some(('a', Accent::Acute)),
        'é' => Some(('e', Accent::Acute)),
        'í' => Some(('ı', Accent::Acute)),
        'ó' => Some(('o', Accent::Acute)),
        'ú' => Some(('u', Accent::Acute)),
        'ü' => Some(('u', Accent::Diaeresis)),
        'ñ' => Some(('n', Accent::Tilde)),
        'Á' => Some(('A', Accent::Acute)),
        'É' => Some(('E', Accent::Acute)),
        'Í' => Some(('I', Accent::Acute)),
        'Ó' => Some(('O', Accent::Acute)),
        'Ú' => Some(('U', Accent::Acute)),
        'Ü' => Some(('U', Accent::Diaeresis)),
        'Ñ' => Some(('N', Accent::Tilde)),
        _ => None,
    }
}

use crate::ui_font_table::body;

#[cfg(test)]
mod tests {
    use super::*;

    const SPANISH: &str = "¿Versión rápida? ¡Sí! Ñandú, pingüino, «Vigía», ‹1931–1979›…";

    #[test]
    fn spanish_text_has_no_missing_glyphs() {
        for ch in SPANISH.chars().filter(|c| *c != ' ') {
            assert_ne!(
                glyph(ch).rows,
                from_rows(&MISSING, false).rows,
                "falta «{ch}»"
            );
        }
        for ch in ('a'..='z').chain('A'..='Z').chain('0'..='9') {
            assert_ne!(
                glyph(ch).rows,
                from_rows(&MISSING, false).rows,
                "falta «{ch}»"
            );
        }
    }

    #[test]
    fn accents_sit_above_the_letter_and_lowercase_descends() {
        let a = glyph('a');
        let a_acute = glyph('á');
        let upper = glyph('Á');
        // La tilde de la minúscula cae dentro de la altura de mayúscula.
        assert_eq!(a.rows[ABOVE], 0);
        assert_ne!(a_acute.rows[ABOVE], 0);
        // La de la mayúscula, por encima.
        assert_ne!(upper.rows[0] | upper.rows[1], 0);
        // Las descendentes bajan de la línea base y las demás no.
        let below = ABOVE + CAP_HEIGHT;
        assert_ne!(glyph('g').rows[below], 0);
        assert_eq!(glyph('a').rows[below], 0);
    }

    #[test]
    fn widths_are_proportional_but_digits_are_fixed() {
        assert!(glyph('i').width < glyph('m').width);
        assert_eq!(glyph('1').width, glyph('8').width);
        assert_eq!(glyph(' ').width, SPACE_WIDTH);
    }
}
