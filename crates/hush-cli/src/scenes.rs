//! Escenas de captura: cada una prepara un mundo y una cámara reproducibles
//! para que `hush-cli shot` genere siempre la misma imagen.

use anyhow::Result;
use clap::ValueEnum;
use glam::{DVec3, Vec3};
use grisu_render::{ColorBox, FlyCamera, OrbitCamera, TerrainStyle, UiBatch, View};
use grisu_voxel::{
    AIR, ChunkPos, ChunkedVoxels, FIXED_ONE, SparseWorld, VoxelWorld, VoxelsMut, raycast,
};
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
    /// El «Hito de la Frontera»: una estructura reconocible a 7,5 M bloques
    /// del origen, en un `SparseWorld`, para revisar a ojo que el render
    /// relativo no tiembla.
    Frontier,
    /// Cámara en tercera persona detrás del minero, con su lámpara.
    ThirdPerson,
    /// Menú de pausa del juego, dibujado con el mismo código que el cliente.
    Menu,
    /// Muestra de la fuente de la interfaz (mayúsculas, minúsculas, tildes,
    /// cifras) a varios tamaños, para revisar el texto de menús y HUD.
    Type,
    /// Lanzador completo (el mismo dibujo que la ventana real).
    Launcher,
}

/// Mundo de una escena: finito (el mapa del Turno) o abierto ([`SparseWorld`]).
pub enum SceneWorld {
    Finite(VoxelWorld),
    Sparse(SparseWorld),
}

impl SceneWorld {
    /// El mundo detrás de la interfaz de chunks, tal como la ve el render.
    /// Mutable: el renderer consume los chunks sucios.
    pub fn as_chunked_mut(&mut self) -> &mut dyn ChunkedVoxels {
        match self {
            Self::Finite(world) => world,
            Self::Sparse(world) => world,
        }
    }

    /// El mundo finito, si lo hay: solo él tiene mapa de luz.
    pub fn as_finite(&self) -> Option<&VoxelWorld> {
        match self {
            Self::Finite(world) => Some(world),
            Self::Sparse(_) => None,
        }
    }
}

/// Mundo, punto de vista y estilo de una escena de terreno.
pub struct TerrainScene {
    pub world: SceneWorld,
    camera: Camera,
    pub style: TerrainStyle,
    /// Entidades (otros jugadores) que se dibujan sobre el terreno.
    pub boxes: Vec<ColorBox>,
    /// Interfaz 2D opcional para verificar el menú y el HUD.
    pub ui: UiBatch,
}

enum Camera {
    Orbit(OrbitCamera),
    Fly(FlyCamera),
    ThirdPerson { cam: FlyCamera, distance: f32 },
}

impl TerrainScene {
    pub fn view(&self, aspect: f32) -> View {
        match &self.camera {
            Camera::Orbit(c) => c.view(aspect),
            Camera::Fly(c) => c.view(aspect),
            Camera::ThirdPerson { cam, distance } => {
                cam.third_person_view(aspect, *distance, false)
            }
        }
    }
}

