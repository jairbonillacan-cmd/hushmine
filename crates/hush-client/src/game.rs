//! Partida local de un jugador: la simulación corre a 30 Hz dentro del
//! cliente y la cámara interpola entre ticks.
//!
//! Controles: WASD para andar, espacio para saltar, ratón para mirar, clic
//! izquierdo para cavar, clic derecho para poner y 1-6 para elegir bloque.

use glam::Vec3;
use grisu_platform::{App, Input, KeyCode, MouseButton};
use grisu_render::{CrosshairRenderer, DepthTarget, FlyCamera, Gpu, TerrainStyle, VoxelRenderer};
use grisu_voxel::{FIXED_ONE, RayHit, raycast};
use hush_sim::{Block, Command, MatchConfig, MoveInput, Simulation, TICK_RATE};

/// Radianes por píxel de ratón.
const MOUSE_SENSITIVITY: f32 = 0.0025;
/// Alcance del raycast del cliente, algo menor que el del servidor para no
/// proponer acciones que se van a rechazar.
const AIM_REACH: f32 = 4.0;
/// Máximo de ticks por fotograma, para no entrar en espiral si va lento.
const MAX_TICKS_PER_FRAME: u32 = 5;
const TICK_SECONDS: f32 = 1.0 / TICK_RATE as f32;
/// Bloques que se pueden poner con las teclas 1-6.
const PLACEABLE: [Block; 6] = [
    Block::Dirt,
    Block::Rock,
    Block::HardRock,
    Block::Ore,
    Block::Gold,
    Block::Diamond,
];
const SLOT_KEYS: [KeyCode; 6] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
];

pub struct Game {
    sim: Simulation,
    player: usize,
    camera: FlyCamera,
    accumulator: f32,
    /// Ojos del jugador en el tick anterior y en el actual, para interpolar.
    prev_eye: Vec3,
    eye: Vec3,
    target: Option<RayHit>,
    place_block: Block,
    queued: Vec<Command>,
    terrain: Option<VoxelRenderer>,
    crosshair: Option<CrosshairRenderer>,
    stats_timer: f32,
    frames: u32,
}

impl Game {
    pub fn new(seed: u64) -> anyhow::Result<Self> {
        let config = MatchConfig {
            seed,
            max_ticks: u32::MAX,
            ..MatchConfig::default()
        };
        let mut sim = Simulation::new(config)?;
        let player = sim.spawn_player();
        let eye = eye_of(&sim, player);
        log::info!("mapa con semilla {seed}; jugador en {eye:?}");
        Ok(Self {
            sim,
            player,
            camera: FlyCamera::default(),
            accumulator: 0.0,
            prev_eye: eye,
            eye,
            target: None,
            place_block: Block::Rock,
            queued: Vec::new(),
            terrain: None,
            crosshair: None,
            stats_timer: 0.0,
            frames: 0,
        })
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
        let player = self.player;
        if input.was_button_pressed(MouseButton::Left) {
            self.queued.push(Command::Dig {
                player,
                cell: hit.block,
            });
        }
        if input.was_button_pressed(MouseButton::Right) {
            let cell = std::array::from_fn(|i| hit.block[i] + hit.normal[i]);
            self.queued.push(Command::Place {
                player,
                cell,
                block: self.place_block,
            });
        }
    }

    fn step_simulation(&mut self, dt: f32, input: MoveInput) {
        self.accumulator += dt;
        let mut ticks = 0;
        while self.accumulator >= TICK_SECONDS && ticks < MAX_TICKS_PER_FRAME {
            self.sim.submit(Command::Move {
                player: self.player,
                input,
            });
            for command in self.queued.drain(..) {
                self.sim.submit(command);
            }
            self.sim.step();
            self.prev_eye = self.eye;
            self.eye = eye_of(&self.sim, self.player);
            self.accumulator -= TICK_SECONDS;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            self.accumulator = 0.0;
        }
    }

    fn log_stats(&mut self, dt: f32) {
        self.frames += 1;
        self.stats_timer += dt;
        if self.stats_timer >= 5.0 {
            let fps = self.frames as f32 / self.stats_timer;
            let faces = self.terrain.as_ref().map_or(0, VoxelRenderer::face_count);
            log::info!("{fps:.0} FPS, {faces} caras, tick {}", self.sim.tick());
            self.frames = 0;
            self.stats_timer = 0.0;
        }
    }
}

/// Ojos del jugador en bloques (coma flotante, solo para presentación).
fn eye_of(sim: &Simulation, player: usize) -> Vec3 {
    let eye = sim.players()[player].eye();
    Vec3::from_array(eye.map(|c| c as f32 / FIXED_ONE as f32))
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
        self.crosshair = Some(CrosshairRenderer::new(&gpu.device, surface_format));
    }

    fn update(&mut self, dt: f32, input: &Input) {
        let (dx, dy) = input.mouse_delta();
        self.camera
            .rotate(-dx * MOUSE_SENSITIVITY, -dy * MOUSE_SENSITIVITY);
        self.handle_actions(input);
        let movement = self.move_input(input);
        self.step_simulation(dt, movement);

        let alpha = self.accumulator / TICK_SECONDS;
        self.camera.position = self.prev_eye.lerp(self.eye, alpha.clamp(0.0, 1.0));
        self.target = raycast(
            self.sim.world(),
            self.camera.position.to_array(),
            self.camera.forward().to_array(),
            AIM_REACH,
        );
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
        let stats = terrain.update(&gpu.device, self.sim.world_mut());
        if stats.chunks_meshed > 4 {
            log::debug!("{} chunks en {:.1} ms", stats.chunks_meshed, stats.millis);
        }
        let aspect = depth.width as f32 / depth.height as f32;
        let style = TerrainStyle {
            fog_distance: 56.0,
            selected: self.target.map(|hit| hit.block),
            lamp_radius: 14.0,
        };
        terrain.render(
            &gpu.queue,
            encoder,
            color,
            depth,
            &self.camera.view(aspect),
            &style,
        );
        if let Some(crosshair) = &self.crosshair {
            crosshair.render(&gpu.queue, encoder, color, depth.width, depth.height);
        }
    }
}
