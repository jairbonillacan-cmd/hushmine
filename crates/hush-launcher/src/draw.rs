//! Dibujo del panel: barra lateral, cabecera, página activa y franja de
//! estado. Toda la geometría sale de `layout` para coincidir con el ratón.

use crate::layout::{HitSlot, Layout, layout};
use crate::page_play;
use crate::state::{LauncherState, PAGES, Page};
use crate::theme::*;
use grisu_render::{UiBatch, UiRect};

/// Dibuja el fotograma completo del launcher.
pub fn draw(batch: &mut UiBatch, state: &LauncherState, width: f32, height: f32) {
    let lay = layout(state, width, height);
    batch.rect(0.0, 0.0, width, height, FONDO);
    draw_sidebar(batch, state, &lay);
    batch.panel(
        lay.content.rect[0],
        lay.content.rect[1],
        lay.content.rect[2],
        lay.content.rect[3],
        PANEL,
        LATERAL_BORDE,
        2.0,
    );
    draw_header(batch, state, &lay);
    match state.page() {
        Page::Play => page_play::draw_play(batch, state, &lay),
        Page::Instalaciones => draw_instalaciones(batch, state, &lay),
        Page::Opciones => draw_opciones(batch, state, &lay),
        Page::Acerca => draw_acerca(batch, state, &lay),
    }
    // Velo de fundido mientras la pagina recien cambiada entra.
    let cover = fade_cover(state.page_fade);
    if cover[3] > 0.0 {
        batch.rect(
            lay.content.rect[0],
            lay.content.rect[1],
            lay.content.rect[2],
            lay.content.rect[3],
            cover,
        );
    }
    draw_status(batch, state, &lay);
}

pub(crate) fn hovered(state: &LauncherState, r: &UiRect) -> bool {
    let (mx, my) = state.mouse;
    r.contains(mx, my)
}

fn draw_sidebar(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    batch.panel(
        lay.sidebar_bg.rect[0],
        lay.sidebar_bg.rect[1],
        lay.sidebar_bg.rect[2],
        lay.sidebar_bg.rect[3],
        LATERAL,
        LATERAL_BORDE,
        2.0,
    );
    for (idx, hit) in lay.sidebar.iter().enumerate() {
        let is_exit = hit.slot == HitSlot::Exit;
        let active = !is_exit && PAGES.get(idx) == Some(&state.page());
        let hot = hovered(state, &hit.rect) || active;
        let (bg, border, text) = if is_exit {
            (
                if hot { [0.16, 0.08, 0.08, 0.98] } else { BOTON },
                if hot { ROJO } else { [0.30, 0.22, 0.22, 1.0] },
                if hot { [1.0, 0.80, 0.78, 1.0] } else { TEXTO },
            )
        } else if active {
            (BOTON_ACTIVO, LATON, TEXTO_CALIDO)
        } else if hot {
            (PANEL_CLARO, [0.38, 0.42, 0.48, 1.0], TEXTO)
        } else {
            (BOTON, [0.26, 0.30, 0.36, 1.0], TEXTO_SUAVE)
        };
        batch.panel(
            hit.rect.rect[0],
            hit.rect.rect[1],
            hit.rect.rect[2],
            hit.rect.rect[3],
            bg,
            border,
            2.0,
        );
        let label = if is_exit {
            "SALIR".to_owned()
        } else {
            PAGES[idx].label().to_owned()
        };
        // Las etiquetas largas bajan de escala para respetar el margen.
        let max_w = hit.rect.rect[2] - 46.0;
        let scale = if UiBatch::text_width(1.9, &label) > max_w {
            1.5
        } else {
            1.9
        };
        let ty = hit.rect.rect[1] + (hit.rect.rect[3] - UiBatch::text_height(scale)) * 0.5;
        if active {
            batch.text(hit.rect.rect[0] + 12.0, ty, scale, VERDE, ">");
        }
        batch.text_centered(
            hit.rect.rect[0] + hit.rect.rect[2] * 0.5 + if active { 7.0 } else { 0.0 },
            ty,
            scale,
            text,
            &label,
        );
    }
}

fn draw_header(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    let [x, y, w, _] = lay.header.rect;
    batch.text(x + 4.0, y + 8.0, 3.0, TEXTO_CALIDO, state.page().label());
    batch.text(x + 4.0, y + 38.0, 1.4, TEXTO_SUAVE, state.page().subtitle());
    let brand_w = UiBatch::text_width(2.0, "HUSHMINE");
    batch.text(x + w - brand_w - 8.0, y + 8.0, 2.0, TEXTO, "HUSHMINE");
    let sub = "LAUNCHER // MOTOR GRISU";
    let sub_w = UiBatch::text_width(1.3, sub);
    batch.text(x + w - sub_w - 8.0, y + 30.0, 1.3, TEXTO_SUAVE, sub);
}

