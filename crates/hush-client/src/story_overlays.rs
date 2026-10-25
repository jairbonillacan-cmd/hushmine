//! Capas del HUD del modo Historia que se abren a propósito: fabricar,
//! pausa, la pantalla de muerte y la telemetría F3. Fabricar y la pausa usan
//! la misma pared de la caseta que el menú del Turno.

use grisu_render::{UiBatch, fit_text};
use grisu_voxel::cell_of;
use hush_art::ui_palette::*;

use crate::story_view::HudView;

const TEXT_BODY: f32 = 2.0;
const TEXT_HEADING: f32 = 4.0;
const GUTTER: f32 = 40.0;
/// Primera fila bajo el título y su línea.
const ROWS_TOP: f32 = GUTTER + TEXT_HEADING * 7.0 + 40.0;

/// Pared de la caseta con su título; devuelve su ancho.
fn wall(batch: &mut UiBatch, w: f32, h: f32, title: &str) -> f32 {
    let wall_w = (w * 0.5).clamp(360.0, 560.0).min(w);
    batch.rect(0.0, 0.0, w, h, NIGHT_VEIL);
    batch.rect(0.0, 0.0, wall_w, h, WALL);
    batch.rect(wall_w - 1.0, 0.0, 1.0, h, STEEL_DARK);
    batch.text(GUTTER, GUTTER, TEXT_HEADING, ENAMEL_CREAM, title);
    batch.rect(
        GUTTER,
        ROWS_TOP - 16.0,
        wall_w - 2.0 * GUTTER,
        1.0,
        STEEL_DARK,
    );
    wall_w
}

/// Atajos al pie de la pared, en pares tecla y acción.
fn hints(batch: &mut UiBatch, h: f32, pairs: &[(&str, &str)]) {
    let y = h - GUTTER - UiBatch::text_height(TEXT_BODY);
    let mut x = GUTTER;
    for (key, action) in pairs {
        batch.text(x, y, TEXT_BODY, ENAMEL_CREAM, key);
        x += UiBatch::text_width(TEXT_BODY, key) + 8.0;
        batch.text(x, y, TEXT_BODY, PAPER_AGED, action);
        x += UiBatch::text_width(TEXT_BODY, action) + 20.0;
    }
}

/// Menú de fabricación: las recetas, con las que no se pueden hacer marcadas
/// a la derecha.
pub fn draw_craft(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    let wall_w = wall(batch, w, h, "Fabricación");
    let line = UiBatch::text_height(TEXT_BODY);
    let row_h = line + 14.0;
    let rows_fit = (((h - ROWS_TOP - GUTTER - 50.0) / row_h) as usize).max(1);
    let first = view.craft_selected.saturating_sub(rows_fit - 1);
    let (x, rw) = (GUTTER - 16.0, wall_w - 2.0 * (GUTTER - 16.0));
    for (idx, row) in view
        .craft_rows
        .iter()
        .enumerate()
        .skip(first)
        .take(rows_fit)
    {
        let y = ROWS_TOP + (idx - first) as f32 * row_h;
        let ty = y + 7.0;
        let selected = idx == view.craft_selected;
        if selected {
            draw_tally(batch, x + 16.0, ty + line * 0.5);
        }
        let missing = "faltan materiales";
        let tag_w = if row.available {
            0.0
        } else {
            UiBatch::text_width(TEXT_BODY, missing) + 20.0
        };
        let colour = if selected || row.available {
            ENAMEL_CREAM
        } else {
            PAPER_AGED
        };
        let label = fit_text(&row.label, TEXT_BODY, rw - 56.0 - tag_w);
        batch.text(x + 40.0, ty, TEXT_BODY, colour, &label);
        if !row.available {
            let mw = UiBatch::text_width(TEXT_BODY, missing);
            batch.text(x + rw - 16.0 - mw, ty, TEXT_BODY, LAVA_ORANGE, missing);
        }
    }
    hints(
        batch,
        h,
        &[
            ("Flechas", "elegir"),
            ("Enter", "fabricar"),
            ("F", "cerrar"),
        ],
    );
}

