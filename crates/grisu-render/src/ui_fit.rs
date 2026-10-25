//! Encajar texto en un ancho: recortar con puntos suspensivos o partir en
//! líneas por palabras. Mide con la misma fuente que dibuja (`UiBatch`).

use crate::UiBatch;

/// `text` tal cual si cabe en `max_w`; si no, recortado y terminado en «…».
pub fn fit_text(text: &str, scale: f32, max_w: f32) -> String {
    if UiBatch::text_width(scale, text) <= max_w {
        return text.to_owned();
    }
    let mut cut: String = text.to_owned();
    while !cut.is_empty() {
        cut.pop();
        let candidate = format!("{}…", cut.trim_end());
        if UiBatch::text_width(scale, &candidate) <= max_w {
            return candidate;
        }
    }
    String::new()
}

/// Parte `text` en líneas que caben en `max_w`, sin cortar palabras salvo
/// que una sola no quepa (entonces se recorta).
pub fn wrap_text(text: &str, scale: f32, max_w: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if UiBatch::text_width(scale, &candidate) <= max_w || current.is_empty() {
            current = candidate;
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_owned();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
        .into_iter()
        .map(|l| fit_text(&l, scale, max_w))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_short_text_and_cuts_long_text() {
        assert_eq!(fit_text("Roca", 2.0, 500.0), "Roca");
        let long = "target/release/hushmine.exe · la más rápida";
        let cut = fit_text(long, 2.0, 200.0);
        assert!(cut.ends_with('…'));
        assert!(UiBatch::text_width(2.0, &cut) <= 200.0);
        assert_eq!(fit_text("abc", 2.0, 0.0), "");
    }

    #[test]
    fn wrap_breaks_on_words_and_every_line_fits() {
        let text = "Minería competitiva en vóxeles: dos cuadrillas roban oro a una montaña";
        let lines = wrap_text(text, 2.0, 260.0);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(UiBatch::text_width(2.0, line) <= 260.0, "{line}");
        }
        assert_eq!(lines.join(" "), text);
        assert!(wrap_text("", 2.0, 100.0).is_empty());
    }
}
