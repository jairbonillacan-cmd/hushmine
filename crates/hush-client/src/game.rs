//! Juego: lee la entrada configurable, gestiona el menú (lanzador, pausa con
//! Escape y controles), alterna entre primera y tercera persona, envía las
//! órdenes al servidor a 30 Hz a través de la [`Session`] y dibuja la réplica
//! del mundo con la posición predicha y la interfaz encima.

use glam::{DVec3, Vec3};
use grisu_platform::{App, Input, KeyCode, MouseButton};
use grisu_render::{
    BoxRenderer, CameraMode, ColorBox, CrosshairRenderer, DepthTarget, FlyCamera, Gpu,
    TerrainStyle, UiBatch, UiRenderer, View, VoxelRenderer,
};
use grisu_voxel::{FIXED_ONE, LightMap, RayHit, raycast};
use hush_net::{Action, ClientState};
use hush_sim::{BlockKind, EYE_HEIGHT, HALF_WIDTH, HEIGHT, MoveInput, TICK_RATE};

use crate::session::Session;
use hush_client::controls::{Controls, SprintMode};
use hush_client::hud::draw_ui;
use hush_client::menu::{MenuCommand, MenuState};

/// Alcance del raycast del cliente, algo menor que el del servidor para no
/// proponer acciones que se van a rechazar.
const AIM_REACH: f64 = 4.0;
/// Máximo de ticks por fotograma, para no entrar en espiral si va lento.
const MAX_TICKS_PER_FRAME: u32 = 5;
const TICK_SECONDS: f32 = 1.0 / TICK_RATE as f32;
/// Los demás jugadores se dibujan 100 ms en el pasado para interpolar.
const REMOTE_DELAY_TICKS: f64 = 3.0;
const FOG_DISTANCE: f32 = 56.0;
/// Margen entre la cámara en tercera persona y la pared que la recorta.
const THIRD_PERSON_WALL_MARGIN: f64 = 0.35;
/// Ensanchamiento del campo de visión al correr.
const SPRINT_FOV_BOOST_DEG: f32 = 5.0;
/// Bloques que se pueden poner con las teclas 1-6.
const SLOT_BINDINGS: [(KeyCode, BlockKind); 6] = [
    (KeyCode::Digit1, BlockKind::Dirt),
    (KeyCode::Digit2, BlockKind::Rock),
    (KeyCode::Digit3, BlockKind::HardRock),
    (KeyCode::Digit4, BlockKind::Ore),
    (KeyCode::Digit5, BlockKind::Gold),
    (KeyCode::Digit6, BlockKind::Diamond),
];
/// Colores de las cuadrillas: cobre y cobalto.
const CREW_COLORS: [[f32; 3]; 2] = [[0.80, 0.38, 0.14], [0.16, 0.34, 0.80]];
/// Luz del casco del minero propio en tercera persona (ámbar de sodio).
const HELMET_LAMP_COLOR: [f32; 3] = [0.98, 0.88, 0.45];

/// Cómo arranca el juego (lo decide `main` a partir de los argumentos).
pub struct GameConfig {
    pub seed: u64,
    pub bots: u32,
    pub name: String,
    pub start_in_launcher: bool,
    pub remote_session: bool,
}

pub struct Game {
    session: Session,
    player_name: String,
    controls: Controls,
    menu: MenuState,
    camera: FlyCamera,
    camera_mode: CameraMode,
    /// Carrera activada con pulsación (modo `Toggle`).
    sprint_toggled: bool,
    /// ¿Corre el minero en este tick? (para el HUD y el campo de visión).
    sprinting: bool,
    exit_requested: bool,
    accumulator: f32,
    /// Segundos desde el arranque, para animar el terreno (lava).
    elapsed: f32,
    /// Ojos del jugador en el tick anterior y en el actual, para interpolar.
    prev_eye: DVec3,
    eye: DVec3,
    /// Luz del mundo; se recalcula cuando hay chunks sucios.
    light: Option<LightMap>,
    target: Option<RayHit>,
    place_block: BlockKind,
    queued: Option<Action>,
    last_state: ClientState,
    terrain: Option<VoxelRenderer>,
    boxes: Option<BoxRenderer>,
    crosshair: Option<CrosshairRenderer>,
    ui: Option<UiRenderer>,
    ui_batch: UiBatch,
    stats_timer: f32,
    frames: u32,
}

