//! Dibujo del lanzador: la pared de la caseta (rótulo, menú, estado y
//! salir) y la página activa. Toda la geometría sale de `layout` para que el
//! dibujo coincida con el ratón; los colores y tamaños, de `theme`.

use crate::layout::{HEADING_BLOCK, HitSlot, Layout, layout, line_top};
use crate::page_play;
use crate::state::{LauncherState, PAGES, Page};
use crate::theme::*;
use grisu_render::{UiBatch, UiRect};
use grisu_render::{fit_text, wrap_text};

/// Lema de la tienda, en la traducción del glosario.
const TAGLINE: &str = "Roba a una montaña dormida.";

/// Dibuja el fotograma completo del lanzador.
pub fn draw(batch: &mut UiBatch, state: &LauncherState, width: f32, height: f32) {
    let lay = layout(state, width, height);
    batch.rect(0.0, 0.0, width, height, NIGHT_VEIL);
    draw_wall(batch, state, &lay);
    match state.page() {
        Page::Play => page_play::draw_play(batch, state, &lay),
        Page::Instalaciones => draw_versions(batch, state, &lay),
        Page::Opciones => draw_options(batch, state, &lay),
        Page::Acerca => draw_about(batch, &lay),
    }
    // Fundido corto mientras entra la página recién elegida.
    let cover = fade_cover(state.page_fade);
    if cover[3] > 0.0 {
        let x = lay.wall.rect[2];
        batch.rect(x, 0.0, width - x, height, cover);
    }
}

pub(crate) fn hovered(state: &LauncherState, r: &UiRect) -> bool {
    let (mx, my) = state.mouse;
    r.contains(mx, my)
}

pub(crate) use hush_art::ui_palette::draw_tally as tally;

fn draw_wall(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    let [_, _, wall_w, wall_h] = lay.wall.rect;
    batch.rect(0.0, 0.0, wall_w, wall_h, WALL);
    batch.rect(wall_w - 1.0, 0.0, 1.0, wall_h, STEEL_DARK);

    let (tx, ty) = lay.title;
    batch.text(tx, ty, TEXT_TITLE, ENAMEL_CREAM, "HUSHMINE");
    let tag_y = ty + UiBatch::text_height(TEXT_TITLE) + 18.0;
    batch.text(tx, tag_y, TEXT_BODY, PAPER_AGED, TAGLINE);

    for (idx, hit) in lay.sidebar.iter().enumerate() {
        let rect = hit.rect;
        let hot = hovered(state, &rect);
        let (label, scale) = match hit.slot {
            HitSlot::SidebarPage(page) => (page.label(), TEXT_LARGE),
            HitSlot::Exit => ("Salir", TEXT_BODY),
        };
        let active = hit.slot != HitSlot::Exit && PAGES.get(idx) == Some(&state.page());
        let colour = if active {
            ENAMEL_CREAM
        } else if hot {
            PAPER_AGED
        } else {
            STEEL_LIGHT
        };
        // Salir no lleva ficha, pero se alinea con el texto de las entradas.
        let x = rect.rect[0] + 16.0 + 24.0;
        let y = line_top(&rect, scale);
        if active {
            tally(
                batch,
                rect.rect[0] + 16.0,
                y + UiBatch::text_height(scale) * 0.5,
            );
        }
        // El gris acero no llega a 4,5:1 en texto pequeño: Salir usa papel.
        let colour = if hit.slot == HitSlot::Exit && !hot {
            PAPER_AGED
        } else {
            colour
        };
        batch.text(x, y, scale, colour, label);
    }

    let (sx, sy) = lay.status;
    let colour = if state.status_ok {
        PHOSPHOR_MID
    } else {
        LAVA_ORANGE
    };
    // Alineado con el texto de las entradas y de Salir.
    batch.text(sx + 24.0, sy, TEXT_BODY, colour, &state.status);
}

/// Encabezado de las páginas con título: nombre y una línea de contexto.
fn draw_heading(batch: &mut UiBatch, lay: &Layout, title: &str, line: &str) {
    let [x, y, w, _] = lay.content.rect;
    batch.text(x, y, TEXT_HEADING, ENAMEL_CREAM, title);
    let line_y = y + UiBatch::text_height(TEXT_HEADING) + 14.0;
    batch.text(x, line_y, TEXT_BODY, PAPER_AGED, line);
    batch.rect(x, y + HEADING_BLOCK - 12.0, w, 1.0, STEEL_DARK);
}

/// Separador fino al pie de una fila.
fn row_rule(batch: &mut UiBatch, rect: &UiRect) {
    let [x, y, w, h] = rect.rect;
    batch.rect(x, y + h - 1.0, w, 1.0, STEEL_DARK);
}

