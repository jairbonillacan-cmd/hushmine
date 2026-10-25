//! Modo historia del cliente: expedición local a 20 Hz con ciclo diurno,
//! fisiología, inventario y telemetría F3.
//!
//! Controles: clic captura el ratón; WASD anda; MAYÚS se agacha; CTRL
//! corre; ESPACIO salta; clic izquierdo pica (mantener); clic derecho pone
//! el bloque empuñado; E interactúa (banco, horno, arcón, válvulas,
//! engranajes, motor); F fabrica (menú compacto); R come; 1-9 elige casilla
//! de la barra; F3 abre la telemetría; ESC suelta el ratón o cierra.

use glam::{DVec3, Vec3};
use grisu_platform::{App, Input, KeyCode, MouseButton};
use grisu_render::{
    BoxRenderer, CameraMode, ColorBox, CrosshairRenderer, DepthTarget, FlyCamera, Gpu, MeshStats,
    TerrainStyle, UiBatch, UiRenderer, VoxelRenderer,
};
use grisu_voxel::{DualLightMap, FIXED_ONE, RayHit, raycast};
use hush_sim::MoveInput;
use hush_sim::story::{Expedition, STORY_TICK_RATE, StoryConfig, StoryOrder};

use crate::story_hud::{self, FpsTracker};
use crate::story_orders::{CraftMenu, move_input_from_camera};

const AIM_REACH: f64 = 5.0;
const MAX_TICKS_PER_FRAME: u32 = 5;
const TICK_SECONDS: f32 = 1.0 / STORY_TICK_RATE as f32;
const FOG_DISTANCE: f32 = 56.0;
const MOB_COLORS: [[f32; 3]; 4] = [
    [0.75, 0.20, 0.15],
    [0.85, 0.55, 0.15],
    [0.80, 0.80, 0.70],
    [0.45, 0.60, 0.20],
];

pub struct StoryApp {
    expedition: Expedition,
    camera: FlyCamera,
    camera_mode: CameraMode,
    exit_requested: bool,
    paused: bool,
    f3_open: bool,
    accumulator: f32,
    elapsed: f32,
    eye: DVec3,
    prev_eye: DVec3,
    target: Option<RayHit>,
    held_to_dig: bool,
    craft: CraftMenu,
    fps: FpsTracker,
    banner: Option<(&'static str, f32)>,
    terrain: Option<VoxelRenderer>,
    /// Luz doble (cielo y emisores) del mundo de la expedición; la calcula el
    /// juego y el renderer solo la empaqueta en las caras.
    light: Option<DualLightMap>,
    boxes: Option<BoxRenderer>,
    crosshair: Option<CrosshairRenderer>,
    ui: Option<UiRenderer>,
    ui_batch: UiBatch,
    last_mesh: MeshStats,
}

impl StoryApp {
    pub fn new(config: StoryConfig) -> anyhow::Result<Self> {
        Ok(Self {
            expedition: Expedition::new(config)?,
            camera: FlyCamera::default(),
            camera_mode: CameraMode::FirstPerson,
            exit_requested: false,
            paused: false,
            f3_open: false,
            accumulator: 0.0,
            elapsed: 0.0,
            eye: DVec3::ZERO,
            prev_eye: DVec3::ZERO,
            target: None,
            held_to_dig: false,
            craft: CraftMenu::default(),
            fps: FpsTracker::default(),
            banner: None,
            terrain: None,
            light: None,
            boxes: None,
            crosshair: None,
            ui: None,
            ui_batch: UiBatch::new(),
            last_mesh: MeshStats::default(),
        })
    }

    fn eye_from_body(&self) -> DVec3 {
        fixed_to_world(self.expedition.body().eye())
    }

    fn tick_orders(&mut self, input: &Input) -> Vec<StoryOrder> {
        let mut orders = Vec::new();
        if self.paused || self.craft.is_open() {
            orders.push(StoryOrder::Move(MoveInput::default()));
            orders.push(StoryOrder::Mine(None));
            return orders;
        }
        let movement =
            move_input_from_camera(input, self.camera.flat_forward(), self.camera.flat_right());
        orders.push(StoryOrder::Move(movement));
        orders.push(StoryOrder::Sneak(input.is_held(KeyCode::ShiftLeft)));
        // Picar: mantener el clic izquierdo sobre el bloque apuntado.
        let mine = if self.held_to_dig {
            self.target.map(|hit| hit.block)
        } else {
            None
        };
        orders.push(StoryOrder::Mine(mine));
        // Colocar e interactuar.
        if input.was_button_pressed(MouseButton::Right)
            && let Some(hit) = self.target
        {
            let cell = std::array::from_fn(|i| hit.block[i] + hit.normal[i]);
            orders.push(StoryOrder::Place(cell));
        }
        if input.was_pressed(KeyCode::KeyE)
            && let Some(hit) = self.target
        {
            orders.push(StoryOrder::Interact(hit.block));
        }
        if input.was_pressed(KeyCode::KeyR) {
            orders.push(StoryOrder::Eat);
        }
        for (idx, key) in [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
        ]
        .into_iter()
        .enumerate()
        {
            if input.was_pressed(key) {
                orders.push(StoryOrder::Select(idx as u8));
            }
        }
        // Ataque cuerpo a cuerpo con la herramienta empuñada.
        if input.was_button_pressed(MouseButton::Left)
            && !self.held_to_dig
            && let Some(hit) = self.target
        {
            orders.push(StoryOrder::Attack(hit.block));
        }
        orders
    }

