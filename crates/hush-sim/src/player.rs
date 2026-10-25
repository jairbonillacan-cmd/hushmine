//! Movimiento del minero en punto fijo, a 30 ticks por segundo.
//!
//! El cliente lo usa para predecir y el servidor para decidir; por eso solo
//! hay enteros y la entrada llega ya cuantizada. Las posiciones son `i64`
//! Q47.16 (16 bits de fracción) para llegar a los ±7,5 millones de bloques
//! del mundo grande (ADR 0002); las velocidades, `i32` Q16.16, compatibles
//! con [`Fixed`](crate::Fixed).

use grisu_voxel::{Aabb, FIXED_ONE, VoxelWorld, Voxels, move_aabb};
use serde::{Deserialize, Serialize};

/// Un bloque en unidades de posición.
const ONE: i64 = FIXED_ONE;
/// Un bloque en unidades de velocidad por tick (Q16.16 sobre `i32`).
const ONE_PER_TICK: i32 = FIXED_ONE as i32;
/// Media anchura del minero (0,3 bloques).
pub const HALF_WIDTH: i64 = 3 * ONE / 10;
/// Altura del minero (1,7 bloques).
pub const HEIGHT: i64 = 17 * ONE / 10;
/// Altura de los ojos sobre los pies (1,55 bloques).
pub const EYE_HEIGHT: i64 = 155 * ONE / 100;
/// Velocidad al andar: 4,3 bloques/s.
const WALK_PER_TICK: i32 = 43 * ONE_PER_TICK / 300;
/// Gravedad: 28 bloques/s².
const GRAVITY_PER_TICK: i32 = 28 * ONE_PER_TICK / 900;
/// Velocidad inicial del salto (unos 1,25 bloques de altura): 0,278
/// bloques por tick, el `285` de las antiguas unidades de 1/1024 reescalado.
const JUMP_PER_TICK: i32 = 18_240;
/// Velocidad máxima de caída: 40 bloques/s.
const MAX_FALL_PER_TICK: i32 = 40 * ONE_PER_TICK / 30;

/// Entrada de un tick. La dirección deseada está en coordenadas del mundo y
/// cuantizada a `-127..=127` por eje; el cliente la calcula a partir de su
/// cámara.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MoveInput {
    pub wish_x: i8,
    pub wish_z: i8,
    pub jump: bool,
}

impl MoveInput {
    /// Cuantiza una dirección horizontal (no hace falta que sea unitaria).
    /// Solo la usa el cliente: a la simulación llega ya en enteros.
    pub fn from_direction(x: f32, z: f32, jump: bool) -> Self {
        let len = (x * x + z * z).sqrt();
        let (x, z) = if len > 1.0 {
            (x / len, z / len)
        } else {
            (x, z)
        };
        let q = |c: f32| (c * 127.0).round().clamp(-127.0, 127.0) as i8;
        Self {
            wish_x: q(x),
            wish_z: q(z),
            jump,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerBody {
    /// Centro de la base, en Q47.16 (`i64` con 16 bits de fracción).
    pub feet: [i64; 3],
    /// Bloques por tick en Q16.16 (`i32`, como [`Fixed`](crate::Fixed)).
    pub velocity: [i32; 3],
    pub on_ground: bool,
}

impl PlayerBody {
    pub fn new(feet: [i64; 3]) -> Self {
        Self {
            feet,
            velocity: [0; 3],
            on_ground: false,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::from_feet(self.feet, HALF_WIDTH, HEIGHT)
    }

    pub fn eye(&self) -> [i64; 3] {
        [self.feet[0], self.feet[1] + EYE_HEIGHT, self.feet[2]]
    }

    /// Avanza un tick contra cualquier mundo que implemente [`Voxels`]:
    /// el mapa finito del modo Turno o el mundo abierto del modo Historia.
    pub fn step(&mut self, world: &impl Voxels, input: MoveInput) {
        // Sin inercia horizontal: el movimiento responde al instante, como en
        // Infiniminer. Se divide entre 127 con redondeo hacia cero.
        self.velocity[0] = i32::from(input.wish_x) * WALK_PER_TICK / 127;
        self.velocity[2] = i32::from(input.wish_z) * WALK_PER_TICK / 127;
        if input.jump && self.on_ground {
            self.velocity[1] = JUMP_PER_TICK;
        }
        self.velocity[1] = (self.velocity[1] - GRAVITY_PER_TICK).max(-MAX_FALL_PER_TICK);

        let result = move_aabb(world, self.aabb(), self.velocity.map(i64::from));
        self.feet = result.aabb.feet();
        self.on_ground = result.blocked[1] && self.velocity[1] < 0;
        for axis in 0..3 {
            if result.blocked[axis] {
                self.velocity[axis] = 0;
            }
        }
    }
}

/// Primer hueco libre para aparecer en la columna `(x, z)`: encima del
/// bloque sólido más alto. Devuelve los pies en punto fijo Q47.16.
pub fn spawn_point(world: &VoxelWorld, x: i32, z: i32) -> [i64; 3] {
    let top = (0..world.size()[1] as i32)
        .rev()
        .find(|&y| world.is_solid(x, y, z))
        .map_or(0, |y| y + 1);
    [
        i64::from(x) * ONE + ONE / 2,
        i64::from(top) * ONE,
        i64::from(z) * ONE + ONE / 2,
    ]
}

#[cfg(test)]
mod tests {
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
}
