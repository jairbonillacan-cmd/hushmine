//! Escenas de captura: cada una prepara un mundo y una cámara reproducibles
//! para que `hush-cli shot` genere siempre la misma imagen.

use anyhow::Result;
use clap::ValueEnum;
use glam::{DVec3, Vec3};
use grisu_render::{ColorBox, FlyCamera, OrbitCamera, TerrainStyle, View};
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
        style: TerrainStyle {
            fog_distance: 56.0,
            selected: None,
            lamp_radius: 14.0,
            time_seconds: 0.0,
        },
    }
}

/// El «Hito de la Frontera»: una pirámide con obelisco, pilares y vetas de
/// color sobre una losa, a 7,5 millones de bloques del origen. Un `f32` ahí
/// solo distingue bloques enteros; con el render relativo al anchor la
/// escena debe verse idéntica a como se vería junto al origen.
fn frontier_scene() -> TerrainScene {
    /// Esquina del mundo grande donde se apoya la escena (Q47.16 alcanza de
    /// sobra: ±7,5 M bloques con 16 bits de fracción).
    const BASE: [i32; 3] = [7_499_800, 0, -7_499_800];
    let mut world = SparseWorld::new();
    let at = |dx: i32, dy: i32, dz: i32| [BASE[0] + dx, BASE[1] + dy, BASE[2] + dz];
    // Losa de roca dura de 25×25, un bloque de grueso.
    world.fill(at(-12, -1, -12), at(12, -1, 12), BlockKind::HardRock.id());
    // Pirámide de tres escalones con un obelisco coronado en oro.
    world.fill(at(-4, 0, -4), at(4, 0, 4), BlockKind::Rock.id());
    world.fill(at(-3, 1, -3), at(3, 1, 3), BlockKind::Rock.id());
    world.fill(at(-2, 2, -2), at(2, 2, 2), BlockKind::Rock.id());
    world.fill(at(0, 3, 0), at(0, 9, 0), BlockKind::Rock.id());
    world.fill(at(0, 10, 0), at(0, 10, 0), BlockKind::Gold.id());
    // Cuatro pilares de mena en las esquinas de la losa.
    for [sx, sz] in [[-9, -9], [9, -9], [-9, 9], [9, 9]] {
        world.fill(at(sx, 0, sz), at(sx, 3, sz), BlockKind::Ore.id());
    }
    // Franjas de diamante y oro para ver las texturas de lejos.
    world.fill(at(-10, 0, -2), at(-8, 0, 2), BlockKind::Diamond.id());
    world.fill(at(8, 0, -2), at(10, 0, 2), BlockKind::Gold.id());
    // Los chunks quedan cargados y marcados como sucios por `set`/`fill`.
    assert!(world.has_dirty_chunks());
    assert!(world.is_loaded(ChunkPos::containing(BASE)));
    let base = DVec3::new(f64::from(BASE[0]), f64::from(BASE[1]), f64::from(BASE[2]));
    TerrainScene {
        world: SceneWorld::Sparse(world),
        camera: Camera::Fly(FlyCamera {
            // Desde una esquina, mirando al obelisco en diagonal (+X, +Z).
            position: base + DVec3::new(-11.5, 6.5, -11.5),
            yaw: -3.0 * std::f32::consts::FRAC_PI_4,
            pitch: -14f32.to_radians(),
            ..FlyCamera::default()
        }),
        boxes: Vec::new(),
        style: TerrainStyle {
            fog_distance: 90.0,
            selected: Some(at(0, 10, 0)),
            lamp_radius: 0.0,
            time_seconds: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontier_structure_is_far_from_the_origin_and_loaded() {
        let scene = frontier_scene();
        let SceneWorld::Sparse(world) = &scene.world else {
            unreachable!("la frontera es un SparseWorld");
        };
        assert!(!world.is_empty());
        // La cámara está a millones de bloques del origen.
        let Camera::Fly(camera) = &scene.camera else {
            unreachable!();
        };
        assert!(camera.position.x.abs() > 7.0e6);
        assert!(camera.position.z.abs() > 7.0e6);
        // Y el bloque señalado es el remate del obelisco.
        assert_eq!(scene.style.selected, Some([7_499_800, 10, -7_499_800]));
    }
}
