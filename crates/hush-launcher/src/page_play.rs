//! Página Jugar: el monitor de fósforo con el mapa de la semilla, el selector
//! de versión, el botón de bajar al turno y dos enlaces secundarios.

use crate::draw::{draw_hints, hovered, tally};
use crate::layout::{Layout, line_top};
use crate::state::LauncherState;
use crate::theme::*;
use grisu_render::{UiBatch, UiRect};

/// Grosor del bisel de acero del monitor.
const BEZEL: f32 = 6.0;

pub fn draw_play(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    draw_monitor(batch, state, lay.monitor);
    draw_selector(batch, state, lay.items[0].rect);
    draw_play_button(batch, state, lay.items[1].rect);
    draw_link(
        batch,
        state,
        lay.items[2].rect,
        2,
        "Compilar esta versión",
        true,
    );
    draw_link(batch, state, lay.items[3].rect, 3, "Servidor LAN", false);

    // Los atajos, solo si caben debajo de los enlaces.
    let links = lay.items[3].rect.rect;
    let [_, cy, _, ch] = lay.content.rect;
    if links[1] + links[3] + 20.0 < cy + ch - UiBatch::text_height(TEXT_BODY) {
        draw_hints(
            batch,
            lay,
            &[
                ("W S", "elegir"),
                ("A D", "cambiar"),
                ("Enter", "jugar"),
                ("Tab", "página"),
            ],
        );
    }
}

/// Monitor de fósforo: el mapa de la semilla elegida en verde monocromo, con
/// la semilla y los bots rotulados como en una pantalla de 1979.
fn draw_monitor(batch: &mut UiBatch, state: &LauncherState, rect: UiRect) {
    let [x, y, w, h] = rect.rect;
    batch.rect(x, y, w, h, STEEL_DARK);
    let (ix, iy, iw, ih) = (x + BEZEL, y + BEZEL, w - 2.0 * BEZEL, h - 2.0 * BEZEL);
    batch.rect(ix, iy, iw, ih, CRT_BG);

    let pad = 14.0;
    let line = UiBatch::text_height(TEXT_BODY);
    let head = format!("Semilla {}", state.current_seed());
    batch.text(ix + pad, iy + pad, TEXT_BODY, PHOSPHOR_BRIGHT, &head);
    let bots = match state.bots {
        _ if state.start_mode.is_story() => "Expedición".to_owned(),
        1 => "1 bot".to_owned(),
        n => format!("{n} bots"),
    };
    let bots_w = UiBatch::text_width(TEXT_BODY, &bots);
    batch.text(
        ix + iw - pad - bots_w,
        iy + pad,
        TEXT_BODY,
        PHOSPHOR_BRIGHT,
        &bots,
    );
    let foot = format!("{} · {}", state.start_mode.label(), state.current_miner());
    let foot_y = iy + ih - pad - line;
    batch.text(ix + pad, foot_y, TEXT_BODY, PHOSPHOR_MID, &foot);

    let map_top = iy + pad + line + 12.0;
    let map_bottom = foot_y - 12.0;
    let (mx, mw, mh) = (ix + pad, iw - 2.0 * pad, (map_bottom - map_top).max(1.0));
    let Some(preview) = state
        .preview
        .as_ref()
        .filter(|p| p.seed == state.current_seed())
    else {
        let msg = "Generando el mapa…";
        let msg_y = map_top + (mh - line) * 0.5;
        batch.text_centered(ix + iw * 0.5, msg_y, TEXT_BODY, PHOSPHOR_DIM, msg);
        return;
    };
    // Celdas cuadradas centradas en el hueco: el mapa no se deforma.
    let cell = (mw / preview.cols as f32).min(mh / preview.rows as f32);
    let ox = mx + (mw - cell * preview.cols as f32) * 0.5;
    let oy = map_top + (mh - cell * preview.rows as f32) * 0.5;
    // El terreno tiene poco contraste: se estira su brillo entre el más
    // oscuro y el más claro para que el relieve se lea en el monitor.
    let cells = (0..preview.rows).flat_map(|row| (0..preview.cols).map(move |col| (col, row)));
    let (lo, hi) = cells.fold((f32::MAX, f32::MIN), |(lo, hi), (col, row)| {
        let l = luminance(preview.color(col, row)).sqrt();
        (lo.min(l), hi.max(l))
    });
    let span = (hi - lo).max(1e-4);
    for row in 0..preview.rows {
        for col in 0..preview.cols {
            let t = (luminance(preview.color(col, row)).sqrt() - lo) / span;
            let glow = phosphor(t);
            let cx = (ox + col as f32 * cell).round();
            let cy = (oy + row as f32 * cell).round();
            let next_x = (ox + (col + 1) as f32 * cell).round();
            let next_y = (oy + (row + 1) as f32 * cell).round();
            batch.rect(cx, cy, next_x - cx, next_y - cy, glow);
        }
    }
}