/// Pausa: las teclas del modo Historia y cómo va la expedición.
pub fn draw_pause(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    let wall_w = wall(batch, w, h, "Pausa");
    let line = UiBatch::text_height(TEXT_BODY);
    let keys = [
        ("Mayús", "correr"),
        ("Ctrl", "agacharse, en silencio"),
        ("Clic izquierdo", "picar (mantener) o golpear"),
        ("Clic derecho", "poner lo que llevas"),
        ("E", "usar banco, horno o arcón"),
        ("F", "fabricar"),
        ("R", "comer"),
        ("1 a 9", "elegir casilla"),
        ("F3", "telemetría"),
        ("F5", "cámara"),
    ];
    let key_x = GUTTER;
    let widest = keys
        .iter()
        .map(|(key, _)| UiBatch::text_width(TEXT_BODY, key))
        .fold(0.0, f32::max);
    let action_x = GUTTER + widest + 24.0;
    for (i, (key, action)) in keys.iter().enumerate() {
        let y = ROWS_TOP + i as f32 * (line + 12.0);
        batch.text(key_x, y, TEXT_BODY, ENAMEL_CREAM, key);
        let room = wall_w - GUTTER - action_x;
        batch.text(
            action_x,
            y,
            TEXT_BODY,
            PAPER_AGED,
            &fit_text(action, TEXT_BODY, room),
        );
    }
    let days = view.clock.tick() / hush_sim::story::DAY_TICKS + 1;
    let progress = format!("Día {days} · {} de 9 hitos", view.milestones.reached);
    let py = ROWS_TOP + keys.len() as f32 * (line + 12.0) + 20.0;
    batch.text(GUTTER, py, TEXT_BODY, PAPER_AGED, &progress);
    hints(batch, h, &[("Esc", "seguir"), ("Q", "salir del juego")]);
}

/// La expedición ha terminado mal.
pub fn draw_death(batch: &mut UiBatch, w: f32, h: f32) {
    batch.rect(0.0, 0.0, w, h, with_alpha(VOID_BLACK, 0.7));
    let title = "Has desfallecido";
    let x = ((w - UiBatch::text_width(TEXT_HEADING, title)) * 0.5).round();
    batch.text(x, (h * 0.42).round(), TEXT_HEADING, LAVA_ORANGE, title);
    let sub = "La expedición termina aquí.  Esc para salir";
    let sx = ((w - UiBatch::text_width(TEXT_BODY, sub)) * 0.5).round();
    let sy = (h * 0.42).round() + UiBatch::text_height(TEXT_HEADING) + 20.0;
    batch.text(sx, sy, TEXT_BODY, PAPER_AGED, sub);
}

/// Telemetría F3: dos columnas de lectura de fósforo, como un monitor.
pub fn draw_f3(batch: &mut UiBatch, w: f32, view: &HudView) {
    let cell = view.feet.map(cell_of);
    let chunk = cell.map(|c| c.div_euclid(16));
    let left = [
        format!("HUSHMINE {} ({})", env!("CARGO_PKG_VERSION"), profile()),
        format!(
            "{:.0} fps · 1 % bajo {:.0} · {:.2} ms",
            view.fps.0, view.fps.1, view.fps.2
        ),
        String::new(),
        format!("Bloque {} {} {}", cell[0], cell[1], cell[2]),
        format!("Chunk {} {} {}", chunk[0], chunk[1], chunk[2]),
        format!(
            "Rumbo {} · {:+.0}° · {:.0}°",
            view.facing, view.pitch, view.yaw
        ),
        view.stratum.label().to_owned(),
        format!(
            "{} · tick {}",
            view.clock.phase().label(),
            view.clock.tick()
        ),
        format!(
            "Luz de cielo {} · de bloque {}",
            view.sky_light, view.block_light
        ),
        String::new(),
        format!(
            "Salud {} · comida {} · aire {}",
            view.vitals.health, view.vitals.food, view.vitals.air
        ),
        format!("Enemigos {} · tick {}", view.mobs, view.tick),
    ];
    let mut right = vec![
        format!(
            "Mallas {} de {} chunks",
            view.mesh_chunks.0, view.mesh_chunks.1
        ),
        format!("{} caras · {:.1} ms", view.mesh_faces, view.mesh_millis),
        format!("Huella {:016X}", view.state_hash),
        String::new(),
    ];
    match view.target {
        Some(hit) => {
            right.push(format!(
                "Apuntas a {} ({} {} {})",
                view.target_name, hit.block[0], hit.block[1], hit.block[2]
            ));
            right.push(format!(
                "Dureza {} · pico de nivel {}",
                view.target_hardness, view.target_tier
            ));
        }
        None => right.push("Nada a la vista".to_owned()),
    }
    let col_w = ((w - 3.0 * 12.0) * 0.5).min(380.0);
    column(batch, 12.0, &left, col_w);
    column(batch, w - 12.0 - col_w, &right, col_w);
}

fn column(batch: &mut UiBatch, x: f32, lines: &[String], col_w: f32) {
    let lh = UiBatch::text_height(TEXT_BODY) + 8.0;
    let y0 = 70.0;
    let h = lines.len() as f32 * lh + 16.0;
    batch.rect(x, y0, col_w, h, with_alpha(CRT_BG, 0.85));
    for (i, line) in lines.iter().enumerate() {
        if !line.is_empty() {
            let text = fit_text(line, TEXT_BODY, col_w - 20.0);
            batch.text(
                x + 10.0,
                y0 + 8.0 + i as f32 * lh,
                TEXT_BODY,
                PHOSPHOR_BRIGHT,
                &text,
            );
        }
    }
}

fn profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}