fn draw_instalaciones(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    for slot in &lay.items {
        let rect = slot.rect;
        let Some(ver) = state.versions.get(slot.item) else {
            continue;
        };
        let selected = state.selected() == slot.item;
        let hot = hovered(state, &rect);
        let border = if selected {
            LATON
        } else if hot {
            [0.42, 0.46, 0.52, 1.0]
        } else {
            [0.26, 0.30, 0.36, 1.0]
        };
        batch.panel(
            rect.rect[0],
            rect.rect[1],
            rect.rect[2],
            rect.rect[3],
            if selected { PANEL_CLARO } else { BOTON },
            border,
            2.0,
        );
        if selected {
            batch.text(rect.rect[0] + 10.0, rect.rect[1] + 9.0, 1.8, VERDE, ">");
        }
        batch.text(
            rect.rect[0] + 26.0,
            rect.rect[1] + 9.0,
            1.8,
            if selected { TEXTO_CALIDO } else { TEXTO },
            &ver.label,
        );
        let tag = ver.status_tag();
        let tag_w = UiBatch::text_width(1.4, tag);
        batch.text(
            rect.rect[0] + rect.rect[2] - tag_w - 10.0,
            rect.rect[1] + 10.0,
            1.4,
            if ver.is_installed() { VERDE_OK } else { ROJO },
            tag,
        );
        batch.text(
            rect.rect[0] + 26.0,
            rect.rect[1] + 34.0,
            1.4,
            TEXTO_SUAVE,
            &ver.subtitle,
        );
        let age = state.last_played_text(slot.item);
        let age_w = UiBatch::text_width(1.4, &age);
        batch.text(
            rect.rect[0] + rect.rect[2] - age_w - 10.0,
            rect.rect[1] + 34.0,
            1.4,
            TEXTO_SUAVE,
            &age,
        );
    }
}

fn draw_opciones(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    let rows = state.option_rows();
    for (idx, slot) in lay.items.iter().enumerate() {
        let rect = slot.rect;
        let (label, value) = &rows[idx];
        let active = state.selected() == idx;
        let hot = hovered(state, &rect);
        let est = estilo_control(active, false);
        batch.panel(
            rect.rect[0],
            rect.rect[1],
            rect.rect[2],
            rect.rect[3],
            if hot && !active {
                PANEL_CLARO
            } else {
                est.fondo
            },
            est.borde,
            2.0,
        );
        let scale = 1.8;
        let ty = rect.rect[1] + (rect.rect[3] - UiBatch::text_height(scale)) * 0.5;
        batch.text(
            rect.rect[0] + 14.0,
            ty,
            scale,
            if active { est.texto } else { TEXTO },
            label,
        );
        let vw = UiBatch::text_width(scale, value);
        batch.text(
            rect.rect[0] + rect.rect[2] - vw - 14.0,
            ty,
            scale,
            if active { TEXTO_CALIDO } else { TEXTO },
            value,
        );
    }
}

fn draw_acerca(batch: &mut UiBatch, _state: &LauncherState, lay: &Layout) {
    let cx = lay.content.rect[0] + lay.content.rect[2] * 0.5;
    let mut y = lay.content.rect[1] + 84.0;
    let lines: [(&str, f32, Color); 10] = [
        ("HUSHMINE", 2.6, TEXTO_CALIDO),
        ("JUEGO COMPETITIVO DE MINERIA EN VOXELES", 1.5, TEXTO),
        (
            "MOTOR GRISU: RUST + WGPU, SIMULACION DETERMINISTA",
            1.4,
            TEXTO_SUAVE,
        ),
        ("", 1.0, TEXTO),
        ("WASD ANDAR - ESPACIO SALTAR - CLIC IZQ CAVAR", 1.5, TEXTO),
        (
            "CLIC DER PONER - 1-6 ELEGIR BLOQUE - ESCAPE MENU",
            1.5,
            TEXTO,
        ),
        ("", 1.0, TEXTO),
        (
            "MULTIJUGADOR POR LAN EN EL PUERTO UDP 47400",
            1.4,
            TEXTO_SUAVE,
        ),
        ("SERVIDOR DEDICADO: HUSH-SERVER --BOTS N", 1.4, TEXTO_SUAVE),
        (
            "TUS VERSIONES INSTALADAS ESTAN EN INSTALACIONES",
            1.4,
            TEXTO_SUAVE,
        ),
    ];
    for (text, scale, col) in lines {
        if !text.is_empty() {
            batch.text_centered(cx, y, scale, col, text);
        }
        y += 7.0 * scale + 15.0;
    }
}

fn draw_status(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    let [x, y, w, h] = lay.status.rect;
    batch.rect(x, y + h - 34.0, w, 34.0, FRANJA);
    let hint = match state.page() {
        Page::Play => "[WS] MOVER   [AD] AJUSTAR   [ENTER] JUGAR   [C] COMPILAR   [TAB] PAGINA",
        Page::Instalaciones => "[WS] ELEGIR   [ENTER] JUGAR   [C] COMPILAR   [TAB] PAGINA",
        Page::Opciones => "[WS] ELEGIR   [AD] AJUSTAR   [ENTER] AVANZAR   [TAB] PAGINA",
        Page::Acerca => "[TAB] PAGINA   [ESC] SALIR",
    };
    batch.text(x + 8.0, y + h - 26.0, 1.4, TEXTO_SUAVE, hint);
    let col = if state.status_ok { VERDE_OK } else { ROJO };
    batch.text(x + 8.0, y + h - 13.0, 1.4, col, &state.status);
}
