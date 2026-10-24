//! Juego: lee la entrada, la manda al servidor a 30 Hz a través de la
//! [`Session`] y dibuja la réplica del mundo con la posición predicha.
//!
//! Controles: WASD para andar, espacio para saltar, ratón para mirar, clic
//! izquierdo para cavar, clic derecho para poner y 1-6 para elegir bloque.

use glam::Vec3;
use grisu_platform::{App, Input, KeyCode, MouseButton};
use grisu_render::{
    BoxRenderer, ColorBox, CrosshairRenderer, DepthTarget, FlyCamera, Gpu, TerrainStyle,
    VoxelRenderer,
};
use grisu_voxel::{FIXED_ONE, RayHit, raycast};
use hush_net::{Action, ClientState};
use hush_sim::{BlockKind, MoveInput, TICK_RATE};

use crate::session::Session;

/// Radianes por píxel de ratón.
const MOUSE_SENSITIVITY: f32 = 0.0025;
/// Alcance del raycast del cliente, algo menor que el del servidor para no
/// proponer acciones que se van a rechazar.
const AIM_REACH: f32 = 4.0;
/// Máximo de ticks por fotograma, para no entrar en espiral si va lento.
const MAX_TICKS_PER_FRAME: u32 = 5;
const TICK_SECONDS: f32 = 1.0 / TICK_RATE as f32;
/// Los demás jugadores se dibujan 100 ms en el pasado para interpolar.
const REMOTE_DELAY_TICKS: f32 = 3.0;
const FOG_DISTANCE: f32 = 56.0;
/// Bloques que se pueden poner con las teclas 1-6.
const PLACEABLE: [BlockKind; 6] = [
    BlockKind::Dirt,
    BlockKind::Rock,
    BlockKind::HardRock,
    BlockKind::Ore,
    BlockKind::Gold,
    BlockKind::Diamond,
];
const SLOT_KEYS: [KeyCode; 6] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
];
/// Colores de las cuadrillas: cobre y cobalto.
const CREW_COLORS: [[f32; 3]; 2] = [[0.80, 0.38, 0.14], [0.16, 0.34, 0.80]];

pub struct Game {
    session: Session,
    camera: FlyCamera,
    accumulator: f32,
    /// Ojos del jugador en el tick anterior y en el actual, para interpolar.
    prev_eye: Vec3,
    eye: Vec3,
    target: Option<RayHit>,
    place_block: BlockKind,
    queued: Option<Action>,
    last_state: ClientState,
    terrain: Option<VoxelRenderer>,
    boxes: Option<BoxRenderer>,
    crosshair: Option<CrosshairRenderer>,
    stats_timer: f32,
    frames: u32,
}

