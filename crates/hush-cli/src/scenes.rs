//! Escenas de captura: cada una prepara un mundo y una cámara reproducibles
//! para que `hush-cli shot` genere siempre la misma imagen.

use anyhow::Result;
use clap::ValueEnum;
use glam::Vec3;
use grisu_render::{ColorBox, FlyCamera, OrbitCamera, TerrainStyle, View};
use grisu_voxel::{AIR, FIXED_ONE, VoxelWorld, raycast};
use hush_sim::{
    BlockKind, Command, MatchConfig, MoveInput, Simulation, TerrainConfig, generate_terrain,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Scene {
    /// Cubo de mineral del hito 0.
    Cube,
    /// El mapa completo visto desde fuera.
    Terrain,
    /// Túnel horizontal con un cruce, desde dentro.
    Tunnel,
    /// Pozo vertical visto desde arriba, con una escalera excavada.
    Shaft,
    /// Rampa excavada por un jugador simulado, vista desde sus ojos.
    Dig,
    /// Otros jugadores (cajas de las dos cuadrillas) en la superficie.
    Players,
}

/// Mundo, punto de vista y estilo de una escena de terreno.
pub struct TerrainScene {
    pub world: VoxelWorld,
    camera: Camera,
    pub style: TerrainStyle,
    /// Entidades (otros jugadores) que se dibujan sobre el terreno.
    pub boxes: Vec<ColorBox>,
}

enum Camera {
    Orbit(OrbitCamera),
    Fly(FlyCamera),
}

impl TerrainScene {
    pub fn view(&self, aspect: f32) -> View {
        match &self.camera {
            Camera::Orbit(c) => c.view(aspect),
            Camera::Fly(c) => c.view(aspect),
        }
    }
}

/// Construye una escena de terreno. `Scene::Cube` no tiene terreno.
pub fn build(scene: Scene, seed: u64) -> Result<Option<TerrainScene>> {
    let mut world = generate_terrain(seed, &TerrainConfig::default())?;
    let scene = match scene {
        Scene::Cube => return Ok(None),
        Scene::Terrain => TerrainScene {
            world,
            camera: Camera::Orbit(OrbitCamera {
                target: Vec3::new(32.0, 26.0, 32.0),
                distance: 120.0,
                yaw: 35f32.to_radians(),
                pitch: 30f32.to_radians(),
                fov_y: 50f32.to_radians(),
            }),
            boxes: Vec::new(),
            style: TerrainStyle {
                fog_distance: 400.0,
                selected: None,
                lamp_radius: 0.0,
                time_seconds: 0.0,
                fog_color: [0.02, 0.03, 0.05],
                daylight: 1.0,
            },
        },
        Scene::Tunnel => {
            carve_tunnel(&mut world);
            TerrainScene {
                world,
                camera: Camera::Fly(FlyCamera {
                    position: Vec3::new(9.5, 31.6, 32.5),
                    // Mirando hacia +X.
                    yaw: -std::f32::consts::FRAC_PI_2,
                    pitch: -4f32.to_radians(),
                    ..FlyCamera::default()
                }),
                boxes: Vec::new(),
                style: TerrainStyle {
                    fog_distance: 40.0,
                    selected: Some([14, 29, 33]),
                    lamp_radius: 14.0,
                    time_seconds: 0.0,
                    fog_color: [0.02, 0.03, 0.05],
                    daylight: 1.0,
                },
            }
        }
        Scene::Dig => dig_scene(seed)?,
        Scene::Players => players_scene(world),
        Scene::Shaft => {
            carve_shaft(&mut world);
            TerrainScene {
                world,
                camera: Camera::Fly(FlyCamera {
                    position: Vec3::new(32.5, 57.0, 32.5),
                    yaw: 0.0,
                    pitch: -89f32.to_radians(),
                    ..FlyCamera::default()
                }),
                boxes: Vec::new(),
                style: TerrainStyle {
                    fog_distance: 70.0,
                    selected: None,
                    lamp_radius: 14.0,
                    time_seconds: 0.0,
                    fog_color: [0.02, 0.03, 0.05],
                    daylight: 1.0,
                },
            }
        }
    };
    Ok(Some(scene))
}

/// Túnel de 3×3 de x = 8 a 56 con un cruce en x = 32 y vetas a la vista.
fn carve_tunnel(world: &mut VoxelWorld) {
    world.fill([8, 30, 31], [56, 32, 33], AIR);
    world.fill([31, 30, 16], [33, 32, 48], AIR);
    // Vetas en las paredes para comprobar las texturas.
    world.set(20, 31, 34, BlockKind::Gold.id());
    world.set(24, 32, 30, BlockKind::Ore.id());
    world.set(27, 30, 34, BlockKind::Diamond.id());
    world.set(14, 29, 33, BlockKind::Ore.id());
    world.fill([24, 29, 31], [26, 29, 33], BlockKind::Lava.id());
}

/// Pozo de 3×3 desde la superficie hasta y = 10, con escalones en espiral.
fn carve_shaft(world: &mut VoxelWorld) {
    world.fill([31, 10, 31], [33, 63, 33], AIR);
    let ring = [
        [30, 31],
        [30, 32],
        [30, 33],
        [31, 34],
        [32, 34],
        [33, 34],
        [34, 33],
        [34, 32],
        [34, 31],
        [33, 30],
        [32, 30],
        [31, 30],
    ];
    for y in 10..55 {
        let [x, z] = ring[(55 - y) as usize % ring.len()];
        world.fill([x, y, z], [x, y + 2, z], AIR);
    }
}

/// Un jugador apunta en diagonal hacia abajo, cava y avanza: el mismo ciclo
/// que el cliente (raycast, orden `Dig`, movimiento) validado por la
/// simulación.
fn dig_scene(seed: u64) -> Result<TerrainScene> {
    let mut sim = Simulation::new(MatchConfig {
        seed,
        max_ticks: u32::MAX,
        ..MatchConfig::default()
    })?;
    let player = sim.spawn_player();
    for _ in 0..60 {
        sim.step();
    }
    let to_fixed = |c: i32| c as f32 / FIXED_ONE as f32;
    let start_x = to_fixed(sim.players()[player].feet[0]);
    let toward = if start_x < 32.0 { 1.0 } else { -1.0 };
    let aim = Vec3::new(toward, -0.6, 0.0);
    for _ in 0..90 {
        let eye = sim.players()[player].eye().map(to_fixed);
        if let Some(hit) = raycast(sim.world(), eye, aim.to_array(), 4.0) {
            sim.submit(Command::Dig {
                player,
                cell: hit.block,
            });
        }
        sim.submit(Command::Move {
            player,
            input: MoveInput::from_direction(toward, 0.0, false),
        });
        sim.step();
    }
    let eye = Vec3::from_array(sim.players()[player].eye().map(to_fixed));
    let selected = raycast(sim.world(), eye.to_array(), aim.to_array(), 4.0).map(|h| h.block);
    // Se mira un poco más abajo que el rayo para ver el suelo de la rampa.
    let yaw = if toward > 0.0 {
        -std::f32::consts::FRAC_PI_2
    } else {
        std::f32::consts::FRAC_PI_2
    };
    Ok(TerrainScene {
        world: sim.world().clone(),
        camera: Camera::Fly(FlyCamera {
            position: eye,
            yaw,
            pitch: -25f32.to_radians(),
            ..FlyCamera::default()
        }),
        boxes: Vec::new(),
        style: TerrainStyle {
            fog_distance: 40.0,
            selected,
            lamp_radius: 14.0,
            time_seconds: 0.0,
            fog_color: [0.02, 0.03, 0.05],
            daylight: 1.0,
        },
    })
}

/// Tres mineros de pie en la superficie, delante de la cámara.
fn players_scene(world: VoxelWorld) -> TerrainScene {
    let ground = |x: i32, z: i32| {
        let feet = hush_sim::spawn_point(&world, x, z);
        Vec3::new(
            x as f32 + 0.5,
            feet[1] as f32 / FIXED_ONE as f32,
            z as f32 + 0.5,
        )
    };
    let colors = [[0.80, 0.38, 0.14], [0.16, 0.34, 0.80], [0.80, 0.38, 0.14]];
    let boxes = [(30, 24), (33, 22), (36, 25)]
        .iter()
        .zip(colors)
        .map(|(&(x, z), color)| {
            let feet = ground(x, z);
            ColorBox {
                min: feet - Vec3::new(0.3, 0.0, 0.3),
                size: Vec3::new(0.6, 1.7, 0.6),
                color,
            }
        })
        .collect();
    let eye = ground(33, 32) + Vec3::new(0.0, 1.55, 0.0);
    TerrainScene {
        camera: Camera::Fly(FlyCamera {
            position: eye,
            pitch: -12f32.to_radians(),
            ..FlyCamera::default()
        }),
        world,
        boxes,
        style: TerrainStyle {
            fog_distance: 56.0,
            selected: None,
            lamp_radius: 14.0,
            time_seconds: 0.0,
            fog_color: [0.02, 0.03, 0.05],
            daylight: 1.0,
        },
    }
}
