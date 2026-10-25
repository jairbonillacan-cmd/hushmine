//! Escenas de captura: cada una prepara un mundo y una cámara reproducibles
//! para que `hush-cli shot` genere siempre la misma imagen.

use anyhow::Result;
use clap::ValueEnum;
use glam::Vec3;
use grisu_render::{ColorBox, FlyCamera, OrbitCamera, TerrainStyle, UiBatch, View};
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
    /// Cámara en tercera persona detrás del minero con su lámpara frontal.
    ThirdPerson,
    /// Menú del Launcher / Pausa superpuesto sobre la escena 3D.
    Menu,
    /// Panel completo del launcher. Capturar con `--width 1024 --height 640`.
    Launcher,
}

/// Mundo, punto de vista y estilo de una escena de terreno.
pub struct TerrainScene {
    pub world: VoxelWorld,
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
                world,
                camera: Camera::Fly(FlyCamera {
                    position: Vec3::new(9.5, 31.6, 32.5),
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
        Scene::Menu => menu_scene(world),
        Scene::Launcher => launcher_scene(world, seed),
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
                ui: UiBatch::new(),
                style: TerrainStyle {
                    fog_distance: 70.0,
                    selected: None,
                    lamp_radius: 14.0,
                    time_seconds: 0.0,
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
    let mut base = players_scene(world);
    let feet = hush_sim::spawn_point(&base.world, 33, 30);
    let my_feet = Vec3::new(33.5, feet[1] as f32 / FIXED_ONE as f32, 30.5);
    let eye = my_feet + Vec3::new(0.0, 1.55, 0.0);
    base.boxes.push(ColorBox {
        min: my_feet - Vec3::new(0.3, 0.0, 0.3),
        size: Vec3::new(0.6, 1.7, 0.6),
        color: [0.80, 0.38, 0.14],
    });
    base.boxes.push(ColorBox {
        min: eye + Vec3::new(0.0, 0.0, -0.27) - Vec3::splat(0.12),
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

/// Panel completo del launcher sobre el mapa visto desde lejos. El dibujo es
/// el mismo que el de la ventana real, a 1024×640. La semilla elige la
/// página: 2 Instalaciones, 3 Opciones, 4 Acerca de; cualquier otra, Jugar.
fn launcher_scene(world: VoxelWorld, seed: u64) -> TerrainScene {
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
    // Fijamos la animación para que el pulso y el fundido salgan igual en
    // cada captura, y generamos la miniatura del mapa de la semilla.
    state.animation = 0.4;
    state.page_fade = hush_launcher::theme::FADE_TIME;
    if let Ok(textures) = hush_art::block_textures() {
        hush_launcher::thumbnail::refresh(&mut state, &textures);
    }
    let mut ui = UiBatch::new();
    draw_launcher(&mut ui, &state, 1024.0, 640.0);
    TerrainScene {
        world,
        camera: Camera::Orbit(hush_launcher::panorama::panorama_camera(0.9)),
        boxes: Vec::new(),
        ui,
        style: TerrainStyle {
            fog_distance: 320.0,
            selected: None,
            lamp_radius: 0.0,
            time_seconds: 0.0,
        },
    }
}

/// Escena con el menú de Launcher / Pausa dibujado encima del mundo 3D.
fn menu_scene(world: VoxelWorld) -> TerrainScene {
    let mut scene = third_person_scene(world);
    let mut ui = UiBatch::new();
    ui.rect(0.0, 0.0, 512.0, 512.0, [0.02, 0.03, 0.04, 0.78]);
    ui.panel(
        36.0,
        56.0,
        440.0,
        396.0,
        [0.05, 0.07, 0.09, 0.96],
        [0.85, 0.69, 0.32, 1.0],
        3.0,
    );
    ui.rect(39.0, 59.0, 434.0, 44.0, [0.10, 0.12, 0.16, 1.0]);
    ui.text_centered(
        256.0,
        74.0,
        2.0,
        [0.93, 0.90, 0.83, 1.0],
        "HUSHMINE // LAUNCHER & MENU",
    );
    let items = [
        ("INICIAR PARTIDA LOCAL", true),
        ("BOTS EN PARTIDA: < 4 >", false),
        ("VISTA DE CAMARA: < 3A PERSONA >", false),
        ("MODO DE CARRERA: < MANTENER >", false),
        ("CAMBIAR CONTROLES Y RATON", false),
        ("SALIR DEL JUEGO", false),
    ];
    for (idx, (label, active)) in items.iter().enumerate() {
        let y = 120.0 + idx as f32 * 48.0;
        let bg = if *active {
            [0.16, 0.24, 0.18, 0.98]
        } else {
            [0.09, 0.11, 0.14, 0.92]
        };
        let border = if *active {
            [0.41, 0.94, 0.56, 1.0]
        } else {
            [0.28, 0.31, 0.38, 1.0]
        };
        ui.panel(60.0, y, 392.0, 38.0, bg, border, 2.0);
        let col = if *active {
            [0.78, 1.0, 0.84, 1.0]
        } else {
            [0.90, 0.88, 0.82, 1.0]
        };
        ui.text_centered(256.0, y + 12.0, 1.8, col, label);
    }
    scene.ui = ui;
    scene
}
