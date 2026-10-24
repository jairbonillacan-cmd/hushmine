//! Bot de prueba: anda al azar y cava lo que tiene delante.
//!
//! No juega bien; existe para que `hush-cli sim` ejercite el movimiento, la
//! colisión y la edición del terreno. Los bots de verdad llegan en el hito 2.

use grisu_voxel::FIXED_ONE;

use crate::{Command, MoveInput, PlayerBody, Rng};

#[derive(Clone, Debug)]
pub struct DiggerBot {
    rng: Rng,
    heading: [i8; 2],
}

/// Cada cuántos ticks cambia de rumbo y cada cuántos intenta cavar.
const TURN_EVERY: u32 = 45;
const DIG_EVERY: u32 = 8;

impl DiggerBot {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
            heading: [127, 0],
        }
    }

    /// Órdenes del bot para este tick.
    pub fn think(&mut self, player: usize, body: &PlayerBody, tick: u32) -> Vec<Command> {
        if tick.is_multiple_of(TURN_EVERY) {
            const HEADINGS: [[i8; 2]; 8] = [
                [127, 0],
                [90, 90],
                [0, 127],
                [-90, 90],
                [-127, 0],
                [-90, -90],
                [0, -127],
                [90, -90],
            ];
            self.heading = HEADINGS[self.rng.below(8) as usize];
        }
        let mut commands = vec![Command::Move {
            player,
            input: MoveInput {
                wish_x: self.heading[0],
                wish_z: self.heading[1],
                jump: self.rng.below(20) == 0,
            },
        }];
        if tick.is_multiple_of(DIG_EVERY) {
            // Celda delante de los pies, a veces la de abajo para bajar.
            let feet = body.feet.map(|c| c.div_euclid(FIXED_ONE));
            let ahead = [
                feet[0] + i32::from(self.heading[0].signum()),
                feet[1] + self.rng.below(2) as i32,
                feet[2] + i32::from(self.heading[1].signum()),
            ];
            let cell = if self.rng.below(4) == 0 {
                [feet[0], feet[1] - 1, feet[2]]
            } else {
                ahead
            };
            commands.push(Command::Dig { player, cell });
        }
        commands
    }
}
