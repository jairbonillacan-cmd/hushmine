//! Lanzador gráfico (`hush-launcher.exe`): panel con barra lateral para
//! elegir versión, ajustar la partida, compilar y lanzar `hushmine.exe`.

#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use grisu_platform::{App, Input, KeyCode, MouseButton};
use grisu_render::{CubeRenderer, DepthTarget, Gpu, TextureData, UiBatch, UiRenderer};
use hush_art::all_block_textures;
use hush_launcher::draw;
use hush_launcher::history;
use hush_launcher::layout::{HitSlot, Layout, layout};
use hush_launcher::panorama::{PANORAMA_SEED, PanoramaRenderer};
use hush_launcher::settings::{self, SavedSettings};
use hush_launcher::state::{ItemKind, LauncherState, Page, PendingAction};
use hush_launcher::thumbnail;
use hush_launcher::versions::{
    compile_version, discover_versions, launch_game, launch_server, workspace_root,
};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let root = workspace_root();
    let launcher = LauncherApp::new(root);
    grisu_platform::run_with_size("HUSHMINE", [1024, 640], launcher)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

struct LauncherApp {
    root: PathBuf,
    state: LauncherState,
    close_requested: bool,
    panorama: Option<PanoramaRenderer>,
    cube: Option<CubeRenderer>,
    saved: Option<SavedSettings>,
    textures: Vec<TextureData>,
    ui: Option<UiRenderer>,
    batch: UiBatch,
}

impl LauncherApp {
    fn new(root: PathBuf) -> Self {
        let versions = discover_versions(&root);
        let last_played = history::load_last_played(&root);
        let total = versions.len();
        let mut state = LauncherState::new(versions, last_played, now_secs());
        let saved = settings::load_settings(&root);
        if let Some(values) = saved {
            values.apply(&mut state);
        }
        // Con las capas del modo Historia, para su miniatura.
        let textures = all_block_textures().unwrap_or_default();
        if !textures.is_empty() {
            thumbnail::refresh(&mut state, &textures);
        }
        let mut app = Self {
            root,
            state,
            close_requested: false,
            panorama: None,
            cube: None,
            saved,
            textures,
            ui: None,
            batch: UiBatch::new(),
        };
        app.set_status(format!("Listo · {total} versiones encontradas"), true);
        app
    }

    fn set_status(&mut self, msg: String, ok: bool) {
        self.state.set_status(msg, ok);
    }

    fn handle(&mut self, action: PendingAction) {
        match action {
            PendingAction::Launch { version } => {
                let ver = self.state.versions[version].clone();
                let res = launch_game(
                    &self.root,
                    &ver,
                    self.state.start_mode,
                    self.state.current_seed(),
                    self.state.bots,
                    self.state.current_miner(),
                );
                self.refresh_versions();
                match res {
                    Ok(msg) => {
                        let now = now_secs();
                        history::record_last_played(&self.root, &ver.label, now);
                        self.state.record_launch(version, now);
                        self.set_status(msg, true);
                        if self.state.close_on_launch {
                            self.close_requested = true;
                        }
                    }
                    Err(err) => self.set_status(err, false),
                }
            }
            PendingAction::Compile { version } => {
                let ver = self.state.versions[version].clone();
                let res = compile_version(&self.root, &ver);
                self.refresh_versions();
                match res {
                    Ok(msg) => self.set_status(msg, true),
                    Err(err) => self.set_status(err, false),
                }
            }
            PendingAction::StartServer => {
                let res = launch_server(&self.root, self.state.current_seed(), self.state.bots);
                match res {
                    Ok(msg) => self.set_status(msg, true),
                    Err(err) => self.set_status(err, false),
                }
            }
        }
    }

    fn refresh_versions(&mut self) {
        self.state.versions = discover_versions(&self.root);
        if self.state.version_idx >= self.state.versions.len() {
            self.state.version_idx = 0;
        }
    }

    /// Resuelve un clic con el ratón sobre la barra lateral o los elementos.
    fn click(&mut self, lay: &Layout) {
        let (mx, my) = self.state.mouse;
        match lay.sidebar_at(mx, my) {
            Some(HitSlot::SidebarPage(page)) => {
                self.state.select_page(page);
                return;
            }
            Some(HitSlot::Exit) => {
                self.close_requested = true;
                return;
            }
            None => {}
        }
        let Some(item) = lay.item_at(mx, my) else {
            return;
        };
        let kind = self.state.item_kind(self.state.page(), item);
        let already = self.state.selected() == item;
        self.state.select_item(item);
        if kind == ItemKind::Adjuster {
            // Tercios: izquierda baja, derecha sube, centro avanza.
            if let Some(rect) = lay.item_rect(item) {
                let [x, _, w, _] = rect.rect;
                let third = w / 3.0;
                if mx < x + third {
                    self.state.adjust(-1);
                    return;
                }
                if mx > x + 2.0 * third {
                    self.state.adjust(1);
                    return;
                }
            }
        }
        // Las tarjetas se lanzan con el segundo clic; el resto, al primero.
        if kind == ItemKind::Card && !already {
            return;
        }
        if let Some(action) = self.state.activate() {
            self.handle(action);
        }
    }
}

