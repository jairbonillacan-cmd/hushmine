//! Pruebas del menú con entrada simulada. Ninguna toca opciones que guardan
//! `out/controls.cfg`, para no escribir en disco.

use super::*;
use crate::hud::row_rect;

const WINDOW: (u32, u32) = (1280, 720);

fn press(menu: &mut MenuState, key: KeyCode, controls: &mut Controls) -> MenuCommand {
    let mut input = Input::simulated(WINDOW, (-1.0, -1.0));
    input.simulate_key(key, true);
    let mut camera = CameraMode::FirstPerson;
    menu.update(&input, controls, &mut camera)
}

#[test]
fn escape_opens_and_closes_the_pause_menu() {
    let mut controls = Controls::default();
    let mut menu = MenuState::new(false, 42, 4, false);
    assert!(!menu.is_open());
    press(&mut menu, KeyCode::Escape, &mut controls);
    assert_eq!(menu.page, MenuPage::Pause);
    assert_eq!(menu.title(), "Pausa");
    press(&mut menu, KeyCode::Escape, &mut controls);
    assert!(!menu.is_open());
}

#[test]
fn new_game_page_adjusts_bots_and_starts_with_them() {
    let mut controls = Controls::default();
    let mut menu = MenuState::new(true, 42, 4, false);
    press(&mut menu, KeyCode::KeyS, &mut controls);
    assert_eq!(menu.selected, 1);
    press(&mut menu, KeyCode::KeyD, &mut controls);
    press(&mut menu, KeyCode::KeyD, &mut controls);
    assert_eq!(menu.launcher_bots, 6);
    press(&mut menu, KeyCode::KeyW, &mut controls);
    let cmd = press(&mut menu, KeyCode::Enter, &mut controls);
    assert_eq!(cmd, MenuCommand::StartLocal { seed: 42, bots: 6 });
    assert!(!menu.is_open());
}

#[test]
fn quitting_from_pause_asks_to_exit() {
    let mut controls = Controls::default();
    let mut menu = MenuState::new(false, 1, 0, false);
    press(&mut menu, KeyCode::Escape, &mut controls);
    // Hacia arriba desde la primera fila se llega a la última: salir.
    press(&mut menu, KeyCode::KeyW, &mut controls);
    assert_eq!(menu.selected, menu.item_count() - 1);
    assert_eq!(
        press(&mut menu, KeyCode::Enter, &mut controls),
        MenuCommand::Exit
    );
}

#[test]
fn clicking_a_row_selects_and_activates_it() {
    let mut controls = Controls::default();
    let mut menu = MenuState::new(false, 1, 0, false);
    press(&mut menu, KeyCode::Escape, &mut controls);
    let row = row_rect(menu.item_count(), 0, WINDOW.0 as f32, WINDOW.1 as f32);
    let (x, y) = (row.rect[0] + 10.0, row.rect[1] + row.rect[3] * 0.5);
    let mut input = Input::simulated(WINDOW, (x, y));
    input.simulate_button(MouseButton::Left, true);
    let mut camera = CameraMode::FirstPerson;
    menu.update(&input, &mut controls, &mut camera);
    // La primera fila es «Seguir jugando».
    assert!(!menu.is_open());
}

#[test]
fn rows_match_the_hit_test_count_and_read_as_sentences() {
    let controls = Controls::default();
    let pages = [
        MenuPage::Launcher,
        MenuPage::Pause,
        MenuPage::Controls {
            from_launcher: false,
        },
    ];
    for page in pages {
        let mut menu = MenuState::new(false, 42, 4, false);
        menu.page = page;
        let rows = menu.item_rows(&controls, CameraMode::FirstPerson);
        assert_eq!(rows.len(), menu.item_count(), "{page:?}");
        for (idx, (label, value)) in rows.iter().enumerate() {
            assert_ne!(label, &label.to_uppercase(), "{page:?}: {label}");
            if menu.is_adjustable(idx) {
                assert!(!value.is_empty(), "{page:?}: {label} sin valor");
            }
        }
    }
}