/// Verde del fósforo para un brillo relativo `t` en `[0, 1]`. Lo más oscuro
/// no llega al fondo de la pantalla para que el mapa se distinga del marco.
fn phosphor(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        let floor = lerp_color(CRT_BG, PHOSPHOR_DIM, 0.5);
        lerp_color(floor, PHOSPHOR_MID, t * 2.0)
    } else {
        lerp_color(PHOSPHOR_MID, PHOSPHOR_BRIGHT, t * 2.0 - 1.0)
    }
}

/// Selector de versión (elemento 0): nombre entre flechas y su estado.
fn draw_selector(batch: &mut UiBatch, state: &LauncherState, rect: UiRect) {
    let ver = state.current_version();
    let focused = state.selected() == 0;
    let hot = hovered(state, &rect);
    let [x, _, w, _] = rect.rect;
    let y = line_top(&rect, TEXT_LARGE);
    if focused {
        tally(batch, x, y + UiBatch::text_height(TEXT_LARGE) * 0.5);
    }
    let colour = if focused || hot {
        ENAMEL_CREAM
    } else {
        PAPER_AGED
    };
    batch.text(
        x + 24.0,
        y,
        TEXT_LARGE,
        colour,
        &format!("‹ {} ›", ver.label),
    );
    let tag = ver.status_tag();
    let tag_colour = if ver.is_installed() {
        PHOSPHOR_MID
    } else {
        LAVA_ORANGE
    };
    let tag_w = UiBatch::text_width(TEXT_BODY, tag);
    let tag_y = line_top(&rect, TEXT_BODY);
    batch.text(x + w - tag_w, tag_y, TEXT_BODY, tag_colour, tag);
}

/// El único botón con acento: la lámpara de sodio.
fn draw_play_button(batch: &mut UiBatch, state: &LauncherState, rect: UiRect) {
    let [x, y, w, h] = rect.rect;
    let focused = state.selected() == 1;
    let fill = if hovered(state, &rect) {
        lerp_color(SODIUM_WARM, ENAMEL_CREAM, 0.2)
    } else {
        SODIUM_WARM
    };
    if focused {
        // Foco del teclado: un filo de latón separado del botón.
        batch.rect(x - 4.0, y - 4.0, w + 8.0, h + 8.0, BRASS_BRIGHT);
        batch.rect(x - 2.0, y - 2.0, w + 4.0, h + 4.0, SOOT_DARK);
    }
    batch.rect(x, y, w, h, fill);
    let label_y = line_top(&rect, TEXT_LARGE);
    batch.text_centered(
        x + w * 0.5,
        label_y,
        TEXT_LARGE,
        INK_STAMP,
        if state.start_mode.is_story() {
            "Empezar la expedición"
        } else {
            "Bajar al turno"
        },
    );
}

/// Enlace de texto subrayado al pasar el ratón o con el foco.
fn draw_link(
    batch: &mut UiBatch,
    state: &LauncherState,
    rect: UiRect,
    item: usize,
    label: &str,
    align_right: bool,
) {
    let active = state.selected() == item || hovered(state, &rect);
    let colour = if active { ENAMEL_CREAM } else { PAPER_AGED };
    let text_w = UiBatch::text_width(TEXT_BODY, label);
    let [x, _, w, _] = rect.rect;
    let tx = if align_right { x + w - text_w } else { x };
    let ty = line_top(&rect, TEXT_BODY);
    batch.text(tx, ty, TEXT_BODY, colour, label);
    if active {
        let under = ty + UiBatch::text_height(TEXT_BODY) + 4.0;
        batch.rect(tx, under, text_w, 2.0, BRASS_BRIGHT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phosphor_ramps_from_dim_to_bright_and_stays_green() {
        let dark = phosphor(0.0);
        assert!(
            contrast(dark, CRT_BG) > 1.1,
            "el mapa se distingue del fondo"
        );
        assert_eq!(phosphor(1.0), PHOSPHOR_BRIGHT);
        assert_eq!(phosphor(0.5), PHOSPHOR_MID);
        assert_eq!(phosphor(7.0), PHOSPHOR_BRIGHT);
        let mut last = 0.0;
        for t in [0.0, 0.1, 0.4, 0.6, 0.9, 1.0] {
            let c = phosphor(t);
            assert!(c[0] < c[1], "{t}: {c:?}");
            assert!(luminance(c) >= last, "sube con t");
            last = luminance(c);
        }
    }
}