impl Game {
    pub fn new(session: Session, cfg: GameConfig) -> Self {
        Self {
            session,
            player_name: cfg.name,
            controls: Controls::load_or_default(),
            menu: MenuState::new(
                cfg.start_in_launcher,
                cfg.seed,
                cfg.bots,
                cfg.remote_session,
            ),
            camera: FlyCamera::default(),
            camera_mode: CameraMode::FirstPerson,
            sprint_toggled: false,
            sprinting: false,
            exit_requested: false,
            accumulator: 0.0,
            elapsed: 0.0,
            prev_eye: DVec3::ZERO,
            eye: DVec3::ZERO,
            light: None,
            target: None,
            place_block: BlockKind::Rock,
            queued: None,
            last_state: ClientState::Connecting,
            terrain: None,
            boxes: None,
            crosshair: None,
            ui: None,
            ui_batch: UiBatch::new(),
            stats_timer: 0.0,
            frames: 0,
        }
    }

    /// Empieza otra partida local desde el menú. El mundo es nuevo, así que
    /// también se olvidan la luz y las mallas del anterior.
    fn restart_local(&mut self, seed: u64, bots: u32) {
        match Session::local(seed, bots, &self.player_name) {
            Ok(session) => {
                log::info!("nueva partida local: semilla {seed}, {bots} bots");
                self.session = session;
                self.last_state = ClientState::Connecting;
                self.accumulator = 0.0;
                self.queued = None;
                self.target = None;
                self.light = None;
                if let Some(terrain) = self.terrain.as_mut() {
                    terrain.clear();
                }
            }
            Err(err) => log::error!("no se pudo empezar la partida local: {err:#}"),
        }
    }

    fn move_input(&mut self, input: &Input) -> MoveInput {
        let axis = |pos: KeyCode, neg: KeyCode| {
            f32::from(u8::from(input.is_held(pos))) - f32::from(u8::from(input.is_held(neg)))
        };
        let forward = axis(self.controls.forward, self.controls.back);
        let right = axis(self.controls.right, self.controls.left);
        let moving = forward != 0.0 || right != 0.0;
        if self.controls.sprint_mode == SprintMode::Toggle
            && input.was_pressed(self.controls.sprint)
        {
            self.sprint_toggled = !self.sprint_toggled;
        }
        // Al pararse se apaga la carrera, como en la mayoría de juegos.
        if !moving || self.controls.sprint_mode == SprintMode::Hold {
            self.sprint_toggled = false;
        }
        self.sprinting = moving && (self.sprint_toggled || input.is_held(self.controls.sprint));
        let wish = self.camera.flat_forward() * forward + self.camera.flat_right() * right;
        MoveInput::from_direction(wish.x, wish.z, input.is_held(self.controls.jump))
            .with_sprint(self.sprinting)
    }

    fn handle_actions(&mut self, input: &Input) {
        if input.was_pressed(self.controls.camera) || input.was_pressed(KeyCode::F5) {
            self.camera_mode = self.camera_mode.next();
            log::info!("cámara: {}", self.camera_mode.label());
        }
        for (key, block) in SLOT_BINDINGS {
            if input.was_pressed(key) {
                self.place_block = block;
                log::info!("bloque seleccionado: {}", block.name());
            }
        }
        let Some(hit) = self.target else { return };
        if input.was_button_pressed(MouseButton::Left) {
            self.queued = Some(Action::Dig(hit.block));
        }
        if input.was_button_pressed(MouseButton::Right) {
            let cell = std::array::from_fn(|i| hit.block[i] + hit.normal[i]);
            self.queued = Some(Action::Place(cell, self.place_block));
        }
    }

    fn step_ticks(&mut self, dt: f32, movement: MoveInput) {
        self.accumulator += dt;
        let mut ticks = 0;
        while self.accumulator >= TICK_SECONDS && ticks < MAX_TICKS_PER_FRAME {
            self.session.tick(movement, self.queued.take());
            self.prev_eye = self.eye;
            self.eye = self.predicted_eye();
            self.accumulator -= TICK_SECONDS;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            self.accumulator = 0.0;
        }
    }

