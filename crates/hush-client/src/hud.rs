//! Dibujo 2D del juego: el HUD en partida y el menú (nueva partida, pausa y
//! controles).
//!
//! El menú usa la misma pared de la caseta que el lanzador: una columna
//! oscura a la izquierda y el mundo visible a la derecha bajo un velo. El HUD
//! se queda en lo mínimo: los seis bloques al pie y si se está corriendo.

use grisu_render::{CameraMode, UiBatch, UiRect, fit_text};
use hush_art::ui_palette::*;
use hush_sim::BlockKind;

use crate::controls::Controls;
use crate::menu::MenuState;

/// Texto corriente (14 px).
const TEXT_BODY: f32 = 2.0;
/// Título de la página del menú.
const TEXT_HEADING: f32 = 4.0;
/// Margen exterior.
const GUTTER: f32 = 40.0;
/// Primera fila del menú, bajo el título y su línea.
const ROWS_TOP: f32 = GUTTER + TEXT_HEADING * 7.0 + 40.0;
/// Hueco que se reserva al pie de la pared para los atajos.
const FOOTER: f32 = 90.0;

/// Bloques del 1 al 6, con el nombre que se ve en pantalla.
const SLOTS: [(BlockKind, &str); 6] = [
    (BlockKind::Dirt, "Tierra"),
    (BlockKind::Rock, "Roca"),
    (BlockKind::HardRock, "Roca dura"),
    (BlockKind::Ore, "Mena"),
    (BlockKind::Gold, "Oro"),
    (BlockKind::Diamond, "Diamante"),
];

/// Ancho de la pared del menú para una ventana de ancho `w`.
fn wall_width(w: f32) -> f32 {
    // Más ancha que la del lanzador: aquí cada fila lleva etiqueta y valor.
    (w * 0.5).clamp(360.0, 560.0).min(w)
}

/// Rectángulo de la fila `idx` de una página con `count` filas. Lo usan el
/// dibujo y el ratón, así que nunca discrepan.
pub fn row_rect(count: usize, idx: usize, w: f32, h: f32) -> UiRect {
    let wall_w = wall_width(w);
    let room = (h - ROWS_TOP - FOOTER).max(0.0);
    let row_h = (room / count.max(1) as f32).floor().clamp(24.0, 40.0);
    let x = GUTTER - 16.0;
    UiRect::new(
        x,
        ROWS_TOP + idx as f32 * row_h,
        wall_w - 2.0 * x,
        row_h,
        [0.0; 4],
    )
}

pub fn draw_ui(
    menu: &MenuState,
    batch: &mut UiBatch,
    size: [u32; 2],
    controls: &Controls,
    camera_mode: CameraMode,
    sprinting: bool,
    place_block: BlockKind,
) {
    let [w, h] = [size[0].max(1) as f32, size[1].max(1) as f32];
    if menu.is_open() {
        draw_menu(menu, batch, w, h, controls, camera_mode);
    } else {
        draw_hud(batch, w, h, sprinting, place_block);
    }
}

fn draw_menu(
    menu: &MenuState,
    batch: &mut UiBatch,
    w: f32,
    h: f32,
    controls: &Controls,
    camera_mode: CameraMode,
) {
    let wall_w = wall_width(w);
    batch.rect(0.0, 0.0, w, h, NIGHT_VEIL);
    batch.rect(0.0, 0.0, wall_w, h, WALL);
    batch.rect(wall_w - 1.0, 0.0, 1.0, h, STEEL_DARK);

    batch.text(GUTTER, GUTTER, TEXT_HEADING, ENAMEL_CREAM, menu.title());
    batch.rect(
        GUTTER,
        ROWS_TOP - 16.0,
        wall_w - 2.0 * GUTTER,
        1.0,
        STEEL_DARK,
    );

    let rows = menu.item_rows(controls, camera_mode);
    let line = UiBatch::text_height(TEXT_BODY);
    for (idx, (label, value)) in rows.iter().enumerate() {
        let r = row_rect(rows.len(), idx, w, h);
        let [x, y, rw, rh] = r.rect;
        let ty = y + ((rh - line) * 0.5).round();
        let selected = idx == menu.selected;
        if selected {
            draw_tally(batch, x + 16.0, ty + line * 0.5);
        }
        let colour = if selected { ENAMEL_CREAM } else { PAPER_AGED };
        let shown = if value.is_empty() {
            String::new()
        } else if selected && menu.is_adjustable(idx) {
            format!("‹ {value} ›")
        } else {
            value.clone()
        };
        let vw = UiBatch::text_width(TEXT_BODY, &shown);
        batch.text(x + rw - 16.0 - vw, ty, TEXT_BODY, ENAMEL_CREAM, &shown);
        // La etiqueta nunca pisa el valor: si no cabe, se recorta.
        let label_room = rw - 40.0 - 16.0 - vw - 20.0;
        batch.text(
            x + 40.0,
            ty,
            TEXT_BODY,
            colour,
            &fit_text(label, TEXT_BODY, label_room),
        );
    }

    let hints: &[(&str, &str)] = if menu.rebinding.is_some() {
        &[("", "Pulsa la tecla nueva"), ("Esc", "cancelar")]
    } else {
        &[
            ("W S", "elegir"),
            ("A D", "cambiar"),
            ("Enter", "aceptar"),
            ("Esc", "volver"),
        ]
    };
    draw_hints(batch, GUTTER, h - GUTTER, wall_w - 2.0 * GUTTER, hints);
}

