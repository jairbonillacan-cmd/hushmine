//! Página JUGAR: tarjeta de versión, vista previa del mapa y la pila de
//! acciones al pie (JUGAR, COMPILAR, SERVIDOR DEDICADO).

use crate::draw::hovered;
use crate::layout::Layout;
use crate::state::LauncherState;
use crate::theme::*;
use grisu_render::{UiBatch, UiRect};

pub fn draw_play(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    draw_version_card(batch, state, lay.items[0].rect);

    // Franja de resumen con los ajustes de la partida.
    let sum = lay.summary;
    batch.panel(
        sum.rect[0],
        sum.rect[1],
        sum.rect[2],
        sum.rect[3],
        FRANJA,
        [0.18, 0.21, 0.26, 1.0],
        1.0,
    );
    batch.text_centered(
        sum.rect[0] + sum.rect[2] * 0.5,
        sum.rect[1] + 20.0,
        1.5,
        TEXTO,
        &state.summary_line(),
    );

    // Vista previa del mapa centrada en el hueco libre de la página.
    let cw = lay.content.rect[2];
    let preview_w = 320.0f32.min(cw * 0.5);
    let top = sum.rect[1] + sum.rect[3] + 16.0;
    let bottom = lay.items[3].rect.rect[1] - 14.0;
    let preview_h = (bottom - top).clamp(80.0, 210.0);
    draw_map_preview(
        batch,
        state,
        lay.content.rect[0] + (cw - preview_w) * 0.5,
        top,
        preview_w,
        preview_h,
    );

    draw_play_button(batch, state, lay.items[1].rect);
    draw_action_button(
        batch,
        state,
        lay.items[2].rect,
        2,
        AZUL_RELLENO,
        [0.26, 0.36, 0.48, 1.0],
        AZUL_BORDE,
        [0.14, 0.20, 0.29, 1.0],
        "COMPILAR ESTA VERSION",
    );
    draw_action_button(
        batch,
        state,
        lay.items[3].rect,
        3,
        [0.20, 0.13, 0.06, 0.97],
        [0.52, 0.40, 0.20, 1.0],
        LATON,
        [0.30, 0.21, 0.10, 1.0],
        "SERVIDOR DEDICADO - LAN 47400",
    );
}

fn draw_play_button(batch: &mut UiBatch, state: &LauncherState, play: UiRect) {
    let hot = hovered(state, &play) || state.selected() == 1;
    let wave = pulse(state.animation);
    let (bg, border) = if hot {
        ([0.19, 0.50, 0.26, 1.0], VERDE)
    } else {
        (
            VERDE_RELLENO,
            lerp_color([0.30, 0.68, 0.38, 1.0], VERDE, wave),
        )
    };
    let sheen = lerp_color([0.0, 0.0, 0.0, 0.0], [0.85, 1.0, 0.90, 0.35], wave);
    batch.rect(
        play.rect[0] + 4.0,
        play.rect[1] + 4.0,
        (play.rect[2] - 8.0).max(0.0),
        4.0,
        sheen,
    );
    batch.panel(
        play.rect[0],
        play.rect[1],
        play.rect[2],
        play.rect[3],
        bg,
        border,
        3.0,
    );
    batch.text_centered(
        play.rect[0] + play.rect[2] * 0.5,
        play.rect[1] + (play.rect[3] - UiBatch::text_height(3.4)) * 0.5,
        3.4,
        [0.90, 1.0, 0.92, 1.0],
        "JUGAR",
    );
}

/// Botón secundario reutilizable (COMPILAR y SERVIDOR).
#[allow(clippy::too_many_arguments)]
fn draw_action_button(
    batch: &mut UiBatch,
    state: &LauncherState,
    rect: UiRect,
    item: usize,
    fill: Color,
    border: Color,
    border_hot: Color,
    fill_hot: Color,
    label: &str,
) {
    let hot = hovered(state, &rect) || state.selected() == item;
    batch.panel(
        rect.rect[0],
        rect.rect[1],
        rect.rect[2],
        rect.rect[3],
        if hot { fill_hot } else { fill },
        if hot { border_hot } else { border },
        2.0,
    );
    batch.text_centered(
        rect.rect[0] + rect.rect[2] * 0.5,
        rect.rect[1] + (rect.rect[3] - UiBatch::text_height(1.8)) * 0.5,
        1.8,
        [0.90, 0.90, 0.86, 1.0],
        label,
    );
}

/// Vista previa grande del mapa: mosaico de la superficie con su título.
fn draw_map_preview(batch: &mut UiBatch, state: &LauncherState, x: f32, y: f32, w: f32, h: f32) {
    batch.panel(
        x,
        y,
        w,
        h,
        [0.02, 0.03, 0.05, 0.88],
        [0.34, 0.38, 0.44, 1.0],
        2.0,
    );
    let title = format!("MAPA - SEMILLA {}", state.current_seed());
    batch.text_centered(x + w * 0.5, y + 8.0, 1.3, TEXTO_SUAVE, &title);
    let pad = 8.0;
    let top = y + 8.0 + UiBatch::text_height(1.3) + 6.0;
    let (iw, ih) = ((w - 2.0 * pad).max(1.0), (y + h - top - pad).max(1.0));
    let Some(preview) = state
        .preview
        .as_ref()
        .filter(|p| p.seed == state.current_seed())
    else {
        batch.text_centered(
            x + w * 0.5,
            y + h * 0.5 - 4.0,
            1.3,
            TEXTO_SUAVE,
            "SIN VISTA PREVIA",
        );
        return;
    };
    let cell_w = iw / preview.cols as f32;
    let cell_h = ih / preview.rows as f32;
    for row in 0..preview.rows {
        for col in 0..preview.cols {
            let color = preview.color(col, row);
            batch.rect(
                (x + pad + col as f32 * cell_w).round(),
                (top + row as f32 * cell_h).round(),
                (cell_w + 0.5).ceil(),
                (cell_h + 0.5).ceil(),
                color,
            );
        }
    }
}

/// Tarjeta de versión (elemento 0): selector `< >`, estado y subtítulo.
pub fn draw_version_card(batch: &mut UiBatch, state: &LauncherState, card: UiRect) {
    let ver = state.current_version();
    let est = estilo_control(state.selected() == 0, false);
    batch.panel(
        card.rect[0],
        card.rect[1],
        card.rect[2],
        card.rect[3],
        est.fondo,
        est.borde,
        2.0,
    );
    batch.text(
        card.rect[0] + 10.0,
        card.rect[1] + 7.0,
        1.3,
        TEXTO_SUAVE,
        "VERSION DE JUEGO",
    );
    let tag = ver.status_tag();
    let tag_w = UiBatch::text_width(1.3, tag);
    batch.text(
        card.rect[0] + card.rect[2] - tag_w - 10.0,
        card.rect[1] + 7.0,
        1.3,
        if ver.is_installed() { VERDE_OK } else { ROJO },
        tag,
    );
    let sel = format!("< {} >", ver.label);
    batch.text_centered(
        card.rect[0] + card.rect[2] * 0.5,
        card.rect[1] + 22.0,
        2.2,
        TEXTO_CALIDO,
        &sel,
    );
    batch.text_centered(
        card.rect[0] + card.rect[2] * 0.5,
        card.rect[1] + 54.0,
        1.4,
        TEXTO_SUAVE,
        &ver.subtitle,
    );
}
