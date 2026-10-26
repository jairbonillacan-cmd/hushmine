//! Escenas de captura de la interfaz: el lanzador, el menú del juego y la
//! muestra de la fuente, dibujados con el mismo código que la ventana real.

use super::*;

/// Lanzador completo sobre el mapa visto desde lejos, con el mismo dibujo
/// que la ventana real. La semilla elige la página: 2 Instalaciones,
/// 3 Opciones, 4 Acerca de; cualquier otra, Jugar.
pub(super) fn launcher_scene(world: VoxelWorld, seed: u64) -> TerrainScene {
    use hush_launcher::draw::draw as draw_launcher;
    use hush_launcher::state::{LauncherState, Page};
    use hush_launcher::versions;

    let root = versions::workspace_root();
    let mut state = LauncherState::new(
        versions::discover_versions(&root),
        hush_launcher::history::load_last_played(&root),
        0,
    );
    let page = match seed {
        2 => Page::Instalaciones,
        3 => Page::Opciones,
        4 => Page::Acerca,
        _ => Page::Play,
    };
    state.select_page(page);
    // Fundido terminado para que cada captura salga igual.
    state.page_fade = hush_launcher::theme::FADE_TIME;
    if let Ok(textures) = hush_art::all_block_textures() {
        hush_launcher::thumbnail::refresh(&mut state, &textures);
    }
    let mut ui = UiBatch::new();
    draw_launcher(&mut ui, &state, 1024.0, 640.0);
    TerrainScene {
        world: SceneWorld::Finite(world),
        camera: Camera::Orbit(hush_launcher::panorama::panorama_camera(0.9)),
        boxes: Vec::new(),
        ui,
        style: TerrainStyle {
            fog_distance: 320.0,
            selected: None,
            lamp_radius: 0.0,
            time_seconds: 0.0,
            ..TerrainStyle::default()
        },
    }
}

/// Muestra de la fuente sobre fondo negro, a los tamaños que usan los menús.
pub(super) fn type_scene(world: VoxelWorld) -> TerrainScene {
    const LINES: [&str; 6] = [
        "HUSHMINE — Bajar al turno",
        "¿Versión rápida? ¡Sí! Ñandú, pingüino, Vigía.",
        "abcdefghijklmnopqrstuvwxyz áéíóúüñ",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ ÁÉÍÓÚÜÑ",
        "0123456789 · 1931–1979 · 41 fichas · 100 %",
        "Semilla 42 · 4 bots · 3,5 bloques · Desde atrás",
    ];
    let mut ui = UiBatch::new();
    ui.rect(0.0, 0.0, 4096.0, 4096.0, [0.04, 0.043, 0.055, 1.0]);
    let mut y = 24.0;
    for scale in [4.0, 3.0, 2.0] {
        for line in LINES {
            ui.text(24.0, y, scale, [0.93, 0.90, 0.83, 1.0], line);
            y += UiBatch::text_height(scale) + 6.0 * scale;
        }
        y += 12.0;
    }
    let mut scene = players_scene(world);
    scene.ui = ui;
    scene
}

/// Menú del juego sobre la escena en tercera persona, con el mismo dibujo
/// que el cliente (`hush_client::hud`). La semilla elige qué se ve: 2
/// Controles, 3 Nueva partida, 4 solo el HUD corriendo; cualquier otra,
/// Pausa.
pub(super) fn menu_scene(world: VoxelWorld, seed: u64) -> TerrainScene {
    use grisu_render::CameraMode;
    use hush_client::controls::Controls;
    use hush_client::hud::draw_ui;
    use hush_client::menu::{MenuPage, MenuState};

    let mut menu = MenuState::new(false, 42, 4, false);
    menu.page = match seed {
        2 => MenuPage::Controls {
            from_launcher: false,
        },
        3 => MenuPage::Launcher,
        4 => MenuPage::Hidden,
        _ => MenuPage::Pause,
    };
    menu.selected = 1;
    let mut ui = UiBatch::new();
    draw_ui(
        &menu,
        &mut ui,
        [1024, 640],
        &Controls::default(),
        CameraMode::ThirdPersonBack,
        seed == 4,
        BlockKind::Gold,
    );
    let mut scene = third_person_scene(world);
    scene.ui = ui;
    scene
}
