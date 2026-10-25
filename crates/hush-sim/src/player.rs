//! Movimiento del minero en punto fijo, a 30 ticks por segundo.
//!
//! El cliente lo usa para predecir y el servidor para decidir; por eso solo
//! hay enteros y la entrada llega ya cuantizada.

use grisu_voxel::{Aabb, FIXED_ONE, VoxelWorld, move_aabb};
use serde::{Deserialize, Serialize};

const ONE: i32 = FIXED_ONE;
/// Media anchura del minero (0,3 bloques).
pub const HALF_WIDTH: i32 = 3 * ONE / 10;
/// Altura del minero (1,7 bloques).
pub const HEIGHT: i32 = 17 * ONE / 10;
/// Altura de los ojos sobre los pies (1,55 bloques).
pub const EYE_HEIGHT: i32 = 155 * ONE / 100;
/// Velocidad al andar: 4,3 bloques/s.
const WALK_PER_TICK: i32 = 43 * ONE / 300;
/// Velocidad al correr: 6,5 bloques/s.
const SPRINT_PER_TICK: i32 = 65 * ONE / 300;
/// Gravedad: 28 bloques/s².
const GRAVITY_PER_TICK: i32 = 28 * ONE / 900;
/// Velocidad inicial del salto (unos 1,25 bloques de altura).
const JUMP_PER_TICK: i32 = 285;
/// Velocidad máxima de caída: 40 bloques/s.
const MAX_FALL_PER_TICK: i32 = 40 * ONE / 30;

/// Entrada de un tick. La dirección deseada está en coordenadas del mundo y
/// cuantizada a `-127..=127` por eje; el cliente la calcula a partir de su
/// cámara.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MoveInput {
    pub wish_x: i8,
    pub wish_z: i8,
    pub jump: bool,
    pub sprint: bool,
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
            sprint: false,
        }
    }

    /// Activa o desactiva la carrera para esta entrada.
    pub fn with_sprint(mut self, sprint: bool) -> Self {
        self.sprint = sprint;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerBody {
    /// Centro de la base, en punto fijo.
    pub feet: [i32; 3],
    pub velocity: [i32; 3],
    pub on_ground: bool,
}

impl PlayerBody {
    pub fn new(feet: [i32; 3]) -> Self {
        Self {
            feet,
            velocity: [0; 3],
            on_ground: false,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::from_feet(self.feet, HALF_WIDTH, HEIGHT)
    }

    pub fn eye(&self) -> [i32; 3] {
        [self.feet[0], self.feet[1] + EYE_HEIGHT, self.feet[2]]
    }

    /// Avanza un tick.
    pub fn step(&mut self, world: &VoxelWorld, input: MoveInput) {
        // Sin inercia horizontal: el movimiento responde al instante, como en
        // Infiniminer. Se divide entre 127 con redondeo hacia cero.
        let speed = if input.sprint {
            SPRINT_PER_TICK
        } else {
            WALK_PER_TICK
        };
        self.velocity[0] = i32::from(input.wish_x) * speed / 127;
        self.velocity[2] = i32::from(input.wish_z) * speed / 127;
        if input.jump && self.on_ground {
            self.velocity[1] = JUMP_PER_TICK;
        }
        self.velocity[1] = (self.velocity[1] - GRAVITY_PER_TICK).max(-MAX_FALL_PER_TICK);

        let result = move_aabb(world, self.aabb(), self.velocity);
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
/// bloque sólido más alto. Devuelve los pies en punto fijo.
pub fn spawn_point(world: &VoxelWorld, x: i32, z: i32) -> [i32; 3] {
    let top = (0..world.size()[1] as i32)
        .rev()
        .find(|&y| world.is_solid(x, y, z))
        .map_or(0, |y| y + 1);
    [x * ONE + ONE / 2, top * ONE, z * ONE + ONE / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat() -> VoxelWorld {
        let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
        w.fill([0, 0, 0], [31, 3, 31], 2);
        w
    }

    fn settle(body: &mut PlayerBody, world: &VoxelWorld) {
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
        let blocks = f64::from(body.feet[0] - start) / f64::from(ONE);
        assert!((4.0..4.5).contains(&blocks), "anduvo {blocks} bloques");
    }

    #[test]
    fn sprints_about_six_point_five_blocks_per_second() {
        let w = flat();
        let mut body = PlayerBody::new(spawn_point(&w, 4, 8));
        settle(&mut body, &w);
        let start = body.feet[0];
        let input = MoveInput::from_direction(1.0, 0.0, false).with_sprint(true);
        for _ in 0..30 {
            body.step(&w, input);
        }
        let blocks = f64::from(body.feet[0] - start) / f64::from(ONE);
        assert!((6.2..6.8).contains(&blocks), "corrió {blocks} bloques");
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
                };
                body.step(&w, input);
            }
            body
        };
        assert_eq!(run(), run());
    }
}
