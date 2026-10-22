use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::{Rng, SimReport};

/// Ticks de simulación por segundo (servidor autoritativo a 30 Hz).
pub const TICK_RATE: u32 = 30;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchConfig {
    pub seed: u64,
    /// Duración máxima en ticks. 16 minutos es el tope del Sismógrafo.
    pub max_ticks: u32,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            max_ticks: 16 * 60 * TICK_RATE,
        }
    }
}

/// Estado de una partida. De momento solo el reloj y un resumen del estado
/// que permite comprobar el determinismo.
pub struct Simulation {
    config: MatchConfig,
    tick: u32,
    rng: Rng,
    /// Huella acumulada del estado; dos simulaciones con las mismas entradas
    /// deben terminar con la misma huella en cualquier plataforma.
    state_hash: u64,
}

impl Simulation {
    pub fn new(config: MatchConfig) -> Self {
        Self {
            rng: Rng::new(config.seed),
            config,
            tick: 0,
            state_hash: 0xCBF2_9CE4_8422_2325,
        }
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn is_finished(&self) -> bool {
        self.tick >= self.config.max_ticks
    }

    /// Avanza un tick. Aquí irán movimiento, terreno, lava y Sismógrafo.
    pub fn step(&mut self) {
        let event = self.rng.next_u64();
        self.state_hash = (self.state_hash ^ event).wrapping_mul(0x0000_0100_0000_01B3);
        self.tick += 1;
    }

    pub fn run_to_end(mut self) -> SimReport {
        let mut tick_micros = Vec::with_capacity(self.config.max_ticks as usize);
        while !self.is_finished() {
            let start = Instant::now();
            self.step();
            tick_micros.push(start.elapsed().as_secs_f64() * 1e6);
        }
        SimReport::new(&self.config, self.tick, self.state_hash, tick_micros)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_same_final_state() {
        let config = MatchConfig {
            seed: 42,
            max_ticks: 600,
        };
        let a = crate::run(config.clone());
        let b = crate::run(config);
        assert_eq!(a.state_hash, b.state_hash);
        assert_eq!(a.ticks, 600);
    }

    #[test]
    fn different_seeds_diverge() {
        let a = crate::run(MatchConfig {
            seed: 1,
            max_ticks: 10,
        });
        let b = crate::run(MatchConfig {
            seed: 2,
            max_ticks: 10,
        });
        assert_ne!(a.state_hash, b.state_hash);
    }
}
