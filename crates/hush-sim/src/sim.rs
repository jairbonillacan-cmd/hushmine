use std::collections::BTreeMap;
use std::time::Instant;

use grisu_voxel::VoxelWorld;
use serde::{Deserialize, Serialize};

use crate::altar::Altar;
use crate::{
    BlockKind, DiggerBot, Economy, EditError, EventLog, LootStore, MatchEventKind, MoveInput,
    PlayerBody, Rng, SeismoEvent, Seismograph, SimReport, Team, TerrainConfig,
};

/// Ticks de simulación por segundo (servidor autoritativo a 30 Hz).
pub const TICK_RATE: u32 = 30; // Se moverá a data/*.ron

/// Configuración inicial de una partida simulada.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchConfig {
    pub seed: u64,
    /// Duración máxima en ticks. 16 minutos es el tope pasivo del Sismógrafo.
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
        block: BlockKind,
    },
}

/// Bloque que cambió en un tick (para enviar a los clientes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockChange {
    pub cell: [i32; 3],
    pub block: u8,
}

/// Acción de un jugador que la simulación rechazó (el cliente que la
/// predijo debe corregirse).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rejection {
    pub player: usize,
    pub cell: [i32; 3],
    pub error: EditError,
}

/// Contadores de edición del terreno para el informe.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditStats {
    pub dug: BTreeMap<String, u32>,
    pub placed: u32,
    pub rejected: BTreeMap<String, u32>,
}

/// Estado determinista de una partida de HUSHMINE.
pub struct Simulation {
    pub(crate) config: MatchConfig,
    pub(crate) tick: u32,
    pub(crate) rng: Rng,
    // Sismógrafo, altar, economía y registro de eventos.
    pub(crate) seismograph: Seismograph,
    pub(crate) altar: Altar,
    pub(crate) economy: Economy,
    pub(crate) loot: LootStore,
    pub(crate) event_log: EventLog,
    pub(crate) awakening_tick: Option<u32>,
    /// Huella acumulada del estado; dos simulaciones con las mismas entradas
    /// deben terminar con la misma huella en cualquier plataforma.
    pub(crate) state_hash: u64,
    // Terreno, mineros y bots.
    pub(crate) world: VoxelWorld,
    pub(crate) players: Vec<PlayerBody>,
    /// Los jugadores que se fueron conservan su índice pero dejan de existir
    /// en el mundo.
    pub(crate) active: Vec<bool>,
    /// Entrada de movimiento vigente de cada jugador.
    pub(crate) inputs: Vec<MoveInput>,
    pub(crate) bots: Vec<(usize, DiggerBot)>,
    pub(crate) bots_active: bool,
    pub(crate) pending: Vec<Command>,
    pub(crate) stats: EditStats,
    pub(crate) changes: Vec<BlockChange>,
    pub(crate) rejections: Vec<Rejection>,
    pub(crate) world_violations: Vec<String>,
}

impl Simulation {
    /// Crea la partida. Falla si la configuración del mapa no es válida.
    pub fn new(config: MatchConfig) -> Result<Self, grisu_voxel::WorldError> {
        let world = crate::terrain::generate(config.seed, &config.terrain)?;
        let mut sim = Self {
            rng: Rng::new(config.seed),
            config,
            tick: 0,
            seismograph: Seismograph::new(),
            altar: Altar::new(),
            economy: Economy::with_default_team_size(),
            loot: LootStore::new(),
            event_log: EventLog::new(),
            awakening_tick: None,
            state_hash: 0xCBF2_9CE4_8422_2325,
            world,
            players: Vec::new(),
            active: Vec::new(),
            inputs: Vec::new(),
            bots: Vec::new(),
            bots_active: true,
            pending: Vec::new(),
            stats: EditStats::default(),
            changes: Vec::new(),
            rejections: Vec::new(),
            world_violations: Vec::new(),
        };
        for i in 0..sim.config.bots {
            let player = sim.spawn_player();
            let seed = sim.config.seed ^ (u64::from(i) + 1).wrapping_mul(0x9E37_79B9);
            sim.bots.push((player, DiggerBot::new(seed)));
        }
        Ok(sim)
    }

