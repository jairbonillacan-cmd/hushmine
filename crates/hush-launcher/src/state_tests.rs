use super::*;
use crate::testutil::sample_versions_for_tests;

fn state() -> LauncherState {
    LauncherState::new(sample_versions_for_tests(), Vec::new(), 1_000)
}

#[test]
fn play_page_cycles_items_and_launches() {
    let mut s = state();
    assert_eq!(s.page(), Page::Play);
    assert_eq!(s.selected(), 1);
    s.move_selection(1);
    assert_eq!(s.selected(), 2);
    assert_eq!(s.activate(), Some(PendingAction::Compile { version: 0 }));
    s.move_selection(-1);
    s.move_selection(-1);
    assert_eq!(s.selected(), 0);
    s.adjust(1);
    assert_eq!(s.current_version().label, "V1 DEBUG");
}

#[test]
fn play_page_has_a_server_button() {
    let mut s = state();
    assert_eq!(s.item_count(Page::Play), 4);
    s.select_item(3);
    assert_eq!(s.item_kind(Page::Play, 3), ItemKind::Button);
    assert_eq!(s.activate(), Some(PendingAction::StartServer));
}

#[test]
fn changing_page_restarts_the_fade() {
    let mut s = state();
    s.tick(crate::theme::FADE_TIME);
    assert!((s.page_fade - crate::theme::FADE_TIME).abs() < 1e-6);
    s.select_page(Page::Opciones);
    assert_eq!(s.page_fade, 0.0);
    s.select_page(Page::Opciones); // misma pagina: no reinicia
    assert_eq!(s.page_fade, 0.0);
    s.cycle_page(1);
    assert_eq!(s.page_fade, 0.0);
    s.tick(0.05);
    assert!((s.page_fade - 0.05).abs() < 1e-5);
    // El avance se queda en el maximo, no crece sin fin.
    s.tick(crate::theme::FADE_TIME * 10.0);
    assert!((s.page_fade - crate::theme::FADE_TIME).abs() < 1e-5);
}

#[test]
fn instalaciones_selects_and_launches_card() {
    let mut s = state();
    s.select_page(Page::Instalaciones);
    assert_eq!(s.item_count(Page::Instalaciones), 3);
    s.move_selection(1);
    assert_eq!(s.activate(), Some(PendingAction::Launch { version: 1 }));
    assert_eq!(s.current_version_idx(), 1);
}

#[test]
fn opciones_adjust_every_row() {
    let mut s = state();
    s.select_page(Page::Opciones);
    s.move_selection(-1); // fila 4: al lanzar
    assert_eq!(s.selected(), 4);
    s.activate();
    assert!(s.close_on_launch);
    s.move_selection(-1); // fila 3: perfil
    let before = s.current_miner();
    s.adjust(-1);
    assert_ne!(s.current_miner(), before);
    s.move_selection(-1); // fila 2: semilla
    s.adjust(1);
    assert_eq!(s.current_seed(), 1931);
    s.move_selection(-1); // fila 1: bots (ya en 4, sube a 5)
    s.adjust(1);
    assert_eq!(s.bots, 5);
    s.bots = MAX_BOTS;
    s.adjust(1);
    assert_eq!(s.bots, MAX_BOTS);
}

#[test]
fn cycle_page_walks_all_pages() {
    let mut s = state();
    s.cycle_page(1);
    assert_eq!(s.page(), Page::Instalaciones);
    s.cycle_page(1);
    s.cycle_page(1);
    assert_eq!(s.page(), Page::Acerca);
    s.cycle_page(1);
    assert_eq!(s.page(), Page::Play);
    s.cycle_page(-1);
    assert_eq!(s.page(), Page::Acerca);
}

#[test]
fn record_launch_keeps_latest_per_label() {
    let mut s = state();
    s.record_launch(0, 2_000);
    s.record_launch(0, 3_000);
    assert_eq!(
        s.last_played_text(0),
        crate::history::format_age(3_000, 3_000)
    );
    assert_eq!(s.last_played_text(1), "Sin jugar");
}