impl Game {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            camera: FlyCamera::default(),
            accumulator: 0.0,
            prev_eye: Vec3::ZERO,
            eye: Vec3::ZERO,
            target: None,
            place_block: BlockKind::Rock,
            queued: None,
            last_state: ClientState::Connecting,
            terrain: None,
            boxes: None,
            crosshair: None,
            stats_timer: 0.0,
            frames: 0,
        }
    }

    fn move_input(&self, input: &Input) -> MoveInput {
        let axis = |pos: KeyCode, neg: KeyCode| {
            f32::from(u8::from(input.is_held(pos))) - f32::from(u8::from(input.is_held(neg)))
        };
        let forward = axis(KeyCode::KeyW, KeyCode::KeyS);
        let right = axis(KeyCode::KeyD, KeyCode::KeyA);
        let wish = self.camera.flat_forward() * forward + self.camera.flat_right() * right;
        MoveInput::from_direction(wish.x, wish.z, input.is_held(KeyCode::Space))
    }

    fn handle_actions(&mut self, input: &Input) {
        for (key, block) in SLOT_KEYS.iter().zip(PLACEABLE) {
            if input.was_pressed(*key) {
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

    fn predicted_eye(&self) -> Vec3 {
        let eye = self.session.client().body().eye();
        Vec3::from_array(eye.map(|c| c as f32 / FIXED_ONE as f32))
    }

    fn report_state(&mut self) {
        let state = self.session.client().state();
        if *state != self.last_state {
            match state {
                ClientState::Loading { chunks, total } if chunks == total => {}
                ClientState::Loading { .. } => log::info!("recibiendo el mapa..."),
                ClientState::Playing => log::info!(
                    "en partida como jugador {:?}",
                    self.session.client().player()
                ),
                ClientState::Disconnected(reason) => log::warn!("desconectado: {reason}"),
                ClientState::Connecting => {}
            }
            self.last_state = state.clone();
        }
    }

    fn remote_boxes(&self) -> Vec<ColorBox> {
        let Some(tick) = self.session.client().last_server_tick() else {
            return Vec::new();
        };
        let at = tick as f32 + self.accumulator / TICK_SECONDS - REMOTE_DELAY_TICKS;
        let width = 2.0 * hush_sim::HALF_WIDTH as f32 / FIXED_ONE as f32;
        let height = hush_sim::HEIGHT as f32 / FIXED_ONE as f32;
        self.session
            .client()
            .remote_players(at)
            .into_iter()
            .map(|(id, [x, y, z])| ColorBox {
                min: Vec3::new(x - width / 2.0, y, z - width / 2.0),
                size: Vec3::new(width, height, width),
                color: CREW_COLORS[usize::from(id) % CREW_COLORS.len()],
            })
            .collect()
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
        match VoxelRenderer::new(
            &gpu.device,
            &gpu.queue,
            surface_format,
            &hush_art::block_textures(),
        ) {
            Ok(mut terrain) => {
                terrain.enable_lighting(hush_art::emission_table());
                self.terrain = Some(terrain);
            }
            Err(err) => log::error!("no se pudo crear el renderizador del terreno: {err:#}"),
        }
        self.boxes = Some(BoxRenderer::new(&gpu.device, surface_format));
        self.crosshair = Some(CrosshairRenderer::new(&gpu.device, surface_format));
    }

    fn update(&mut self, dt: f32, input: &Input) {
        let (dx, dy) = input.mouse_delta();
        self.camera
            .rotate(-dx * MOUSE_SENSITIVITY, -dy * MOUSE_SENSITIVITY);
        self.session.poll();
        self.report_state();
        self.handle_actions(input);
        let movement = self.move_input(input);
        self.step_ticks(dt, movement);
        self.session.flush();

        let alpha = self.accumulator / TICK_SECONDS;
        self.camera.position = self.prev_eye.lerp(self.eye, alpha.clamp(0.0, 1.0));
        self.target = self.session.client().world().and_then(|world| {
            raycast(
                world,
                self.camera.position.to_array(),
                self.camera.forward().to_array(),
                AIM_REACH,
            )
        });
        self.log_stats(dt);
    }

    fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    ) {
        let Some(terrain) = self.terrain.as_mut() else {
            return;
        };
        let playing = *self.session.client().state() == ClientState::Playing;
        if playing && let Some(world) = self.session.client_mut().world_mut() {
            terrain.update(&gpu.device, world);
        }
        let aspect = depth.width as f32 / depth.height as f32;
        let view = self.camera.view(aspect);
        let style = TerrainStyle {
            fog_distance: FOG_DISTANCE,
            selected: self.target.map(|hit| hit.block),
            lamp_radius: 14.0,
        };
        terrain.render(&gpu.queue, encoder, color, depth, &view, &style);
        let remotes = self.remote_boxes();
        if let Some(boxes) = self.boxes.as_mut() {
            boxes.render(
                &gpu.device,
                &gpu.queue,
                encoder,
                color,
                depth,
                &view,
                FOG_DISTANCE,
                &remotes,
            );
        }
        if let Some(crosshair) = &self.crosshair {
            crosshair.render(&gpu.queue, encoder, color, depth.width, depth.height);
        }
    }
}
