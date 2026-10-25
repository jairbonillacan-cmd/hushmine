//! HUD del modo historia: vitalidad (corazones y muslos de comida), barra
//! rápida con las pilas, objetivo de hito, banner de eventos, menú de
//! fabricación y superposición de telemetría F3 (columna izquierda con
//! espacio y entorno, derecha con motor gráfico y bloque inspeccionado).
//!
//! [`HudView`] copia lo que hay que dibujar para que el lote 2D y la app no
//! compitan por los préstamos.

use grisu_render::UiBatch;
use grisu_voxel::{RayHit, cell_of};
use hush_sim::BlockKind;
use hush_sim::story::items::ItemId;
use hush_sim::story::vitals::Vitals;
use hush_sim::story::{DayCycle, Milestones, MiningState, Stratum};

use crate::story::StoryApp;

/// Rastreador de métricas de cuadro: FPS, percentil 1% bajo (ventana móvil
/// de 120 cuadros) y frametime medio.
#[derive(Default)]
pub struct FpsTracker {
    frames: Vec<f32>,
}

impl FpsTracker {
    pub fn frame(&mut self, dt: f32) {
        self.frames.push(dt);
        if self.frames.len() > 120 {
            self.frames.remove(0);
        }
    }

    pub fn stats(&self) -> (f32, f32, f32) {
        if self.frames.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let total: f32 = self.frames.iter().sum();
        let fps = self.frames.len() as f32 / total.max(1e-6);
        let mut sorted = self.frames.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let cut = (sorted.len() / 100).max(1);
        let worst: f32 = sorted[sorted.len() - cut..].iter().sum();
        let low = cut as f32 / worst.max(1e-6);
        (fps, low, total * 1000.0 / self.frames.len() as f32)
    }
}