/// Construye una escena de terreno. `Scene::Cube` no tiene terreno.
pub fn build(scene: Scene, seed: u64) -> Result<Option<TerrainScene>> {
    if scene == Scene::Cube {
        return Ok(None);
    }
    if scene == Scene::Frontier {
        return Ok(Some(frontier_scene()));
    }
    let mut world = generate_terrain(seed, &TerrainConfig::default())?;
    let scene = match scene {
        Scene::Terrain => TerrainScene {
            world: SceneWorld::Finite(world),
            camera: Camera::Orbit(OrbitCamera {
                target: Vec3::new(32.0, 26.0, 32.0),
                distance: 120.0,
                yaw: 35f32.to_radians(),
                pitch: 30f32.to_radians(),
                fov_y: 50f32.to_radians(),
            }),
            boxes: Vec::new(),
            ui: UiBatch::new(),
            style: TerrainStyle {
                fog_distance: 400.0,
                selected: None,
                lamp_radius: 0.0,
                time_seconds: 0.0,
            },
        },
        Scene::Tunnel => {
            carve_tunnel(&mut world);
            TerrainScene {
                world: SceneWorld::Finite(world),
                camera: Camera::Fly(FlyCamera {
                    position: DVec3::new(9.5, 31.6, 32.5),
                    // Mirando hacia +X.
                    yaw: -std::f32::consts::FRAC_PI_2,
                    pitch: -4f32.to_radians(),
                    ..FlyCamera::default()
                }),
                boxes: Vec::new(),
                ui: UiBatch::new(),
                style: TerrainStyle {
                    fog_distance: 40.0,
                    selected: Some([14, 29, 33]),
                    lamp_radius: 14.0,
                    time_seconds: 0.0,
                },
            }
        }
        Scene::Dig => dig_scene(seed)?,
        Scene::Players => players_scene(world),
        Scene::ThirdPerson => third_person_scene(world),
        Scene::Menu => menu_scene(world, seed),
        Scene::Launcher => launcher_scene(world, seed),
        Scene::Type => type_scene(world),
        Scene::Shaft => {
            carve_shaft(&mut world);
            TerrainScene {
                world: SceneWorld::Finite(world),
                camera: Camera::Fly(FlyCamera {
                    position: DVec3::new(32.5, 57.0, 32.5),
                    yaw: 0.0,
                    pitch: -89f32.to_radians(),
                    ..FlyCamera::default()
                }),
                boxes: Vec::new(),
                ui: UiBatch::new(),
                style: TerrainStyle {
                    fog_distance: 70.0,
                    selected: None,
                    lamp_radius: 14.0,
                    time_seconds: 0.0,
                },
            }
        }
        Scene::Cube | Scene::Frontier => unreachable!("cubo y frontera no generan terreno"),
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
    let to_blocks = |c: i64| c as f64 / FIXED_ONE as f64;
    let start_x = to_blocks(sim.players()[player].feet[0]);
    let toward = if start_x < 32.0 { 1.0 } else { -1.0 };
    let aim = DVec3::new(toward, -0.6, 0.0);
    for _ in 0..90 {
        let eye = sim.players()[player].eye().map(to_blocks);
        if let Some(hit) = raycast(sim.world(), eye, aim.to_array(), 4.0) {
            sim.submit(Command::Dig {
                player,
                cell: hit.block,
            });
        }
        sim.submit(Command::Move {
            player,
            input: MoveInput::from_direction(toward as f32, 0.0, false),
        });
        sim.step();
    }
    let eye = DVec3::from_array(sim.players()[player].eye().map(to_blocks));
    let selected = raycast(sim.world(), eye.to_array(), aim.to_array(), 4.0).map(|h| h.block);
    // Se mira un poco más abajo que el rayo para ver el suelo de la rampa.
    let yaw = if toward > 0.0 {
        -std::f32::consts::FRAC_PI_2
    } else {
        std::f32::consts::FRAC_PI_2
    };
    Ok(TerrainScene {
        world: SceneWorld::Finite(sim.world().clone()),
        camera: Camera::Fly(FlyCamera {
            position: eye,
            yaw,
            pitch: -25f32.to_radians(),
            ..FlyCamera::default()
        }),
        boxes: Vec::new(),
        ui: UiBatch::new(),
        style: TerrainStyle {
            fog_distance: 40.0,
            selected,
            lamp_radius: 14.0,
            time_seconds: 0.0,
        },
    })
}

/// Tres mineros de pie en la superficie, delante de la cámara.
fn players_scene(world: VoxelWorld) -> TerrainScene {
    let ground = |x: i32, z: i32| {
        let feet = hush_sim::spawn_point(&world, x, z);
        DVec3::new(
            f64::from(x) + 0.5,
            feet[1] as f64 / FIXED_ONE as f64,
            f64::from(z) + 0.5,
        )
    };
    let colors = [[0.80, 0.38, 0.14], [0.16, 0.34, 0.80], [0.80, 0.38, 0.14]];
    let boxes = [(30, 24), (33, 22), (36, 25)]
        .iter()
        .zip(colors)
        .map(|(&(x, z), color)| {
            let feet = ground(x, z);
            ColorBox {
                min: feet - DVec3::new(0.3, 0.0, 0.3),
                size: Vec3::new(0.6, 1.7, 0.6),
                color,
            }
        })
        .collect();
    let eye = ground(33, 32) + DVec3::new(0.0, 1.55, 0.0);
    TerrainScene {
        camera: Camera::Fly(FlyCamera {
            position: eye,
            pitch: -12f32.to_radians(),
            ..FlyCamera::default()
        }),
        world: SceneWorld::Finite(world),
        boxes,
        ui: UiBatch::new(),
        style: TerrainStyle {
            fog_distance: 56.0,
            selected: None,
            lamp_radius: 14.0,
            time_seconds: 0.0,
        },
    }
}

/// Cámara en tercera persona detrás de un minero que mira a otros dos.
fn third_person_scene(world: VoxelWorld) -> TerrainScene {
    let feet = hush_sim::spawn_point(&world, 33, 30);
    let my_feet = DVec3::new(33.5, feet[1] as f64 / FIXED_ONE as f64, 30.5);
    let eye = my_feet + DVec3::new(0.0, 1.55, 0.0);
    let mut base = players_scene(world);
    base.boxes.push(ColorBox {
        min: my_feet - DVec3::new(0.3, 0.0, 0.3),
        size: Vec3::new(0.6, 1.7, 0.6),
        color: [0.80, 0.38, 0.14],
    });
    base.boxes.push(ColorBox {
        min: eye + DVec3::new(0.0, 0.0, -0.27) - DVec3::splat(0.12),
        size: Vec3::splat(0.24),
        color: [0.98, 0.88, 0.45],
    });
    base.camera = Camera::ThirdPerson {
        cam: FlyCamera {
            position: eye,
            pitch: -14f32.to_radians(),
            ..FlyCamera::default()
        },
        distance: 3.2,
    };
    base
}

#[path = "scenes_ui.rs"]
mod ui;
use ui::{launcher_scene, menu_scene, type_scene};
#[path = "scenes_frontier.rs"]
mod frontier;
use frontier::frontier_scene;
