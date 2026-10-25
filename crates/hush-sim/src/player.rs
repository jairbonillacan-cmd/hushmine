//! Movimiento del minero en punto fijo, a 30 ticks por segundo.
//!
//! El cliente lo usa para predecir y el servidor para decidir; por eso solo
//! hay enteros y la entrada llega ya cuantizada. Las posiciones son `i64`
//! Q47.16 (16 bits de fracción) para llegar a los ±7,5 millones de bloques
//! del mundo grande (ADR 0002); las velocidades, `i32` Q16.16, compatibles
//! con [`Fixed`](crate::Fixed).

use grisu_voxel::{Aabb, FIXED_ONE, VoxelWorld, Voxels, cell_of, move_aabb};
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
/// Altura de los ojos agachado (1,25 bloques).
pub const CROUCH_EYE_HEIGHT: i64 = 125 * ONE / 100;
/// Velocidad al andar: 4,3 bloques/s.
pub const WALK_PER_TICK: i32 = 43 * ONE_PER_TICK / 300;
/// Velocidad al correr: 6,5 bloques/s.
pub const SPRINT_PER_TICK: i32 = 65 * ONE_PER_TICK / 300;
/// Velocidad agachado: 1,3 bloques/s.
pub const CROUCH_PER_TICK: i32 = 13 * ONE_PER_TICK / 300;
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
    pub sprint: bool,
    /// Agachado: va despacio y en silencio, baja los ojos y no se cae por
    /// los bordes. Manda sobre `sprint`.
    pub crouch: bool,
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
            crouch: false,
        }
    }

    /// Activa o desactiva la carrera para esta entrada.
    pub fn with_sprint(mut self, sprint: bool) -> Self {
        self.sprint = sprint;
        self
    }

    /// Activa o desactiva agacharse para esta entrada.
    pub fn with_crouch(mut self, crouch: bool) -> Self {
        self.crouch = crouch;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerBody {
    /// Centro de la base, en Q47.16 (`i64` con 16 bits de fracción).
    pub feet: [i64; 3],
    /// Bloques por tick en Q16.16 (`i32`, como [`Fixed`](crate::Fixed)).
    pub velocity: [i32; 3],
    pub on_ground: bool,
    /// Agachado en el último tick (lo decide la entrada).
    pub crouching: bool,
}

impl PlayerBody {
    pub fn new(feet: [i64; 3]) -> Self {
        Self {
            feet,
            velocity: [0; 3],
            on_ground: false,
            crouching: false,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::from_feet(self.feet, HALF_WIDTH, HEIGHT)
    }

    pub fn eye(&self) -> [i64; 3] {
        let height = if self.crouching {
            CROUCH_EYE_HEIGHT
        } else {
            EYE_HEIGHT
        };
        [self.feet[0], self.feet[1] + height, self.feet[2]]
    }

    /// ¿Hay suelo bajo algún punto de la base con los pies en `feet`?
    fn supported_at(world: &impl Voxels, feet: [i64; 3]) -> bool {
        let y = cell_of(feet[1] - 1);
        let span = |c: i64| cell_of(c - HALF_WIDTH)..=cell_of(c + HALF_WIDTH - 1);
        span(feet[0]).any(|x| span(feet[2]).any(|z| world.is_solid(x, y, z)))
    }

    /// Avanza un tick contra cualquier mundo que implemente [`Voxels`]:
    /// el mapa finito del modo Turno o el mundo abierto del modo Historia.
    pub fn step(&mut self, world: &impl Voxels, input: MoveInput) {
        // Sin inercia horizontal: el movimiento responde al instante, como en
        // Infiniminer. Se divide entre 127 con redondeo hacia cero.
        let speed = if input.crouch {
            CROUCH_PER_TICK
        } else if input.sprint {
            SPRINT_PER_TICK
        } else {
            WALK_PER_TICK
        };
        self.crouching = input.crouch;
        self.velocity[0] = i32::from(input.wish_x) * speed / 127;
        self.velocity[2] = i32::from(input.wish_z) * speed / 127;
        if input.jump && self.on_ground {
            self.velocity[1] = JUMP_PER_TICK;
        }
        self.velocity[1] = (self.velocity[1] - GRAVITY_PER_TICK).max(-MAX_FALL_PER_TICK);

        if input.crouch && self.on_ground && !input.jump {
            self.guard_edge(world);
        }
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

impl PlayerBody {
    /// Agachado sobre el suelo: anula el avance de cada eje horizontal que
    /// dejaría la base sin apoyo, como en Minecraft. Se prueba el paso
    /// completo y después cada eje por separado, para poder deslizarse a lo
    /// largo de un borde sin caer.
    fn guard_edge(&mut self, world: &impl Voxels) {
        let step = |body: &Self, vx: i32, vz: i32| {
            let delta = [i64::from(vx), 0, i64::from(vz)];
            let moved = move_aabb(world, body.aabb(), delta).aabb.feet();
            Self::supported_at(world, moved)
        };
        let [vx, _, vz] = self.velocity;
        if step(self, vx, vz) {
            return;
        }
        if !step(self, vx, 0) {
            self.velocity[0] = 0;
        }
        if !step(self, self.velocity[0], vz) {
            self.velocity[2] = 0;
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
#[path = "player_tests.rs"]
mod tests;
