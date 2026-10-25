//! HUD del modo Historia, con la paleta y las reglas de la caseta del Vigía
//! (ver `hush_art::ui_palette`): sin cajas por elemento, texto en frases
//! normales y un solo acento.
//!
//! En pantalla, de forma permanente, solo lo que hace falta para seguir con
//! vida: salud, comida (y aire bajo el agua), la barra rápida y el hito
//! siguiente. El resto (fabricar, pausa, telemetría) son capas que se abren
//! a propósito ([`crate::story_overlays`]).

use grisu_render::{UiBatch, fit_text};
use hush_art::ui_palette::*;
use hush_sim::story::Milestone;
use hush_sim::story::vitals::{MAX_AIR, MAX_FOOD, MAX_HEALTH};

use crate::hud::shadowed;
use crate::story_overlays;
use crate::story_view::HudView;

/// Texto corriente (14 px).
const TEXT_BODY: f32 = 2.0;
/// Avisos grandes (hitos, asedio).
const TEXT_BANNER: f32 = 4.0;
/// Margen exterior.
const EDGE: f32 = 20.0;
/// Casillas de la barra rápida.
const SLOTS: usize = 9;

pub fn draw(batch: &mut UiBatch, size: [u32; 2], view: &HudView) {
    let [w, h] = [size[0].max(1) as f32, size[1].max(1) as f32];
    if !view.craft_open && !view.paused {
        let strip_top = draw_hotbar(batch, w, h, view);
        draw_vitals(batch, w, strip_top, view);
        draw_mining(batch, w, h, view);
        draw_goal(batch, w, view);
        draw_place(batch, view);
        draw_banner(batch, w, h, view);
    }
    if view.craft_open {
        story_overlays::draw_craft(batch, w, h, view);
    }
    if view.paused {
        story_overlays::draw_pause(batch, w, h, view);
    }
    if view.f3_open {
        story_overlays::draw_f3(batch, w, view);
    }
    if view.dead {
        story_overlays::draw_death(batch, w, h);
    }
}

/// Geometría de la barra rápida: `(x de la primera casilla, ancho de casilla,
/// hueco, y de la franja, alto de la franja)`.
fn hotbar_layout(w: f32, h: f32) -> (f32, f32, f32, f32, f32) {
    // Hueco amplio: la cantidad de una casilla no se pega al número de la
    // siguiente.
    let gap = 14.0;
    let slot_w = ((w - 2.0 * EDGE - gap * (SLOTS - 1) as f32) / SLOTS as f32).clamp(56.0, 100.0);
    let total = slot_w * SLOTS as f32 + gap * (SLOTS - 1) as f32;
    let strip_h = 2.0 * UiBatch::text_height(TEXT_BODY) + 26.0;
    let x = ((w - total) * 0.5).round();
    let y = (h - strip_h - 14.0).round();
    (x, slot_w, gap, y, strip_h)
}