    fn predicted_eye(&self) -> DVec3 {
        let eye = self.session.client().body().eye();
        DVec3::from_array(eye.map(|c| c as f64 / FIXED_ONE as f64))
    }

    fn report_state(&mut self) {
        let state = self.session.client().state();
        if *state != self.last_state {
            match state {
                ClientState::Loading { chunks, total } if chunks == total => {}
                ClientState::Loading { .. } => log::info!("recibiendo el mapa..."),
                ClientState::Playing => {
                    // Sin esto la cámara viajaría desde el origen hasta el
                    // minero durante el primer tick.
                    let eye = self.predicted_eye();
                    self.prev_eye = eye;
                    self.eye = eye;
                    log::info!(
                        "en partida como jugador {:?}",
                        self.session.client().player()
                    );
                }
                ClientState::Disconnected(reason) => log::warn!("desconectado: {reason}"),
                ClientState::Connecting => {}
            }
            self.last_state = state.clone();
        }
    }

    /// Vista según el modo de cámara. En tercera persona, el brazo se
    /// recorta contra el terreno para no atravesar paredes.
    fn active_view(&self, aspect: f32) -> View {
        if !self.camera_mode.is_third_person() {
            return self.camera.view(aspect);
        }
        let front = self.camera_mode == CameraMode::ThirdPersonFront;
        let desired = f64::from(self.controls.third_person_dist);
        let dir = self.camera.third_person_arm_dir(front).as_dvec3();
        let clipped = self
            .session
            .client()
            .world()
            .and_then(|w| raycast(w, self.camera.position.to_array(), dir.to_array(), desired))
            .map_or(desired, |hit| {
                (hit.distance - THIRD_PERSON_WALL_MARGIN).max(0.0)
            });
        self.camera.third_person_view(aspect, clipped as f32, front)
    }

    /// Cajas de los demás jugadores y, en tercera persona, del propio minero
    /// con su lámpara.
    fn entity_boxes(&self) -> Vec<ColorBox> {
        let Some(tick) = self.session.client().last_server_tick() else {
            return Vec::new();
        };
        let at = tick as f64 + self.accumulator as f64 / TICK_SECONDS as f64 - REMOTE_DELAY_TICKS;
        let width = 2.0 * HALF_WIDTH as f64 / FIXED_ONE as f64;
        let height = HEIGHT as f64 / FIXED_ONE as f64;
        let size = Vec3::new(width as f32, height as f32, width as f32);
        let crew = |id: u16| CREW_COLORS[usize::from(id) % CREW_COLORS.len()];
        let mut list: Vec<ColorBox> = self
            .session
            .client()
            .remote_players(at)
            .into_iter()
            .map(|(id, [x, y, z])| ColorBox {
                min: DVec3::new(x - width / 2.0, y, z - width / 2.0),
                size,
                color: crew(id),
            })
            .collect();
        if self.camera_mode.is_third_person() {
            let feet = self.camera.position - DVec3::Y * (EYE_HEIGHT as f64 / FIXED_ONE as f64);
            list.push(ColorBox {
                min: DVec3::new(feet.x - width / 2.0, feet.y, feet.z - width / 2.0),
                size,
                color: crew(self.session.client().player().unwrap_or(0)),
            });
            let lamp =
                self.camera.position + self.camera.flat_forward().as_dvec3() * (width * 0.45);
            list.push(ColorBox {
                min: lamp - DVec3::splat(0.12),
                size: Vec3::splat(0.24),
                color: HELMET_LAMP_COLOR,
            });
        }
        list
    }

    fn log_stats(&mut self, dt: f32) {
        self.frames += 1;
        self.stats_timer += dt;
        if self.stats_timer >= 5.0 {
            let fps = self.frames as f32 / self.stats_timer;
            let client = self.session.client();
            let net = client.connection().stats();
            let p = client.stats();
            log::info!(
                "{fps:.0} FPS, rtt {:.0} ms, {} correcciones de {} instantáneas",
                net.rtt_ms,
                p.corrections,
                p.snapshots
            );
            self.frames = 0;
            self.stats_timer = 0.0;
        }
    }
}

