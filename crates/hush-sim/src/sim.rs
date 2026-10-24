use std::collections::BTreeMap;
use std::time::Instant;

use grisu_voxel::VoxelWorld;
use serde::{Deserialize, Serialize};

use crate::player::spawn_point;
use crate::{Block, DiggerBot, EditError, MoveInput, PlayerBody, Rng, SimReport, TerrainConfig};

/// Ticks de simulación por segundo (servidor autoritativo a 30 Hz).
pub const TICK_RATE: u32 = 30;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchConfig {
    pub seed: u64,
    /// Duración máxima en ticks. 16 minutos es el tope del Sismógrafo.
    pub max_ticks: u32,
    pub terrain: TerrainConfig,
    /// Bots de prueba que cavan durante la simulación sin gráficos.
    pub bots: u32,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            max_ticks: 16 * 60 * TICK_RATE,
            terrain: TerrainConfig::default(),
            bots: 0,
        }
    }
}

/// Órdenes que llegan a la simulación (de un cliente o de un bot).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Move {
        player: usize,
        input: MoveInput,
    },
    Dig {
        player: usize,
        cell: [i32; 3],
    },
    Place {
        player: usize,
        cell: [i32; 3],
        block: Block,
    },
}

/// Contadores para el informe.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditStats {
    pub dug: BTreeMap<String, u32>,
    pub placed: u32,
    pub rejected: BTreeMap<String, u32>,
}

/// Estado de una partida.
pub struct Simulation {
    config: MatchConfig,
    tick: u32,
    rng: Rng,
    world: VoxelWorld,
    players: Vec<PlayerBody>,
    /// Entrada de movimiento vigente de cada jugador.
    inputs: Vec<MoveInput>,
    bots: Vec<(usize, DiggerBot)>,
    pending: Vec<Command>,
    stats: EditStats,
    broken_invariants: Vec<String>,
}

impl Simulation {
    /// Crea la partida. Falla si la configuración del mapa no es válida.
    pub fn new(config: MatchConfig) -> Result<Self, grisu_voxel::WorldError> {
        let world = crate::terrain::generate(config.seed, &config.terrain)?;
        let mut sim = Self {
            rng: Rng::new(config.seed),
            config,
            tick: 0,
            world,
            players: Vec::new(),
            inputs: Vec::new(),
            bots: Vec::new(),
            pending: Vec::new(),
            stats: EditStats::default(),
            broken_invariants: Vec::new(),
        };
        for i in 0..sim.config.bots {
            let player = sim.spawn_player();
            let seed = sim.config.seed ^ (u64::from(i) + 1).wrapping_mul(0x9E37_79B9);
            sim.bots.push((player, DiggerBot::new(seed)));
        }
        Ok(sim)
    }