/// Vista copiada del estado del juego para dibujar el HUD.
pub struct HudView {
    pub paused: bool,
    pub f3_open: bool,
    pub dead: bool,
    pub craft_open: bool,
    pub craft_rows: Vec<(String, bool)>,
    pub craft_selected: usize,
    pub banner: Option<(&'static str, f32)>,
    pub vitals: Vitals,
    pub hotbar: Vec<Option<(String, u16, u16, u16)>>,
    pub selected: usize,
    pub milestones: Milestones,
    pub tick: u32,
    pub clock: DayCycle,
    pub feet: [i64; 3],
    pub stratum: Stratum,
    pub sky_light: u8,
    pub block_light: u8,
    pub mobs: usize,
    pub target: Option<RayHit>,
    pub target_name: &'static str,
    /// Dureza Q10 del bloque inspeccionado.
    pub target_hardness: u32,
    /// Nivel mínimo de pico para que suelte material.
    pub target_tier: u8,
    /// Luz que emite el bloque.
    pub target_emission: u8,
    /// ¿Es madera (inflamable por el magma)?
    pub target_flammable: bool,
    pub mining: MiningState,
    pub mesh_faces: usize,
    pub mesh_millis: f32,
    pub mesh_chunks: (usize, usize),
    pub fps: (f32, f32, f32),
    pub facing: &'static str,
    pub pitch: f32,
    pub yaw: f32,
    pub state_hash: u64,
}

impl HudView {
    pub fn of(app: &StoryApp) -> Self {
        let e = app.expedition();
        let feet = e.body().feet;
        let block_cell = feet.map(cell_of);
        let target = app.target();
        let target_kind = target.and_then(|hit| {
            BlockKind::from_id(e.world().get(hit.block[0], hit.block[1], hit.block[2])).ok()
        });
        let target_name = target_kind.map(BlockKind::name).unwrap_or("desconocido");
        let (target_hardness, target_tier, target_emission, target_flammable) = target_kind
            .map(|kind| {
                (
                    hush_sim::story::mining::hardness_q10(kind),
                    hush_sim::story::mining::required_tier(kind),
                    hush_sim::story::expedition::emission_of(kind.id()),
                    matches!(
                        kind,
                        BlockKind::Log
                            | BlockKind::Plank
                            | BlockKind::PropBeam
                            | BlockKind::Workbench
                            | BlockKind::Chest
                    ),
                )
            })
            .unwrap_or((0, 0, 0, false));
        Self {
            paused: app.paused(),
            f3_open: app.f3_open(),
            dead: e.is_dead(),
            craft_open: app.craft_menu().is_open(),
            craft_rows: app.craft_rows(),
            craft_selected: app.craft_menu().selected(),
            banner: app.banner(),
            vitals: *e.vitals(),
            hotbar: e
                .inventory()
                .hotbar
                .iter()
                .map(|slot| {
                    slot.map(|s| {
                        (
                            short_name(s.id).to_owned(),
                            s.count,
                            s.durability,
                            s.id.max_durability(),
                        )
                    })
                })
                .collect(),
            selected: usize::from(e.inventory().selected),
            milestones: e.milestones().clone(),
            tick: e.tick(),
            clock: *e.clock(),
            feet,
            stratum: e.bands().at(block_cell[1]),
            sky_light: e.sky_light_at(block_cell),
            block_light: e.block_light_at(block_cell),
            mobs: e.mobs().list.len(),
            target,
            target_name,
            target_hardness,
            target_tier,
            target_emission,
            target_flammable,
            mining: *e.mining(),
            mesh_faces: app.mesh_stats().faces,
            mesh_millis: app.mesh_stats().millis as f32,
            mesh_chunks: app.chunk_stats(),
            fps: app.fps_stats(),
            facing: app.facing(),
            pitch: app.pitch_deg(),
            yaw: app.yaw_deg(),
            state_hash: e.state_hash(),
        }
    }
}

pub fn draw(batch: &mut UiBatch, size: [u32; 2], view: &HudView) {
    let [w, h] = [size[0].max(1) as f32, size[1].max(1) as f32];
    draw_status(batch, view);
    draw_vitals(batch, w, h, view);
    draw_hotbar(batch, w, h, view);
    draw_goal(batch, w, view);
    draw_banner(batch, w, h, view);
    if view.craft_open {
        draw_craft(batch, w, h, view);
    }
    if view.paused {
        batch.rect(0.0, 0.0, w, h, [0.02, 0.03, 0.04, 0.80]);
        batch.text_centered(
            w * 0.5,
            h * 0.5 - 30.0,
            3.0,
            [0.93, 0.90, 0.83, 1.0],
            "PAUSA",
        );
        batch.text_centered(
            w * 0.5,
            h * 0.5 + 10.0,
            1.8,
            [0.7, 0.72, 0.75, 1.0],
            "ESC CONTINUAR | F FABRICAR | F3 TELEMETRIA | F5 CAMARA",
        );
    }
    if view.f3_open {
        draw_f3(batch, w, h, view);
    }
    if view.dead {
        batch.rect(0.0, 0.0, w, h, [0.25, 0.0, 0.0, 0.45]);
        batch.text_centered(
            w * 0.5,
            h * 0.5,
            4.0,
            [1.0, 0.4, 0.3, 1.0],
            "HAS DESFALLECIDO",
        );
    }
}

fn draw_status(batch: &mut UiBatch, view: &HudView) {
    let siege = if view.milestones.siege_started {
        format!(" | ASEDIO {}s", view.milestones.siege_ticks_left / 20)
    } else {
        String::new()
    };
    let status = format!(
        "[ESC] PAUSA | [F] FABRICAR | [E] USAR | [R] COMER | [F3] TELEMETRIA | {}{}",
        view.clock.phase().label(),
        siege
    );
    batch.panel(
        14.0,
        14.0,
        UiBatch::text_width(1.5, &status) + 20.0,
        26.0,
        [0.05, 0.07, 0.09, 0.82],
        [0.55, 0.44, 0.20, 0.9],
        2.0,
    );
    batch.text(24.0, 21.0, 1.5, [0.92, 0.89, 0.80, 1.0], &status);
}

fn draw_vitals(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    use hush_sim::story::vitals::{MAX_AIR, MAX_FOOD, MAX_HEALTH};
    let vitals = &view.vitals;
    let y = (h - 116.0).max(60.0);
    for i in 0..10u8 {
        let x = w * 0.5 - 232.0 + f32::from(i) * 22.0;
        let filled = vitals.health.saturating_sub(i * 2);
        let color = if filled >= 2 {
            [0.90, 0.15, 0.15, 1.0]
        } else if filled == 1 {
            [0.70, 0.10, 0.10, 1.0]
        } else {
            [0.18, 0.16, 0.16, 0.9]
        };
        heart(batch, x, y, color);
    }
    for i in 0..10u8 {
        let x = w * 0.5 + 14.0 + f32::from(i) * 22.0;
        let filled = vitals.food.saturating_sub(i * 2);
        let color = if filled >= 2 {
            [0.85, 0.60, 0.20, 1.0]
        } else if filled == 1 {
            [0.65, 0.45, 0.15, 1.0]
        } else {
            [0.18, 0.16, 0.16, 0.9]
        };
        drumstick(batch, x, y, color);
    }
    if vitals.air < MAX_AIR {
        let bubbles = usize::from(vitals.air) * 10 / usize::from(MAX_AIR);
        for i in 0..bubbles {
            let x = w * 0.5 + 14.0 + i as f32 * 18.0;
            batch.rect(x, y - 22.0, 12.0, 12.0, [0.35, 0.60, 0.95, 0.95]);
        }
    }
    let _ = (MAX_FOOD, MAX_HEALTH);
}

fn heart(batch: &mut UiBatch, x: f32, y: f32, color: [f32; 4]) {
    batch.rect(x, y, 16.0, 6.0, color);
    batch.rect(x + 2.0, y + 6.0, 12.0, 5.0, color);
    batch.rect(x + 5.0, y + 11.0, 6.0, 4.0, color);
}

fn drumstick(batch: &mut UiBatch, x: f32, y: f32, color: [f32; 4]) {
    batch.rect(x, y, 12.0, 10.0, color);
    batch.rect(x + 8.0, y + 9.0, 5.0, 5.0, [0.75, 0.68, 0.55, 1.0]);
}

fn draw_hotbar(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    let (slot_w, gap) = (94.0, 6.0);
    let total_w = 9.0 * slot_w + 8.0 * gap;
    let start_x = ((w - total_w) * 0.5).round();
    let y = (h - 64.0).max(40.0);
    for i in 0..9 {
        let x = start_x + i as f32 * (slot_w + gap);
        let active = i == view.selected;
        let (bg, border) = if active {
            ([0.22, 0.18, 0.08, 0.94], [0.95, 0.78, 0.32, 1.0])
        } else {
            ([0.06, 0.08, 0.10, 0.84], [0.28, 0.32, 0.38, 0.9])
        };
        batch.panel(x, y, slot_w, 30.0, bg, border, 2.0);
        let label = view.hotbar[i]
            .as_ref()
            .map(|(name, count, _, _)| {
                if *count > 1 {
                    format!("{name}:{count}")
                } else {
                    name.clone()
                }
            })
            .unwrap_or_else(|| format!("{}", i + 1));
        let color = if active {
            [1.0, 0.92, 0.55, 1.0]
        } else {
            [0.82, 0.84, 0.88, 1.0]
        };
        batch.text_centered(x + slot_w * 0.5, y + 10.0, 1.5, color, &label);
        if let Some((_, _, durability, max)) = &view.hotbar[i]
            && *max > 0
        {
            let frac = f32::from(*durability) / f32::from(*max);
            batch.rect(
                x + 6.0,
                y + 26.0,
                (slot_w - 12.0) * frac.clamp(0.0, 1.0),
                2.0,
                [0.4, 0.9, 0.4, 1.0],
            );
        }
    }
    // Progreso de fractura del bloque en minado.
    if let Some(cell) = view.mining.cell {
        let frac = if view.mining.total > 0 {
            view.mining.progress.min(view.mining.total) as f32 / view.mining.total as f32
        } else {
            0.0
        };
        batch.panel(
            start_x,
            y - 16.0,
            total_w,
            10.0,
            [0.05, 0.07, 0.09, 0.85],
            [0.3, 0.34, 0.4, 0.9],
            1.0,
        );
        batch.rect(
            start_x + 2.0,
            y - 14.0,
            (total_w - 4.0) * frac,
            6.0,
            [0.95, 0.6, 0.15, 1.0],
        );
        let _ = cell;
    }
}

fn draw_goal(batch: &mut UiBatch, w: f32, view: &HudView) {
    use hush_sim::story::Milestone;
    let next = Milestone::ALL
        .get(usize::from(view.milestones.reached))
        .copied()
        .map_or("EXPEDICION COMPLETA".to_owned(), |m| {
            format!("{}: {}", m.label(), m.goal())
        });
    let label = format!("HITO {}/9 | {next}", view.milestones.reached);
    let tw = UiBatch::text_width(1.5, &label);
    batch.panel(
        w - tw - 46.0,
        14.0,
        tw + 24.0,
        26.0,
        [0.05, 0.07, 0.09, 0.82],
        [0.4, 0.5, 0.6, 0.9],
        2.0,
    );
    batch.text(w - tw - 34.0, 21.0, 1.5, [0.85, 0.9, 0.95, 1.0], &label);
}

fn draw_banner(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    if let Some((text, _)) = view.banner {
        batch.text_centered(w * 0.5, h * 0.28, 3.0, [1.0, 0.85, 0.4, 1.0], text);
    }
}

fn draw_craft(batch: &mut UiBatch, w: f32, h: f32, view: &HudView) {
    batch.rect(0.0, 0.0, w, h, [0.02, 0.03, 0.04, 0.75]);
    let (pw, ph) = (640.0f32.min(w - 32.0), 540.0f32.min(h - 32.0));
    let (px, py) = (((w - pw) * 0.5).round(), ((h - ph) * 0.5).round());
    batch.panel(
        px,
        py,
        pw,
        ph,
        [0.05, 0.07, 0.09, 0.96],
        [0.85, 0.69, 0.32, 1.0],
        3.0,
    );
    batch.text_centered(
        w * 0.5,
        py + 16.0,
        2.2,
        [0.93, 0.90, 0.83, 1.0],
        "FABRICACION (F PARA CERRAR)",
    );
    let per_page = 15;
    let first = view.craft_selected.saturating_sub(per_page - 1);
    for (idx, (label, has_materials)) in view
        .craft_rows
        .iter()
        .enumerate()
        .skip(first)
        .take(per_page)
    {
        let row = idx - first;
        let y = py + 56.0 + row as f32 * 30.0;
        let active = idx == view.craft_selected;
        let (bg, border) = if active {
            ([0.16, 0.24, 0.18, 0.98], [0.41, 0.94, 0.56, 1.0])
        } else {
            ([0.09, 0.11, 0.14, 0.92], [0.28, 0.31, 0.38, 1.0])
        };
        batch.panel(px + 24.0, y, pw - 48.0, 26.0, bg, border, 2.0);
        let color = if *has_materials {
            [0.85, 0.95, 0.85, 1.0]
        } else {
            [0.55, 0.55, 0.55, 1.0]
        };
        batch.text(px + 34.0, y + 9.0, 1.5, color, label);
    }
    batch.text_centered(
        w * 0.5,
        py + ph - 22.0,
        1.5,
        [0.85, 0.69, 0.32, 1.0],
        "FLECHAS ELIGEN | ENTER FABRICA | GRIS = FALTAN MATERIALES",
    );
}

fn draw_f3(batch: &mut UiBatch, w: f32, _h: f32, view: &HudView) {
    let block_cell = view.feet.map(cell_of);
    let chunk = block_cell.map(|c| c.div_euclid(16));
    let local = block_cell.map(|c| c.rem_euclid(16));
    let region = chunk.map(|c| c.div_euclid(32));

    let left = [
        format!(
            "HUSHMINE {} ({} / {})",
            env!("CARGO_PKG_VERSION"),
            profile(),
            arch()
        ),
        format!(
            "FPS {:.0} | 1% LOW {:.0} | {:.2} MS",
            view.fps.0, view.fps.1, view.fps.2
        )
        .replace(".\n2}", ".2}"),
        String::new(),
        format!(
            "POS FIJA X{} Y{} Z{} (Q10)",
            view.feet[0], view.feet[1], view.feet[2]
        ),
        format!(
            "BLOQUE X{} Y{} Z{}",
            block_cell[0], block_cell[1], block_cell[2]
        ),
        format!(
            "CHUNK {},{},{} LOCAL {},{},{}",
            chunk[0], chunk[1], chunk[2], local[0], local[1], local[2]
        ),
        format!("REGION {},{},{}", region[0], region[1], region[2]),
        String::new(),
        format!(
            "RUMBO {} PITCH {:+.1} YAW {:.1}",
            view.facing, view.pitch, view.yaw
        ),
        format!("ESTRATO {}", view.stratum.label()),
        format!(
            "FASE {} (TICK {})",
            view.clock.phase().label(),
            view.clock.tick()
        ),
        format!(
            "LUZ CIELO {} | LUZ BLOQUE {}",
            view.sky_light, view.block_light
        ),
        String::new(),
        format!(
            "VIDA {} COMIDA {} AIRE {}",
            view.vitals.health, view.vitals.food, view.vitals.air
        ),
        format!("ENEMIGOS {} | TICK {}", view.mobs, view.tick),
    ];
    let panel_w = 350.0f32.min(w - 24.0);
    let lh = 15.0;
    let panel_h = left.len() as f32 * lh + 14.0;
    batch.rect(8.0, 46.0, panel_w, panel_h, [0.0, 0.0, 0.0, 0.62]);
    for (i, line) in left.iter().enumerate() {
        if !line.is_empty() {
            batch.text(14.0, 52.0 + i as f32 * lh, 1.3, [0.9, 0.95, 0.8, 1.0], line);
        }
    }

    let mut right = vec![
        format!(
            "MALLAS {}/{} CHUNKS",
            view.mesh_chunks.0, view.mesh_chunks.1
        ),
        format!(
            "CARAS {} | REMALLADO {:.1} MS",
            view.mesh_faces, view.mesh_millis
        ),
        format!("SIMULACION 20 HZ | HASH {:016X}", view.state_hash),
        String::new(),
        "OBJETIVO (RAYCAST 5 M)".to_owned(),
    ];
    if let Some(hit) = view.target {
        right.push(format!(
            "BLOQUE X{} Y{} Z{} | {}",
            hit.block[0], hit.block[1], hit.block[2], view.target_name
        ));
        right.push(format!(
            "NORMAL ({},{},{}) | DUREZA {} | NIVEL PICO {}",
            hit.normal[0], hit.normal[1], hit.normal[2], view.target_hardness, view.target_tier
        ));
        right.push(format!(
            "EMISION LUZ {} | FLAMABLE {}",
            view.target_emission,
            if view.target_flammable { "SI" } else { "NO" }
        ));
    } else {
        right.push("SIN BLOQUE A LA VISTA".to_owned());
    }
    let right_w = 310.0f32.min(w - 24.0);
    let right_h = right.len() as f32 * lh + 14.0;
    let rx = (w - right_w - 8.0).max(8.0);
    batch.rect(rx, 46.0, right_w, right_h, [0.0, 0.0, 0.0, 0.62]);
    for (i, line) in right.iter().enumerate() {
        if !line.is_empty() {
            batch.text(
                rx + 6.0,
                52.0 + i as f32 * lh,
                1.3,
                [0.85, 0.88, 0.95, 1.0],
                line,
            );
        }
    }
}

/// Nombre corto de ítem para la barra rápida.
fn short_name(id: ItemId) -> &'static str {
    match id.name() {
        "dirt" => "TIERRA",
        "rock" => "ROCA",
        "hard_rock" => "ROCA DURA",
        "ore" => "MENA",
        "gold" => "ORO",
        "diamond" => "DIAMANTE",
        "plank" => "TABLON",
        "log" => "TRONCO",
        "torch" => "ANTORCHA",
        "workbench" => "BANCO",
        "furnace" => "HORNO",
        "chest" => "ARCON",
        "gravel" => "GRAVA",
        "water" => "AGUA",
        "obsidian" => "OBSIDIANA",
        "coal_ore" | "coal" => "CARBON",
        "handle" => "MANGO",
        "iron_ingot" => "HIERRO",
        "gold_ingot" => "ORO LINGOTE",
        "diamond_gem" => "GEMA",
        "coal_block" => "BLOQ CARBON",
        "steel_ingot" => "ACERO",
        "ration" => "RACION",
        "wood_pickaxe" => "PICO MADERA",
        "wood_shovel" => "PALA MADERA",
        "wood_axe" => "HACHA MADERA",
        "wood_sword" => "ESPADA MADERA",
        "stone_pickaxe" => "PICO PIEDRA",
        "stone_shovel" => "PALA PIEDRA",
        "stone_axe" => "HACHA PIEDRA",
        "stone_sword" => "ESPADA PIEDRA",
        "iron_pickaxe" => "PICO HIERRO",
        "iron_shovel" => "PALA HIERRO",
        "iron_axe" => "HACHA HIERRO",
        "iron_sword" => "ESPADA HIERRO",
        "steel_pickaxe" => "PICO ACERO",
        "steel_shovel" => "PALA ACERO",
        "steel_axe" => "HACHA ACERO",
        "steel_sword" => "ESPADA ACERO",
        "diamond_pickaxe" => "PICO DIAMANTE",
        "diamond_shovel" => "PALA DIAMANTE",
        "diamond_axe" => "HACHA DIAMANTE",
        "diamond_sword" => "ESPADA DIAMANTE",
        _ => "ITEM",
    }
}

fn profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

fn arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "otra"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_tracker_window_and_percentile() {
        let mut fps = FpsTracker::default();
        // 60 FPS constantes: 1% bajo igual.
        for _ in 0..120 {
            fps.frame(1.0 / 60.0);
        }
        let (avg, low, ms) = fps.stats();
        assert!((avg - 60.0).abs() < 1.0);
        assert!((low - 60.0).abs() < 1.0);
        assert!((ms - 16.67).abs() < 0.2);
        // Un tirón de 200 ms hunde el 1% bajo.
        fps.frame(0.2);
        let (_, low, _) = fps.stats();
        assert!(low < 10.0, "1% bajo demasiado alto: {low}");
        // La ventana se limita a 120 cuadros.
        for _ in 0..500 {
            fps.frame(1.0 / 60.0);
        }
        assert!(fps.frames.len() <= 120);
    }
}