fn draw_versions(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    draw_heading(batch, lay, "Versiones", Page::Instalaciones.subtitle());
    for slot in &lay.items {
        let rect = slot.rect;
        let Some(ver) = state.versions.get(slot.item) else {
            continue;
        };
        let selected = state.selected() == slot.item;
        let hot = hovered(state, &rect);
        let [x, y, w, _] = rect.rect;
        let name_y = y + 12.0;
        if selected {
            tally(batch, x, name_y + UiBatch::text_height(TEXT_LARGE) * 0.5);
        }
        let name_colour = if selected || hot {
            ENAMEL_CREAM
        } else {
            PAPER_AGED
        };
        batch.text(x + 24.0, name_y, TEXT_LARGE, name_colour, &ver.label);
        let tag = ver.status_tag();
        let tag_colour = if ver.is_installed() {
            PHOSPHOR_MID
        } else {
            LAVA_ORANGE
        };
        let tag_w = UiBatch::text_width(TEXT_BODY, tag);
        batch.text(x + w - tag_w, name_y + 4.0, TEXT_BODY, tag_colour, tag);
        let sub_y = name_y + UiBatch::text_height(TEXT_LARGE) + 12.0;
        let age = state.last_played_text(slot.item);
        let age_w = UiBatch::text_width(TEXT_BODY, &age);
        batch.text(x + w - age_w, sub_y, TEXT_BODY, PAPER_AGED, &age);
        let sub = fit_text(&ver.subtitle, TEXT_BODY, w - 24.0 - age_w - 24.0);
        batch.text(x + 24.0, sub_y, TEXT_BODY, PAPER_AGED, &sub);
        row_rule(batch, &rect);
    }
}

fn draw_options(batch: &mut UiBatch, state: &LauncherState, lay: &Layout) {
    draw_heading(batch, lay, "Opciones", Page::Opciones.subtitle());
    let rows = state.option_rows();
    for (idx, slot) in lay.items.iter().enumerate() {
        let rect = slot.rect;
        let (label, value) = &rows[idx];
        let selected = state.selected() == idx;
        let hot = hovered(state, &rect);
        let [x, _, w, _] = rect.rect;
        let y = line_top(&rect, TEXT_BODY);
        if selected {
            tally(batch, x, y + UiBatch::text_height(TEXT_BODY) * 0.5);
        }
        let label_colour = if selected { ENAMEL_CREAM } else { PAPER_AGED };
        batch.text(x + 24.0, y, TEXT_BODY, label_colour, label);
        // El valor lleva flechas cuando se puede cambiar: en la fila elegida
        // o bajo el ratón (los tercios de la fila bajan y suben).
        let shown = if selected || hot {
            format!("‹ {value} ›")
        } else {
            value.clone()
        };
        let vw = UiBatch::text_width(TEXT_BODY, &shown);
        batch.text(x + w - vw, y, TEXT_BODY, ENAMEL_CREAM, &shown);
        row_rule(batch, &rect);
    }
}

fn draw_about(batch: &mut UiBatch, lay: &Layout) {
    draw_heading(batch, lay, "HUSHMINE", Page::Acerca.subtitle());
    let [x, y, w, _] = lay.content.rect;
    let mut y = y + HEADING_BLOCK + 4.0;
    let intro = "Minería competitiva en vóxeles: dos cuadrillas roban oro a una \
                 montaña que duerme. Cada explosión la despierta un poco más.";
    for line in wrap_text(intro, TEXT_BODY, w) {
        batch.text(x, y, TEXT_BODY, ENAMEL_CREAM, &line);
        y += UiBatch::text_height(TEXT_BODY) + 12.0;
    }
    y += 22.0;
    let controls = [
        ("W A S D", "andar"),
        ("Espacio", "saltar"),
        ("Mayús", "correr"),
        ("Clic izquierdo", "cavar"),
        ("Clic derecho", "poner un bloque"),
        ("1 a 6", "elegir bloque"),
        ("V", "cambiar de cámara"),
        ("Escape", "menú del juego"),
    ];
    let key_w = w * 0.42;
    for (key, action) in controls {
        batch.text(x, y, TEXT_BODY, ENAMEL_CREAM, key);
        batch.text(x + key_w, y, TEXT_BODY, PAPER_AGED, action);
        y += UiBatch::text_height(TEXT_BODY) + 12.0;
    }
    y += 22.0;
    batch.text(
        x,
        y,
        TEXT_BODY,
        PAPER_AGED,
        "Partidas por LAN en el puerto UDP 47400.",
    );
}

/// Atajos de teclado de la página, al pie del contenido.
pub(crate) fn draw_hints(batch: &mut UiBatch, lay: &Layout, hints: &[(&str, &str)]) {
    let [x, y, _, h] = lay.content.rect;
    let mut pen = x;
    let base = y + h - UiBatch::text_height(TEXT_BODY);
    for (key, action) in hints {
        batch.text(pen, base, TEXT_BODY, ENAMEL_CREAM, key);
        pen += UiBatch::text_width(TEXT_BODY, key) + 8.0;
        batch.text(pen, base, TEXT_BODY, PAPER_AGED, action);
        pen += UiBatch::text_width(TEXT_BODY, action) + 22.0;
    }
}
