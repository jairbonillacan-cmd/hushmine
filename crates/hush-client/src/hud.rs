//! Dibujo 2D del HUD en partida y de los paneles del menú.

use grisu_render::{CameraMode, UiBatch, UiRect};
use hush_sim::BlockKind;

use crate::controls::{Controls, key_label};
use crate::menu::{MenuPage, MenuState};

pub fn panel_dims(page: MenuPage, w: f32, h: f32) -> (f32, f32) {
    match page {
        MenuPage::Controls { .. } => (680.0f32.min(w - 32.0), 540.0f32.min(h - 32.0)),
        _ => (620.0f32.min(w - 32.0), 430.0f32.min(h - 32.0)),
    }
}

pub fn row_rect(page: MenuPage, idx: usize, w: f32, h: f32) -> UiRect {
    let (pw, ph) = panel_dims(page, w, h);
    let (row_h, gap) = if matches!(page, MenuPage::Controls { .. }) {
        (32.0, 6.0)
    } else {
        (40.0, 8.0)
    };
    let (px, py) = (((w - pw) * 0.5).round(), ((h - ph) * 0.5).round());
    let ry = py + 70.0 + idx as f32 * (row_h + gap);
    UiRect::new(px + 36.0, ry, pw - 72.0, row_h, [0.0; 4])
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
    draw_hud(batch, w, h, controls, camera_mode, sprinting, place_block);
    if !menu.is_open() {
        return;
    }
    batch.rect(0.0, 0.0, w, h, [0.02, 0.03, 0.04, 0.80]);

    let (pw, ph) = panel_dims(menu.page, w, h);
    let (px, py) = (((w - pw) * 0.5).round(), ((h - ph) * 0.5).round());
    let brass = [0.85, 0.69, 0.32, 1.0];
    batch.panel(px, py, pw, ph, [0.05, 0.07, 0.09, 0.96], brass, 3.0);
    batch.rect(px + 3.0, py + 3.0, pw - 6.0, 52.0, [0.10, 0.12, 0.16, 1.0]);

    let title = match menu.page {
        MenuPage::Launcher => "HUSHMINE // LAUNCHER - CASETA DEL VIGIA",
        MenuPage::Pause => "HUSHMINE // PAUSA Y OPCIONES (ESC)",
        MenuPage::Controls { .. } => "HUSHMINE // CONTROLES Y CAMARA",
        MenuPage::Hidden => "",
    };
    batch.text_centered(w * 0.5, py + 20.0, 2.4, [0.93, 0.90, 0.83, 1.0], title);

    for (idx, label) in menu.item_labels(controls, camera_mode).iter().enumerate() {
        let r = row_rect(menu.page, idx, w, h);
        let active = idx == menu.selected;
        let row_bg = if active {
            [0.16, 0.24, 0.18, 0.98]
        } else {
            [0.09, 0.11, 0.14, 0.92]
        };
        let border = if active {
            [0.41, 0.94, 0.56, 1.0]
        } else {
            [0.28, 0.31, 0.38, 1.0]
        };
        batch.panel(
            r.rect[0], r.rect[1], r.rect[2], r.rect[3], row_bg, border, 2.0,
        );
        let col = if active {
            [0.78, 1.0, 0.84, 1.0]
        } else {
            [0.90, 0.88, 0.82, 1.0]
        };
        let ty = r.rect[1] + (r.rect[3] - 14.0) * 0.5;
        batch.text_centered(w * 0.5, ty, 2.0, col, label);
    }

    let footer = if menu.rebinding.is_some() {
        "PULSA LA NUEVA TECLA PARA ASIGNAR (ESC PARA CANCELAR)"
    } else {
        "CLIC RATON O FLECHAS + ENTER | IZQ/DER AJUSTA VALORES"
    };
    batch.text_centered(w * 0.5, py + ph - 24.0, 1.6, brass, footer);
}

fn draw_hud(
    batch: &mut UiBatch,
    w: f32,
    h: f32,
    controls: &Controls,
    camera_mode: CameraMode,
    sprinting: bool,
    place_block: BlockKind,
) {
    let status = format!(
        "[ESC] MENU | [{}/F5] {} | [{}] CORRER: {}",
        key_label(controls.camera),
        camera_mode.label(),
        key_label(controls.sprint),
        if sprinting { "ACTIVO" } else { "LISTO" }
    );
    let sw = UiBatch::text_width(1.6, &status) + 20.0;
    batch.panel(
        14.0,
        14.0,
        sw,
        28.0,
        [0.05, 0.07, 0.09, 0.82],
        [0.55, 0.44, 0.20, 0.9],
        2.0,
    );
    batch.text(24.0, 22.0, 1.6, [0.92, 0.89, 0.80, 1.0], &status);

    let slots = [
        (BlockKind::Dirt, "1:TIERRA"),
        (BlockKind::Rock, "2:ROCA"),
        (BlockKind::HardRock, "3:DURA"),
        (BlockKind::Ore, "4:MENA"),
        (BlockKind::Gold, "5:ORO"),
        (BlockKind::Diamond, "6:DIAMANTE"),
    ];
    let (slot_w, gap) = (112.0, 6.0);
    let total_w = slots.len() as f32 * slot_w + (slots.len() - 1) as f32 * gap;
    let (start_x, y) = (((w - total_w) * 0.5).round(), (h - 46.0).max(50.0));
    for (i, (kind, label)) in slots.iter().enumerate() {
        let x = start_x + i as f32 * (slot_w + gap);
        let sel = *kind == place_block;
        let bg = if sel {
            [0.22, 0.18, 0.08, 0.94]
        } else {
            [0.06, 0.08, 0.10, 0.84]
        };
        let brd = if sel {
            [0.95, 0.78, 0.32, 1.0]
        } else {
            [0.28, 0.32, 0.38, 0.9]
        };
        let col = if sel {
            [1.0, 0.92, 0.55, 1.0]
        } else {
            [0.82, 0.84, 0.88, 1.0]
        };
        batch.panel(x, y, slot_w, 32.0, bg, brd, 2.0);
        batch.text_centered(x + slot_w * 0.5, y + 10.0, 1.6, col, label);
    }
}
