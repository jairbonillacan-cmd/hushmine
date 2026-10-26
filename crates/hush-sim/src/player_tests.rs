//! Pruebas de `player`.

use super::*;
use grisu_voxel::{AIR, BlockId};

fn flat() -> VoxelWorld {
    let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
    w.fill([0, 0, 0], [31, 3, 31], 2);
    w
}

fn settle(body: &mut PlayerBody, world: &impl Voxels) {
    for _ in 0..60 {
        body.step(world, MoveInput::default());
    }
}

#[test]
fn lands_on_the_ground() {
    let w = flat();
    let mut body = PlayerBody::new([8 * ONE, 10 * ONE, 8 * ONE]);
    settle(&mut body, &w);
    assert!(body.on_ground);
    assert_eq!(body.feet[1], 4 * ONE);
    assert_eq!(spawn_point(&w, 8, 8)[1], 4 * ONE);
}

#[test]
fn sprints_about_six_and_a_half_blocks_per_second() {
    let w = flat();
    let mut body = PlayerBody::new(spawn_point(&w, 4, 8));
    settle(&mut body, &w);
    let start = body.feet[0];
    let input = MoveInput::from_direction(1.0, 0.0, false).with_sprint(true);
    for _ in 0..30 {
        body.step(&w, input);
    }
    let blocks = (body.feet[0] - start) as f64 / ONE as f64;
    assert!((6.2..6.8).contains(&blocks), "corrió {blocks} bloques");
}

#[test]
fn walks_about_four_blocks_per_second() {
    let w = flat();
    let mut body = PlayerBody::new(spawn_point(&w, 4, 8));
    settle(&mut body, &w);
    let start = body.feet[0];
    let input = MoveInput {
        wish_x: 127,
        ..MoveInput::default()
    };
    for _ in 0..30 {
        body.step(&w, input);
    }
    let blocks = (body.feet[0] - start) as f64 / ONE as f64;
    assert!((4.0..4.5).contains(&blocks), "anduvo {blocks} bloques");
}

#[test]
fn jump_clears_one_block_but_not_two() {
    let w = flat();
    let mut body = PlayerBody::new(spawn_point(&w, 8, 8));
    settle(&mut body, &w);
    let ground = body.feet[1];
    let mut peak = ground;
    body.step(
        &w,
        MoveInput {
            jump: true,
            ..MoveInput::default()
        },
    );
    for _ in 0..40 {
        body.step(&w, MoveInput::default());
        peak = peak.max(body.feet[1]);
    }
    assert!(peak - ground > ONE, "salto demasiado bajo");
    assert!(peak - ground < 2 * ONE, "salto demasiado alto");
    assert_eq!(body.feet[1], ground);
}

#[test]
fn quantized_input_is_bounded() {
    let i = MoveInput::from_direction(3.0, 4.0, false);
    assert_eq!((i.wish_x, i.wish_z), (76, 102));
    let i = MoveInput::from_direction(-1.0, 0.0, true);
    assert_eq!((i.wish_x, i.wish_z, i.jump), (-127, 0, true));
}

#[test]
fn same_inputs_same_path() {
    let w = flat();
    let run = || {
        let mut body = PlayerBody::new([5 * ONE, 9 * ONE, 5 * ONE]);
        for t in 0..300u32 {
            let input = MoveInput {
                wish_x: ((t * 37) % 255) as i8,
                wish_z: ((t * 91) % 255) as i8,
                jump: t % 17 == 0,
                sprint: t % 5 == 0,
                crouch: false,
            };
            body.step(&w, input);
        }
        body
    };
    assert_eq!(run(), run());
}

/// Plano uniforme: sólido por debajo de `y = 4` y aire encima, en
/// cualquier parte del mundo (propicio para comparar orígenes).
struct FlatPlane;

impl Voxels for FlatPlane {
    fn get(&self, _x: i32, y: i32, _z: i32) -> BlockId {
        if y < 4 { 2 } else { AIR }
    }
}

/// Plano con un escalón de un bloque a 8 y 9 bloques al este de
/// `origin`: andar contra él, saltarlo y caer al otro lado.
struct PlaneWithStep {
    origin: [i32; 3],
}

impl Voxels for PlaneWithStep {
    fn get(&self, x: i32, y: i32, _z: i32) -> BlockId {
        let rel = x - self.origin[0];
        if y < 4 || (y < 5 && (8..10).contains(&rel)) {
            2
        } else {
            AIR
        }
    }
}

/// Un minero cae, anda hacia el este saltando sin parar y recorre el
/// escalón. Devuelve el estado relativo a `origin` en cada tick.
fn walk_jump_fall<W: Voxels>(world: &W, origin: [i64; 3]) -> Vec<([i64; 3], [i32; 3], bool)> {
    let start = std::array::from_fn(|i| origin[i] + [0, 10 * ONE, 0][i]);
    let mut body = PlayerBody::new(start);
    let input = MoveInput {
        wish_x: 127,
        wish_z: 0,
        jump: true,
        sprint: false,
        crouch: false,
    };
    let mut trace = Vec::new();
    for _ in 0..300 {
        body.step(world, input);
        let rel = std::array::from_fn(|i| body.feet[i] - origin[i]);
        trace.push((rel, body.velocity, body.on_ground));
    }
    trace
}