    fn step_simulation(&mut self, dt: f32, orders: Vec<StoryOrder>) {
        self.accumulator += dt;
        let mut ticks = 0;
        while self.accumulator >= TICK_SECONDS && ticks < MAX_TICKS_PER_FRAME {
            self.expedition.step(&orders);
            self.prev_eye = self.eye;
            self.eye = self.eye_from_body();
            self.accumulator -= TICK_SECONDS;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            self.accumulator = 0.0;
        }
    }

    fn consume_events(&mut self) {
        use hush_sim::story::StoryEventKind;
        for event in self.expedition.take_events() {
            match event.kind {
                StoryEventKind::MilestoneReached(m) => {
                    let milestone = hush_sim::story::Milestone::ALL
                        .get(usize::from(m.saturating_sub(1)))
                        .copied();
                    if let Some(milestone) = milestone {
                        self.banner = Some((milestone.label(), 4.0));
                    }
                }
                StoryEventKind::Died => self.banner = Some(("HAS DESFALLECIDO", 6.0)),
                StoryEventKind::Victory => self.banner = Some(("VICTORIA: ASCENSO FINAL", 10.0)),
                StoryEventKind::SiegeStarted => self.banner = Some(("ASEDIO: 5 MINUTOS", 5.0)),
                StoryEventKind::ToolBroke(_) => self.banner = Some(("HERRAMIENTA ROTA", 2.0)),
                _ => {}
            }
        }
    }

    fn mob_boxes(&self) -> Vec<ColorBox> {
        self.expedition
            .mobs()
            .list
            .iter()
            .map(|mob| {
                let color = MOB_COLORS[mob.kind as usize % MOB_COLORS.len()];
                ColorBox {
                    min: DVec3::new(
                        f64::from(mob.cell[0]),
                        f64::from(mob.cell[1]),
                        f64::from(mob.cell[2]),
                    ),
                    size: Vec3::new(0.7, 1.6, 0.7),
                    color,
                }
            })
            .collect()
    }

    fn fog_color(&self) -> [f32; 3] {
        let factor = self.expedition.clock().sky_factor_q8() as f32 / 256.0;
        // De noche la niebla se oscurece hacia el azul profundo.
        [
            0.04 + 0.07 * factor,
            0.05 + 0.09 * factor,
            0.08 + 0.11 * factor,
        ]
    }
}

impl App for StoryApp {
    fn init(&mut self, gpu: &Gpu, surface_format: wgpu::TextureFormat) {
        let spawn = self.expedition.body().feet;
        self.eye = fixed_to_world(spawn);
        self.prev_eye = self.eye;
        self.camera.position = self.eye;

        let terrain = hush_art::all_block_textures()
            .map_err(anyhow::Error::from)
            .and_then(|t| VoxelRenderer::new(&gpu.device, &gpu.queue, surface_format, &t));
        match terrain {
            Ok(terrain) => self.terrain = Some(terrain),
            Err(err) => log::error!("error al crear el renderizador de terreno: {err:#}"),
        }
        self.boxes = Some(BoxRenderer::new(&gpu.device, surface_format));
        self.crosshair = Some(CrosshairRenderer::new(&gpu.device, surface_format));
        self.ui = Some(UiRenderer::new(&gpu.device, surface_format));
    }

    fn update(&mut self, dt: f32, input: &Input) {
        self.elapsed += dt;
        self.fps.frame(dt);
        if let Some((_, ttl)) = &mut self.banner {
            *ttl -= dt;
            if *ttl <= 0.0 {
                self.banner = None;
            }
        }

        if input.was_pressed(KeyCode::Escape) {
            if self.paused || self.craft.is_open() {
                self.craft.close();
                self.paused = false;
            } else {
                self.paused = true;
            }
        }
        if input.was_pressed(KeyCode::F3) {
            self.f3_open = !self.f3_open;
        }
        if input.was_pressed(KeyCode::F5) {
            self.camera_mode = self.camera_mode.next();
        }
        self.craft.update(input, &mut self.expedition);

        let ui_capture = self.paused || self.craft.is_open();
        if !ui_capture {
            let (dx, dy) = input.mouse_delta();
            self.camera.rotate(-dx * 0.0022, -dy * 0.0022);
            self.held_to_dig = input.is_button_held(MouseButton::Left);
        } else {
            self.held_to_dig = false;
        }

        let orders = self.tick_orders(input);
        self.step_simulation(dt, orders);
        self.consume_events();

        let alpha = (self.accumulator / TICK_SECONDS).clamp(0.0, 1.0);
        self.camera.position = self.prev_eye.lerp(self.eye, f64::from(alpha));
        if ui_capture {
            self.target = None;
        } else {
            self.target = raycast(
                self.expedition.world(),
                self.camera.position.to_array(),
                self.camera.forward().as_dvec3().to_array(),
                AIM_REACH,
            );
        }
    }

    fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    ) {
        let view = match self.camera_mode {
            CameraMode::FirstPerson => self.camera.view(depth.width as f32 / depth.height as f32),
            CameraMode::ThirdPersonBack | CameraMode::ThirdPersonFront => {
                self.camera.third_person_view(
                    depth.width as f32 / depth.height as f32,
                    3.5,
                    self.camera_mode == CameraMode::ThirdPersonFront,
                )
            }
        };
        let style = TerrainStyle {
            fog_distance: FOG_DISTANCE,
            fog_color: self.fog_color(),
            selected: self.target.map(|hit| hit.block),
            lamp_radius: 0.0,
            time_seconds: self.elapsed,
            daylight: self.expedition.clock().sky_factor_q8() as f32 / 256.0,
        };
        let mobs = self.mob_boxes();
        let Some(terrain) = self.terrain.as_mut() else {
            return;
        };
        let world = self.expedition.world_mut();
        let light = match self.light.as_mut() {
            Some(light) => {
                if world.has_dirty() {
                    light.update(world);
                }
                light
            }
            None => {
                world.mark_all_dirty();
                self.light
                    .insert(DualLightMap::new(world, hush_art::emission_table()))
            }
        };
        self.last_mesh = terrain.update_dual(&gpu.device, world, Some(light));
        terrain.render(&gpu.queue, encoder, color, depth, &view, &style);
        if let Some(boxes) = self.boxes.as_mut() {
            boxes.render(
                &gpu.device,
                &gpu.queue,
                encoder,
                color,
                depth,
                &view,
                FOG_DISTANCE,
                &mobs,
            );
        }
        if !self.paused
            && !self.craft.is_open()
            && let Some(crosshair) = &self.crosshair
        {
            crosshair.render(&gpu.queue, encoder, color, depth.width, depth.height);
        }
        let size = [depth.width, depth.height];
        // El HUD se dibuja desde una vista con datos propios: así el lote y
        // el renderizador no compiten por `self`.
        let view = story_hud::HudView::of(self);
        let mut batch = std::mem::take(&mut self.ui_batch);
        batch.clear();
        story_hud::draw(&mut batch, size, &view);
        if let Some(ui) = self.ui.as_mut() {
            ui.render(&gpu.device, &gpu.queue, encoder, color, size, &batch);
        }
        self.ui_batch = batch;
    }

    fn wants_cursor_grab(&self) -> bool {
        !self.paused && !self.craft.is_open()
    }

    fn should_exit(&self) -> bool {
        self.exit_requested
    }
}

/// Posición en punto fijo (Q47.16) a coordenadas del mundo.
fn fixed_to_world(p: [i64; 3]) -> DVec3 {
    DVec3::from_array(p.map(|c| c as f64 / FIXED_ONE as f64))
}

/// Constructor desde la configuración para `main.rs`.
pub fn story_app(config: StoryConfig) -> anyhow::Result<StoryApp> {
    StoryApp::new(config)
}

// Accesores que usa el HUD (`story_hud`).
impl StoryApp {
    pub fn expedition(&self) -> &Expedition {
        &self.expedition
    }

    pub fn craft_menu(&self) -> &CraftMenu {
        &self.craft
    }

    pub fn craft_rows(&self) -> Vec<(String, bool)> {
        self.craft.rows(&self.expedition)
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn f3_open(&self) -> bool {
        self.f3_open
    }

    pub fn banner(&self) -> Option<(&'static str, f32)> {
        self.banner
    }

    /// (fps, 1% bajo, frametime ms).
    pub fn fps_stats(&self) -> (f32, f32, f32) {
        self.fps.stats()
    }

    /// Rumbo cardinal dominante desde la guiñada de la cámara.
    pub fn facing(&self) -> &'static str {
        // yaw 0 mira a -Z (norte); crece hacia -X (oeste).
        let deg = self.camera.yaw.to_degrees().rem_euclid(360.0);
        match deg {
            d if d < 45.0 || d >= 315.0 => "NORTE (-Z)",
            d if d < 135.0 => "OESTE (-X)",
            d if d < 225.0 => "SUR (+Z)",
            _ => "ESTE (+X)",
        }
    }

    pub fn pitch_deg(&self) -> f32 {
        self.camera.pitch.to_degrees()
    }

    pub fn yaw_deg(&self) -> f32 {
        self.camera.yaw.to_degrees().rem_euclid(360.0)
    }

    pub fn mesh_stats(&self) -> MeshStats {
        self.last_mesh
    }

    pub fn chunk_stats(&self) -> (usize, usize) {
        let total = self.expedition.world().chunk_count();
        self.terrain
            .as_ref()
            .map_or((0, total), |t| (t.meshed_chunks(), total))
    }

    pub fn target(&self) -> Option<RayHit> {
        self.target
    }
}