/// Barra rápida: una franja con las nueve casillas (número, nombre y
/// cantidad) y, encima, el nombre completo del objeto en la mano. Devuelve
/// la parte de arriba de lo dibujado, para apilar encima las constantes.
fn draw_hotbar(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) -> f32 {
    let (x0, slot_w, gap, y, strip_h) = hotbar_layout(w, h);
    let total = slot_w * SLOTS as f32 + gap * (SLOTS - 1) as f32;
    let pad = 8.0;
    let line = UiBatch::text_height(TEXT_BODY);
    batch.rect(
        x0 - pad,
        y,
        total + 2.0 * pad,
        strip_h,
        with_alpha(SOOT_DARK, 0.84),
    );
    for i in 0..SLOTS {
        let x = x0 + i as f32 * (slot_w + gap);
        let selected = i == view.selected;
        let slot = view.hotbar.get(i).and_then(Option::as_ref);
        let number_colour = if selected { BRASS_BRIGHT } else { PAPER_AGED };
        let number = (i + 1).to_string();
        batch.text(x, y + 8.0, TEXT_BODY, number_colour, &number);
        if let Some(slot) = slot {
            // La cantidad, pegada a su número y no al de la casilla siguiente.
            if slot.count > 1 {
                let count = format!("×{}", slot.count);
                let cx = x + UiBatch::text_width(TEXT_BODY, &number) + 10.0;
                batch.text(cx, y + 8.0, TEXT_BODY, PAPER_AGED, &count);
            }
            let colour = if selected { ENAMEL_CREAM } else { PAPER_AGED };
            let name = fit_text(slot.name, TEXT_BODY, slot_w);
            batch.text(x, y + 8.0 + line + 8.0, TEXT_BODY, colour, &name);
            if slot.max_durability > 0 {
                // Verde mientras queda más de un tercio; después, naranja. Dos
                // colores y no un degradado: a medias, la mezcla se confundía
                // con el latón de la casilla elegida.
                let left =
                    (f32::from(slot.durability) / f32::from(slot.max_durability)).clamp(0.0, 1.0);
                let colour = if left > 1.0 / 3.0 {
                    PHOSPHOR_MID
                } else {
                    LAVA_ORANGE
                };
                batch.rect(x, y + strip_h - 9.0, slot_w, 2.0, STEEL_DARK);
                batch.rect(x, y + strip_h - 9.0, slot_w * left, 2.0, colour);
            }
        }
        if selected {
            batch.rect(x, y + strip_h - 3.0, slot_w, 2.0, BRASS_BRIGHT);
        }
    }
    let held = view
        .hotbar
        .get(view.selected)
        .and_then(Option::as_ref)
        .map_or("Mano vacía", |slot| slot.name);
    let held_w = UiBatch::text_width(TEXT_BODY, held);
    let top = y - line - 12.0;
    shadowed(batch, ((w - held_w) * 0.5).round(), top, ENAMEL_CREAM, held);
    top
}

/// Salud y comida en segmentos (dos puntos por segmento, como en la
/// simulación) sobre la barra, a izquierda y derecha; el aire, solo bajo el
/// agua.
fn draw_vitals(batch: &mut UiBatch, w: f32, above: f32, view: &HudView) {
    let (x0, slot_w, gap, _, _) = hotbar_layout(w, 0.0);
    let total = slot_w * SLOTS as f32 + gap * (SLOTS - 1) as f32;
    let line = UiBatch::text_height(TEXT_BODY);
    let y = above - line - 14.0;
    let v = &view.vitals;
    gauge(
        batch,
        x0,
        y,
        "Salud",
        v.health,
        MAX_HEALTH,
        LAVA_ORANGE,
        false,
    );
    let food_w = gauge_width("Comida");
    gauge(
        batch,
        x0 + total - food_w,
        y,
        "Comida",
        v.food,
        MAX_FOOD,
        BRASS_MID,
        false,
    );
    if v.air < MAX_AIR {
        let air = (u32::from(v.air) * u32::from(MAX_FOOD) / u32::from(MAX_AIR)) as u8;
        let y = y - line - 10.0;
        gauge(
            batch,
            x0 + total - food_w,
            y,
            "Aire",
            air,
            MAX_FOOD,
            PHOSPHOR_MID,
            true,
        );
    }
}

/// Ancho de un indicador con su etiqueta.
fn gauge_width(label: &str) -> f32 {
    UiBatch::text_width(TEXT_BODY, label) + 10.0 + 10.0 * 16.0
}

/// Etiqueta y diez segmentos; un valor impar deja medio segmento.
#[allow(clippy::too_many_arguments)]
fn gauge(
    batch: &mut UiBatch,
    x: f32,
    y: f32,
    label: &str,
    value: u8,
    max: u8,
    fill: Color,
    urgent_label: bool,
) {
    let line = UiBatch::text_height(TEXT_BODY);
    let label_colour = if urgent_label || value * 4 <= max {
        LAVA_ORANGE
    } else {
        PAPER_AGED
    };
    shadowed(batch, x, y, label_colour, label);
    let mut sx = x + UiBatch::text_width(TEXT_BODY, label) + 10.0;
    let seg_y = y + (line - 8.0) * 0.5;
    for i in 0..10u8 {
        let points = value.saturating_sub(i * 2).min(2);
        batch.rect(sx, seg_y, 13.0, 8.0, with_alpha(VOID_BLACK, 0.7));
        if points > 0 {
            let width = if points == 2 { 13.0 } else { 6.0 };
            batch.rect(sx, seg_y, width, 8.0, fill);
        }
        sx += 16.0;
    }
}