#[test]
fn walk_jump_fall_trace_is_the_same_far_from_the_origin() {
    // El escalón vive a 8-9 bloques al este de cada origen, así que el
    // recorrido relativo debe ser idéntico en los dos sitios.
    let near = PlaneWithStep { origin: [0, 0, 0] };
    let far = PlaneWithStep {
        origin: [-7_499_800, 0, 7_499_800],
    };
    let from_zero = walk_jump_fall(&near, [0, 0, 0]);
    let from_far = walk_jump_fall(&far, [-7_499_800 * ONE, 0, 7_499_800 * ONE]);
    // El recorrido toca las fases buscadas: cae al inicio, salta, se
    // sostiene sobre el escalón (y = 5) y vuelve a caer al plano.
    assert!(
        from_far.iter().any(|(_, _, ground)| *ground),
        "el minero nunca tocó el suelo"
    );
    assert!(
        from_far
            .iter()
            .any(|(feet, _, ground)| *ground && feet[1] == 5 * ONE),
        "el minero nunca se sostuvo sobre el escalón"
    );
    assert!(
        from_far
            .last()
            .is_some_and(|(feet, _, _)| feet[0] > 12 * ONE),
        "el minero no pasó el escalón: {:?}",
        from_far.last()
    );
    assert_eq!(from_far, from_zero);
}

#[test]
fn walking_west_across_zero_matches_all_positive_ground() {
    // Andar hacia el oeste desde x = 2 cruza x = 0, -1, -16 y -17; en
    // terreno uniforme el desplazamiento debe ser el mismo que andando
    // la misma distancia sin cruzar el cero.
    let west = MoveInput {
        wish_x: -127,
        wish_z: 0,
        jump: false,
        sprint: false,
        crouch: false,
    };
    let run = |start_x: i64| {
        let mut body = PlayerBody::new([start_x * ONE + ONE / 2, 4 * ONE, 0]);
        settle(&mut body, &FlatPlane);
        let start = body.feet;
        for _ in 0..150 {
            body.step(&FlatPlane, west);
        }
        let rel: [i64; 3] = std::array::from_fn(|i| body.feet[i] - start[i]);
        (rel, body.velocity, body.on_ground)
    };
    let crossing = run(2);
    let positive = run(40);
    // Con 150 ticks a 4,3 bloques/s se avanzan unos 21 bloques: desde
    // x = 2 se cruza de sobra x = 0, -1, -16 y -17.
    assert!(crossing.0[0] < -19 * ONE, "cruzó poco: {:?}", crossing.0);
    assert_eq!(crossing, positive);
}

/// Plataforma de roca que acaba en `x = 9` (el bloque 9 es el último).
fn ledge() -> VoxelWorld {
    let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
    w.fill([0, 0, 0], [9, 3, 31], 2);
    w
}

#[test]
fn crouching_is_slow_and_lowers_the_eyes() {
    let w = flat();
    let mut body = PlayerBody::new(spawn_point(&w, 4, 8));
    settle(&mut body, &w);
    let start = body.feet[0];
    // Correr y agacharse a la vez: manda agacharse.
    let input = MoveInput::from_direction(1.0, 0.0, false)
        .with_sprint(true)
        .with_crouch(true);
    for _ in 0..30 {
        body.step(&w, input);
    }
    let advanced = body.feet[0] - start;
    assert!(
        (12 * ONE / 10..14 * ONE / 10).contains(&advanced),
        "agachado en 1 s: {advanced}"
    );
    assert!(body.crouching);
    assert_eq!(body.eye()[1], body.feet[1] + CROUCH_EYE_HEIGHT);
    body.step(&w, MoveInput::default());
    assert_eq!(body.eye()[1], body.feet[1] + EYE_HEIGHT);
}

#[test]
fn crouching_never_walks_off_a_ledge() {
    let w = ledge();
    let walk_off = |crouch: bool| {
        let mut body = PlayerBody::new(spawn_point(&w, 8, 8));
        settle(&mut body, &w);
        let input = MoveInput::from_direction(1.0, 0.0, false).with_crouch(crouch);
        for _ in 0..90 {
            body.step(&w, input);
        }
        body
    };
    let careful = walk_off(true);
    assert_eq!(careful.feet[1], 4 * ONE, "agachado no se cae");
    assert!(careful.on_ground);
    // Llega hasta asomarse: la base aún pisa el último bloque.
    assert!(
        careful.feet[0] > 10 * ONE - HALF_WIDTH,
        "{}",
        careful.feet[0]
    );
    assert!(careful.feet[0] < 10 * ONE + HALF_WIDTH);
    // De pie, el mismo paseo acaba en el suelo de abajo.
    assert!(walk_off(false).feet[1] < 4 * ONE);
}

#[test]
fn crouching_slides_along_a_ledge() {
    let w = ledge();
    let mut body = PlayerBody::new(spawn_point(&w, 9, 8));
    settle(&mut body, &w);
    let start_z = body.feet[2];
    // En diagonal hacia el vacío: el eje x se frena en el borde, el z sigue.
    let input = MoveInput::from_direction(1.0, 1.0, false).with_crouch(true);
    for _ in 0..60 {
        body.step(&w, input);
    }
    assert_eq!(body.feet[1], 4 * ONE);
    assert!(body.feet[0] < 10 * ONE + HALF_WIDTH);
    assert!(
        body.feet[2] - start_z > ONE / 2,
        "avanza a lo largo del borde"
    );
}

#[test]
fn crouching_in_the_air_still_falls() {
    let w = flat();
    let mut body = PlayerBody::new([8 * ONE, 10 * ONE, 8 * ONE]);
    let input = MoveInput::default().with_crouch(true);
    for _ in 0..60 {
        body.step(&w, input);
    }
    assert!(body.on_ground);
    assert_eq!(body.feet[1], 4 * ONE);
}