impl App for Game {
    fn init(&mut self, gpu: &Gpu, surface_format: wgpu::TextureFormat) {
        let terrain = hush_art::block_textures()
            .map_err(anyhow::Error::from)
            .and_then(|textures| {
                VoxelRenderer::new(&gpu.device, &gpu.queue, surface_format, &textures)
            });
        match terrain {
            Ok(terrain) => self.terrain = Some(terrain),
            Err(err) => {
                log::error!("no se pudieron cargar las texturas o crear el renderizador: {err:#}")
            }
        }
        self.boxes = Some(BoxRenderer::new(&gpu.device, surface_format));
        self.crosshair = Some(CrosshairRenderer::new(&gpu.device, surface_format));
        self.ui = Some(UiRenderer::new(&gpu.device, surface_format));
    }

    fn update(&mut self, dt: f32, input: &Input) {
        self.elapsed += dt;
        match self
            .menu
            .update(input, &mut self.controls, &mut self.camera_mode)
        {
            MenuCommand::None => {}
            MenuCommand::StartLocal { seed, bots } => self.restart_local(seed, bots),
            MenuCommand::Exit => {
                self.exit_requested = true;
                return;
            }
        }

        self.session.poll();
        self.report_state();

        // Con el menú abierto el minero se queda quieto, pero la partida sigue.
        let movement = if self.menu.is_open() {
            self.sprinting = false;
            MoveInput::default()
        } else {
            let (dx, dy) = input.mouse_delta();
            let sens = self.controls.mouse_sensitivity();
            self.camera.rotate(-dx * sens, -dy * sens);
            self.handle_actions(input);
            self.move_input(input)
        };

        let fov_boost = f32::from(u8::from(self.sprinting)) * SPRINT_FOV_BOOST_DEG.to_radians();
        self.camera.fov_y = self.controls.fov_radians() + fov_boost;
        self.step_ticks(dt, movement);
        self.session.flush();

        let alpha = (self.accumulator / TICK_SECONDS).clamp(0.0, 1.0);
        self.camera.position = self.prev_eye.lerp(self.eye, f64::from(alpha));
        self.target = if self.menu.is_open() {
            None
        } else {
            self.session.client().world().and_then(|world| {
                raycast(
                    world,
                    self.camera.position.to_array(),
                    self.camera.forward().to_array().map(f64::from),
                    AIM_REACH,
                )
            })
        };
        self.log_stats(dt);
    }

    fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    ) {
        let view = self.active_view(depth.width as f32 / depth.height as f32);
        let Some(terrain) = self.terrain.as_mut() else {
            return;
        };
        let playing = *self.session.client().state() == ClientState::Playing;
        if playing && let Some(world) = self.session.client_mut().world_mut() {
            // La luz vive en el juego: se crea con el mapa y se recalcula
            // cuando la simulación deja chunks sucios.
            let light = self
                .light
                .get_or_insert_with(|| LightMap::new(world, hush_art::emission_table()));
            if world.has_dirty() {
                light.update(world);
            }
            terrain.update(&gpu.device, world, Some(light));
        }
        let style = TerrainStyle {
            fog_distance: FOG_DISTANCE,
            selected: self.target.map(|hit| hit.block),
            lamp_radius: 14.0,
            time_seconds: self.elapsed,
            ..TerrainStyle::default()
        };
        terrain.render(&gpu.queue, encoder, color, depth, &view, &style);
        let entities = self.entity_boxes();
        if let Some(boxes) = self.boxes.as_mut() {
            boxes.render(
                &gpu.device,
                &gpu.queue,
                encoder,
                color,
                depth,
                &view,
                FOG_DISTANCE,
                &entities,
            );
        }
        if !self.menu.is_open()
            && let Some(crosshair) = &self.crosshair
        {
            crosshair.render(&gpu.queue, encoder, color, depth.width, depth.height);
        }
        if let Some(ui) = self.ui.as_mut() {
            let size = [depth.width, depth.height];
            self.ui_batch.clear();
            draw_ui(
                &self.menu,
                &mut self.ui_batch,
                size,
                &self.controls,
                self.camera_mode,
                self.sprinting,
                self.place_block,
            );
            ui.render(
                &gpu.device,
                &gpu.queue,
                encoder,
                color,
                size,
                &self.ui_batch,
            );
        }
    }

    fn wants_cursor_grab(&self) -> bool {
        !self.menu.is_open()
    }

    fn should_exit(&self) -> bool {
        self.exit_requested
    }
}