impl App for LauncherApp {
    fn init(&mut self, gpu: &Gpu, surface_format: wgpu::TextureFormat) {
        match PanoramaRenderer::new(gpu, surface_format, PANORAMA_SEED) {
            Ok(panorama) => self.panorama = Some(panorama),
            Err(err) => {
                log::warn!("panorama no disponible, se usa el cubo: {err:#}");
                let tex = TextureData::test_ore();
                self.cube = Some(CubeRenderer::new(
                    &gpu.device,
                    &gpu.queue,
                    surface_format,
                    &tex,
                ));
            }
        }
        self.ui = Some(UiRenderer::new(&gpu.device, surface_format));
    }

    fn update(&mut self, dt: f32, input: &Input) {
        self.state.tick(dt);
        if let Some(panorama) = &mut self.panorama {
            panorama.advance(dt);
        }
        thumbnail::refresh(&mut self.state, &self.textures);
        if input.was_pressed(KeyCode::Escape) {
            self.close_requested = true;
            return;
        }
        if input.was_pressed(KeyCode::Tab) || input.was_pressed(KeyCode::KeyE) {
            self.state.cycle_page(1);
        }
        if input.was_pressed(KeyCode::KeyQ) {
            self.state.cycle_page(-1);
        }
        for (code, page) in [
            (KeyCode::Digit1, Page::Play),
            (KeyCode::Digit2, Page::Instalaciones),
            (KeyCode::Digit3, Page::Opciones),
            (KeyCode::Digit4, Page::Acerca),
        ] {
            if input.was_pressed(code) {
                self.state.select_page(page);
            }
        }
        if input.was_pressed(KeyCode::ArrowUp) || input.was_pressed(KeyCode::KeyW) {
            self.state.move_selection(-1);
        }
        if input.was_pressed(KeyCode::ArrowDown) || input.was_pressed(KeyCode::KeyS) {
            self.state.move_selection(1);
        }
        if input.was_pressed(KeyCode::ArrowLeft) || input.was_pressed(KeyCode::KeyA) {
            self.state.adjust(-1);
        }
        if input.was_pressed(KeyCode::ArrowRight) || input.was_pressed(KeyCode::KeyD) {
            self.state.adjust(1);
        }
        if input.was_pressed(KeyCode::KeyC) {
            self.handle(self.state.compile_selected());
        }
        if (input.was_pressed(KeyCode::Enter) || input.was_pressed(KeyCode::Space))
            && let Some(action) = self.state.activate()
        {
            self.handle(action);
        }

        let (win_w, win_h) = input.window_size();
        let (mx, my) = input.cursor_pos();
        self.state.mouse = (mx, my);
        if input.was_button_pressed(MouseButton::Left) && win_w > 0 && win_h > 0 {
            let lay = layout(&self.state, win_w as f32, win_h as f32);
            self.click(&lay);
        }

        // Guarda los ajustes en cuanto cambien (el archivo es mínimo).
        let snap = SavedSettings::from_state(&self.state);
        if self.saved.as_ref() != Some(&snap) {
            settings::save_settings(&self.root, &snap);
            self.saved = Some(snap);
        }
    }

    fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    ) {
        if let Some(panorama) = &self.panorama {
            panorama.render(&gpu.queue, encoder, color, depth);
        } else if let Some(cube) = &self.cube {
            cube.render(
                &gpu.queue,
                encoder,
                color,
                depth,
                &grisu_render::OrbitCamera::default(),
            );
        }
        self.batch.clear();
        draw::draw(
            &mut self.batch,
            &self.state,
            depth.width as f32,
            depth.height as f32,
        );
        if let Some(ui) = self.ui.as_mut() {
            let size = [depth.width, depth.height];
            ui.render(&gpu.device, &gpu.queue, encoder, color, size, &self.batch);
        }
    }

    fn wants_cursor_grab(&self) -> bool {
        false
    }

    fn should_exit(&self) -> bool {
        self.close_requested
    }
}