    /// Devuelve el tick actual de la partida.
    pub fn tick(&self) -> u32 {
        self.tick
    }

    /// Huella del estado: la acumulada por tick más el terreno y los
    /// mineros actuales.
    pub fn state_hash(&self) -> u64 {
        let mut hash = self.state_hash ^ self.world.content_hash();
        let mut mix = |v: i32| {
            hash = (hash ^ u64::from(v as u32)).wrapping_mul(0x0000_0100_0000_01B3);
        };
        for body in &self.players {
            body.feet.iter().chain(&body.velocity).for_each(|&c| mix(c));
        }
        hash
    }

    /// Indica si la partida ha concluido (por alcanzar `max_ticks`, por finalizar el
    /// Despertar o por alcanzar la meta económica).
    pub fn is_finished(&self) -> bool {
        self.tick >= self.config.max_ticks
            || self.seismograph.is_awakening_ended()
            || self.economy.is_goal_reached()
    }

    /// Avanza la simulación un tick si la partida aún no ha terminado:
    /// órdenes en el orden de llegada, movimiento, botín y Sismógrafo.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        self.tick += 1;
        self.step_world();
        self.step_loot();

        let rng_event = self.rng.next_u64();
        for event in self.seismograph.step() {
            let kind = match event {
                SeismoEvent::ThresholdCrossed(t) => MatchEventKind::ThresholdCrossed(t),
                SeismoEvent::AwakeningStarted => {
                    self.awakening_tick.get_or_insert(self.tick);
                    MatchEventKind::AwakeningStarted
                }
                SeismoEvent::BucketLinesStopped => MatchEventKind::BucketLinesStopped,
                SeismoEvent::AwakeningEnded => MatchEventKind::AwakeningEnded,
            };
            self.event_log.record(self.tick, kind, None);
        }
        self.mix_hash(rng_event);
        self.mix_hash(u64::from(self.seismograph.level()));
        if self.economy.is_active() || !self.loot.is_empty() {
            self.mix_hash(u64::from(self.economy.bank(Team::Copper)));
            self.mix_hash(u64::from(self.economy.bank(Team::Cobalt)));
            self.mix_hash(self.loot.len() as u64);
        }
    }

    fn mix_hash(&mut self, value: u64) {
        self.state_hash = (self.state_hash ^ value).wrapping_mul(0x0000_0100_0000_01B3);
    }

    fn step_loot(&mut self) {
        for bag in self.loot.expire_due_bags(self.tick) {
            let cash_value = bag.base_cash_value().unwrap_or(0);
            let _ = self.economy.record_expired_bag_cash(cash_value);
            self.event_log.record(
                self.tick,
                MatchEventKind::LootExpired {
                    bag_id: bag.id,
                    cash_value,
                },
                Some(bag.origin_team),
            );
        }
    }

    /// Ejecuta la simulación hasta su conclusión y devuelve el informe final.
    pub fn run_to_end(mut self) -> SimReport {
        let mut tick_micros = Vec::with_capacity(self.config.max_ticks as usize);
        while !self.is_finished() {
            let start = Instant::now();
            self.step();
            tick_micros.push(start.elapsed().as_secs_f64() * 1e6);
        }
        let mut broken_invariants = self.check_invariants();
        // Se limita la lista para que el informe siga siendo legible.
        let violations = std::mem::take(&mut self.world_violations);
        broken_invariants.extend(violations.into_iter().take(20));
        let mut report = SimReport::new(
            &self.config,
            self.tick,
            self.state_hash(),
            self.event_log.thresholds_reached(),
            self.awakening_tick,
            tick_micros,
            broken_invariants,
        );
        report.edits = std::mem::take(&mut self.stats);
        report
    }
}

#[cfg(test)]
#[path = "sim_tests.rs"]
mod tests;