/// Atajos en pares tecla y acción, partidos en líneas que caben en `max_w`
/// y apoyados sobre `bottom`.
fn draw_hints(batch: &mut UiBatch, x: f32, bottom: f32, max_w: f32, hints: &[(&str, &str)]) {
    let line = UiBatch::text_height(TEXT_BODY);
    let pair_w = |(key, action): &(&str, &str)| {
        let key_w = if key.is_empty() {
            0.0
        } else {
            UiBatch::text_width(TEXT_BODY, key) + 8.0
        };
        key_w + UiBatch::text_width(TEXT_BODY, action)
    };
    // Primero se reparten en líneas para saber cuántas hay y subir el bloque.
    let mut lines: Vec<Vec<&(&str, &str)>> = vec![Vec::new()];
    let mut used = 0.0;
    for hint in hints {
        let pw = pair_w(hint);
        let current = lines.last_mut().expect("siempre hay una línea");
        if !current.is_empty() && used + pw > max_w {
            lines.push(vec![hint]);
            used = pw + 20.0;
        } else {
            current.push(hint);
            used += pw + 20.0;
        }
    }
    let mut y = bottom - lines.len() as f32 * (line + 10.0) + 10.0;
    for pairs in lines {
        let mut pen = x;
        for (key, action) in pairs {
            if !key.is_empty() {
                batch.text(pen, y, TEXT_BODY, ENAMEL_CREAM, key);
                pen += UiBatch::text_width(TEXT_BODY, key) + 8.0;
            }
            batch.text(pen, y, TEXT_BODY, PAPER_AGED, action);
            pen += UiBatch::text_width(TEXT_BODY, action) + 20.0;
        }
        y += line + 10.0;
    }
}

/// Texto con una sombra de un píxel para leerse sobre el mundo sin caja.
pub(crate) fn shadowed(batch: &mut UiBatch, x: f32, y: f32, colour: Color, text: &str) {
    batch.text(
        x + 2.0,
        y + 2.0,
        TEXT_BODY,
        with_alpha(VOID_BLACK, 0.85),
        text,
    );
    batch.text(x, y, TEXT_BODY, colour, text);
}

fn draw_hud(batch: &mut UiBatch, w: f32, h: f32, sprinting: bool, place_block: BlockKind) {
    // Una sola franja detrás de los seis bloques, no una caja por bloque.
    let pad = 18.0;
    let gap = 28.0;
    let labels: Vec<String> = SLOTS
        .iter()
        .enumerate()
        .map(|(i, (_, name))| format!("{} {name}", i + 1))
        .collect();
    let widths: Vec<f32> = labels
        .iter()
        .map(|l| UiBatch::text_width(TEXT_BODY, l))
        .collect();
    let total = widths.iter().sum::<f32>() + gap * (SLOTS.len() - 1) as f32;
    let line = UiBatch::text_height(TEXT_BODY);
    let strip_h = line + 2.0 * pad;
    let strip_x = ((w - total) * 0.5 - pad).round();
    let strip_y = (h - strip_h - 16.0).round();
    batch.rect(
        strip_x,
        strip_y,
        total + 2.0 * pad,
        strip_h,
        with_alpha(SOOT_DARK, 0.82),
    );
    let mut pen = strip_x + pad;
    let ty = strip_y + pad;
    for (i, (kind, _)) in SLOTS.iter().enumerate() {
        let selected = *kind == place_block;
        let colour = if selected { ENAMEL_CREAM } else { PAPER_AGED };
        batch.text(pen, ty, TEXT_BODY, colour, &labels[i]);
        if selected {
            batch.rect(pen, ty + line + 5.0, widths[i], 2.0, BRASS_BRIGHT);
        }
        pen += widths[i] + gap;
    }

    if sprinting {
        let label = "Corriendo";
        let lw = UiBatch::text_width(TEXT_BODY, label);
        shadowed(
            batch,
            strip_x + total + 2.0 * pad - lw,
            strip_y - line - 12.0,
            ENAMEL_CREAM,
            label,
        );
    }
    shadowed(batch, 20.0, 20.0, PAPER_AGED, "Esc  menú");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_stay_inside_the_wall_and_above_the_hints() {
        for (w, h) in [(1280.0, 720.0), (800.0, 600.0), (1920.0, 1080.0)] {
            for count in [6, 7, 11] {
                let last = row_rect(count, count - 1, w, h);
                assert!(
                    last.rect[1] + last.rect[3] <= h - FOOTER + 0.5,
                    "{w}x{h} {count}"
                );
                let first = row_rect(count, 0, w, h);
                assert!(first.rect[0] >= 0.0);
                assert!(first.rect[0] + first.rect[2] <= wall_width(w));
                assert!(first.rect[1] >= ROWS_TOP);
            }
        }
    }

    #[test]
    fn rows_do_not_overlap() {
        let a = row_rect(7, 2, 1280.0, 720.0);
        let b = row_rect(7, 3, 1280.0, 720.0);
        assert!(a.rect[1] + a.rect[3] <= b.rect[1]);
        assert!(!a.contains(b.rect[0] + 4.0, b.rect[1] + 4.0));
    }
}