/// Avance del picado, justo bajo la cruz: donde se está mirando.
fn draw_mining(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    if view.mining.cell.is_none() || view.mining.total == 0 {
        return;
    }
    let done = view.mining.progress.min(view.mining.total) as f32 / view.mining.total as f32;
    let (bw, x, y) = (64.0, (w * 0.5 - 32.0).round(), (h * 0.5 + 18.0).round());
    batch.rect(x, y, bw, 3.0, with_alpha(VOID_BLACK, 0.7));
    batch.rect(x, y, bw * done, 3.0, BRASS_BRIGHT);
}

/// Hito siguiente, arriba a la derecha.
fn draw_goal(batch: &mut UiBatch, w: f32, view: &HudView) {
    let reached = view.milestones.reached;
    let (head, goal) = match Milestone::ALL.get(usize::from(reached)) {
        Some(m) => (
            format!("Hito {} de 9 · {}", reached + 1, m.label()),
            m.goal(),
        ),
        None => (
            "Expedición completa".to_owned(),
            "Has vuelto a la superficie",
        ),
    };
    let line = UiBatch::text_height(TEXT_BODY);
    let max_w = (w * 0.45).max(200.0);
    let head = fit_text(&head, TEXT_BODY, max_w);
    let goal = fit_text(goal, TEXT_BODY, max_w);
    let right = |text: &str| w - EDGE - UiBatch::text_width(TEXT_BODY, text);
    shadowed(batch, right(&head), EDGE, PAPER_AGED, &head);
    shadowed(batch, right(&goal), EDGE + line + 10.0, ENAMEL_CREAM, &goal);
}

/// Hora y estrato, arriba a la izquierda; el asedio, si lo hay, debajo.
fn draw_place(batch: &mut UiBatch, view: &HudView) {
    let line = UiBatch::text_height(TEXT_BODY);
    let place = format!("{} · {}", view.clock.phase().label(), view.stratum.label());
    shadowed(batch, EDGE, EDGE, PAPER_AGED, &place);
    shadowed(batch, EDGE, EDGE + line + 10.0, PAPER_AGED, "Esc  pausa");
    if view.milestones.siege_started {
        let secs = view.milestones.siege_ticks_left / hush_sim::STORY_TICK_RATE;
        let siege = format!("Asedio · {}:{:02}", secs / 60, secs % 60);
        let y = EDGE + 2.0 * (line + 10.0) + 6.0;
        batch.text(
            EDGE + 2.0,
            y + 2.0,
            3.0,
            with_alpha(VOID_BLACK, 0.85),
            &siege,
        );
        batch.text(EDGE, y, 3.0, LAVA_ORANGE, &siege);
    }
}

/// Aviso grande y centrado; se desvanece en su último segundo.
fn draw_banner(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    let Some((text, ttl)) = view.banner else {
        return;
    };
    let alpha = ttl.clamp(0.0, 1.0);
    let y = (h * 0.28).round();
    let x = ((w - UiBatch::text_width(TEXT_BANNER, text)) * 0.5).round();
    batch.text(
        x + 3.0,
        y + 3.0,
        TEXT_BANNER,
        with_alpha(VOID_BLACK, 0.8 * alpha),
        text,
    );
    batch.text(x, y, TEXT_BANNER, with_alpha(ENAMEL_CREAM, alpha), text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotbar_fits_small_and_wide_windows() {
        for w in [800.0, 1024.0, 1920.0] {
            let (x, slot_w, gap, y, strip_h) = hotbar_layout(w, 640.0);
            let total = slot_w * SLOTS as f32 + gap * (SLOTS - 1) as f32;
            assert!(x >= EDGE - 1.0, "{w}");
            assert!(x + total <= w - EDGE + 1.0, "{w}");
            assert!(y + strip_h <= 640.0);
        }
    }

    #[test]
    fn both_gauges_fit_beside_each_other() {
        let (_, slot_w, gap, _, _) = hotbar_layout(800.0, 0.0);
        let total = slot_w * SLOTS as f32 + gap * (SLOTS - 1) as f32;
        assert!(gauge_width("Salud") + gauge_width("Comida") + 20.0 <= total);
    }
}