    /// Añade un jugador en un punto de la superficie y devuelve su índice.
    pub fn spawn_player(&mut self) -> usize {
        let [sx, _, sz] = self.world.size();
        let x = 2 + self.rng.below(sx - 4) as i32;
        let z = 2 + self.rng.below(sz - 4) as i32;
        self.players
            .push(PlayerBody::new(spawn_point(&self.world, x, z)));
        self.inputs.push(MoveInput::default());
        self.players.len() - 1
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn world(&self) -> &VoxelWorld {
        &self.world
    }

    /// Acceso mutable al terreno, para que el cliente recoja los chunks sucios.
    pub fn world_mut(&mut self) -> &mut VoxelWorld {
        &mut self.world
    }

    pub fn players(&self) -> &[PlayerBody] {
        &self.players
    }

    pub fn stats(&self) -> &EditStats {
        &self.stats
    }

    pub fn is_finished(&self) -> bool {
        self.tick >= self.config.max_ticks
    }

    /// Encola una orden para el próximo tick.
    pub fn submit(&mut self, command: Command) {
        self.pending.push(command);
    }

    /// Avanza un tick: órdenes en el orden de llegada, luego movimiento.
    pub fn step(&mut self) {
        let mut commands = std::mem::take(&mut self.pending);
        for (player, bot) in &mut self.bots {
            commands.extend(bot.think(*player, &self.players[*player], self.tick));
        }
        for command in commands {
            if let Err(err) = self.apply(command) {
                *self.stats.rejected.entry(format!("{err:?}")).or_default() += 1;
            }
        }
        for (body, input) in self.players.iter_mut().zip(&self.inputs) {
            body.step(&self.world, *input);
        }
        self.check_invariants();
        self.tick += 1;
    }

    fn apply(&mut self, command: Command) -> Result<(), EditError> {
        let player = match command {
            Command::Move { player, .. }
            | Command::Dig { player, .. }
            | Command::Place { player, .. } => player,
        };
        let body = *self.players.get(player).ok_or(EditError::UnknownPlayer)?;
        match command {
            Command::Move { input, .. } => self.inputs[player] = input,
            Command::Dig { cell, .. } => {
                let block = crate::edit::dig(&mut self.world, &body, cell)?;
                *self.stats.dug.entry(block.name().to_owned()).or_default() += 1;
            }
            Command::Place { cell, block, .. } => {
                crate::edit::place(&mut self.world, &body, &self.players, cell, block)?;
                self.stats.placed += 1;
            }
        }
        Ok(())
    }

    fn check_invariants(&mut self) {
        for (i, body) in self.players.iter().enumerate() {
            if body.aabb().intersects_world(&self.world) {
                self.broken_invariants.push(format!(
                    "tick {}: jugador {i} dentro de un bloque",
                    self.tick
                ));
            }
            let [x, _, z] = body.feet.map(|c| c.div_euclid(grisu_voxel::FIXED_ONE));
            if !self.world.in_bounds(x, 0, z) || body.feet[1] < 0 {
                self.broken_invariants
                    .push(format!("tick {}: jugador {i} fuera del mapa", self.tick));
            }
        }
    }

    /// Huella del estado: terreno y jugadores. Dos simulaciones con las
    /// mismas entradas deben dar la misma huella en cualquier plataforma.
    pub fn state_hash(&self) -> u64 {
        let mut hash = self.world.content_hash();
        let mut mix = |v: i32| {
            hash = (hash ^ u64::from(v as u32)).wrapping_mul(0x0000_0100_0000_01B3);
        };
        mix(self.tick as i32);
        for body in &self.players {
            body.feet.iter().chain(&body.velocity).for_each(|&c| mix(c));
        }
        hash
    }

    pub fn run_to_end(mut self) -> SimReport {
        let mut tick_micros = Vec::with_capacity(self.config.max_ticks as usize);
        while !self.is_finished() {
            let start = Instant::now();
            self.step();
            tick_micros.push(start.elapsed().as_secs_f64() * 1e6);
        }
        let mut report = SimReport::new(&self.config, self.tick, self.state_hash(), tick_micros);
        report.edits = self.stats;
        // Se limita la lista para que el informe siga siendo legible.
        report.broken_invariants = self.broken_invariants.into_iter().take(20).collect();
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(seed: u64, ticks: u32, bots: u32) -> MatchConfig {
        MatchConfig {
            seed,
            max_ticks: ticks,
            bots,
            ..MatchConfig::default()
        }
    }

    #[test]
    fn same_seed_gives_same_final_state() {
        let a = crate::run(config(42, 600, 4)).unwrap();
        let b = crate::run(config(42, 600, 4)).unwrap();
        assert_eq!(a.state_hash, b.state_hash);
        assert_eq!(a.ticks, 600);
        assert_eq!(a.edits, b.edits);
    }

    #[test]
    fn different_seeds_diverge() {
        let a = crate::run(config(1, 10, 0)).unwrap();
        let b = crate::run(config(2, 10, 0)).unwrap();
        assert_ne!(a.state_hash, b.state_hash);
    }

    #[test]
    fn bots_dig_without_breaking_invariants() {
        let report = crate::run(config(42, 1800, 6)).unwrap();
        assert!(
            report.broken_invariants.is_empty(),
            "{:?}",
            report.broken_invariants
        );
        let dug: u32 = report.edits.dug.values().sum();
        assert!(dug > 50, "los bots apenas cavaron: {:?}", report.edits);
    }

    #[test]
    fn commands_from_unknown_players_are_rejected() {
        let mut sim = Simulation::new(config(5, 10, 0)).unwrap();
        sim.submit(Command::Dig {
            player: 99,
            cell: [5, 5, 5],
        });
        sim.step();
        assert_eq!(sim.stats().rejected.values().sum::<u32>(), 1);
    }

    #[test]
    fn invalid_map_size_is_an_error() {
        let mut c = config(1, 10, 0);
        c.terrain.size = [17, 64, 64];
        assert!(Simulation::new(c).is_err());
    }
}
