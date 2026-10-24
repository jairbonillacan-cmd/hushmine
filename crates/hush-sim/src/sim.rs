use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::{
    EventLog, MatchEventKind, NoiseSource, Rng, SeismoEvent, Seismograph, SimReport, Team,
};

/// Ticks de simulación por segundo (servidor autoritativo a 30 Hz).
pub const TICK_RATE: u32 = 30;

/// Configuración inicial de una partida simulada.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchConfig {
    pub seed: u64,
    /// Duración máxima en ticks. 16 minutos es el tope pasivo del Sismógrafo.
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

/// Estado determinista de una partida de HUSHMINE.
pub struct Simulation {
    config: MatchConfig,
    tick: u32,
    rng: Rng,
    seismograph: Seismograph,
    event_log: EventLog,
    awakening_tick: Option<u32>,
    /// Huella acumulada del estado; dos simulaciones con las mismas entradas
    /// deben terminar con la misma huella en cualquier plataforma.
    state_hash: u64,
}

impl Simulation {
    /// Crea una nueva simulación determinista a partir de la configuración dada.
    pub fn new(config: MatchConfig) -> Self {
        Self {
            rng: Rng::new(config.seed),
            config,
            tick: 0,
            seismograph: Seismograph::new(),
            event_log: EventLog::new(),
            awakening_tick: None,
            state_hash: 0xCBF2_9CE4_8422_2325,
        }
    }

    /// Devuelve el tick actual de la partida.
    pub fn tick(&self) -> u32 {
        self.tick
    }

    /// Devuelve la huella acumulada del estado actual.
    pub fn state_hash(&self) -> u64 {
        self.state_hash
    }

    /// Devuelve una referencia inmutable al Sismógrafo de la partida.
    pub fn seismograph(&self) -> &Seismograph {
        &self.seismograph
    }

    /// Devuelve una referencia mutable al Sismógrafo de la partida.
    pub fn seismograph_mut(&mut self) -> &mut Seismograph {
        &mut self.seismograph
    }

    /// Devuelve el registro determinista de eventos ocurridos en la partida.
    pub fn event_log(&self) -> &EventLog {
        &self.event_log
    }

    /// Devuelve el tick en el que comenzó el Despertar, si ya se alcanzó.
    pub fn awakening_tick(&self) -> Option<u32> {
        self.awakening_tick
    }

    /// Indica si la partida ha concluido (por alcanzar `max_ticks` o por finalizar el Despertar).
    pub fn is_finished(&self) -> bool {
        self.tick >= self.config.max_ticks || self.seismograph.is_awakening_ended()
    }

    /// Añade ruido al Sismógrafo y lo anota en el registro si el incremento es positivo.
    pub fn add_noise(&mut self, source: NoiseSource, team: Option<Team>) -> u32 {
        let amount = self.seismograph.add_noise(source);
        if amount > 0 {
            self.event_log
                .record(self.tick, MatchEventKind::NoiseAdded { amount }, team);
        }
        amount
    }

    /// Realiza una ofrenda de oro para reducir el Sismógrafo antes del Despertar.
    pub fn offer_gold(&mut self, gold: u32, team: Option<Team>) -> u32 {
        let reduction = self.seismograph.offer(gold);
        if reduction > 0 {
            self.event_log.record(
                self.tick,
                MatchEventKind::OfferingMade { gold, reduction },
                team,
            );
        }
        reduction
    }

    /// Devuelve una ficha de latón al altar para reducir el Sismógrafo antes del Despertar.
    pub fn return_tally(&mut self, team: Option<Team>) -> u32 {
        let reduction = self.seismograph.return_tally();
        if reduction > 0 {
            self.event_log
                .record(self.tick, MatchEventKind::TallyReturned { reduction }, team);
        }
        reduction
    }

    /// Avanza la simulación un tick si la partida aún no ha terminado.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }

        self.tick += 1;
        let rng_event = self.rng.next_u64();
        let seismo_events = self.seismograph.step();

        for event in seismo_events {
            match event {
                SeismoEvent::ThresholdCrossed(t) => {
                    self.event_log
                        .record(self.tick, MatchEventKind::ThresholdCrossed(t), None);
                }
                SeismoEvent::AwakeningStarted => {
                    if self.awakening_tick.is_none() {
                        self.awakening_tick = Some(self.tick);
                    }
                    self.event_log
                        .record(self.tick, MatchEventKind::AwakeningStarted, None);
                }
                SeismoEvent::BucketLinesStopped => {
                    self.event_log
                        .record(self.tick, MatchEventKind::BucketLinesStopped, None);
                }
                SeismoEvent::AwakeningEnded => {
                    self.event_log
                        .record(self.tick, MatchEventKind::AwakeningEnded, None);
                }
            }
        }

        self.state_hash = (self.state_hash ^ rng_event).wrapping_mul(0x0000_0100_0000_01B3);
        self.state_hash = (self.state_hash ^ u64::from(self.seismograph.level()))
            .wrapping_mul(0x0000_0100_0000_01B3);
    }

    /// Ejecuta la simulación hasta su conclusión y devuelve el informe final.
    pub fn run_to_end(mut self) -> SimReport {
        let mut tick_micros = Vec::with_capacity(self.config.max_ticks as usize);
        while !self.is_finished() {
            let start = Instant::now();
            self.step();
            tick_micros.push(start.elapsed().as_secs_f64() * 1e6);
        }
        SimReport::new(
            &self.config,
            self.tick,
            self.state_hash,
            self.event_log.thresholds_reached(),
            self.awakening_tick,
            tick_micros,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SeismoThreshold;

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

    #[test]
    fn altering_seismograph_changes_final_state_hash() {
        let config = MatchConfig {
            seed: 77,
            max_ticks: 100,
        };
        let baseline = Simulation::new(config.clone()).run_to_end();

        let mut noisy_sim = Simulation::new(config);
        assert_eq!(
            noisy_sim.add_noise(NoiseSource::Explosion, Some(Team::Copper)),
            3_000
        );
        let noisy = noisy_sim.run_to_end();

        assert_ne!(baseline.state_hash, noisy.state_hash);
    }

    #[test]
    fn simulation_records_events_and_finishes_when_awakening_ends() {
        let mut sim = Simulation::new(MatchConfig {
            seed: 11,
            max_ticks: 50_000,
        });
        sim.add_noise(NoiseSource::Custom(50_000), Some(Team::Copper));
        sim.offer_gold(2, Some(Team::Cobalt));
        sim.return_tally(Some(Team::Copper));
        sim.add_noise(NoiseSource::Custom(100_000), Some(Team::Cobalt));

        let report = sim.run_to_end();
        assert_eq!(report.awakening_tick, Some(1));
        assert_eq!(report.ticks, 1 + 2_700);
        assert_eq!(
            report.thresholds_reached,
            vec![
                SeismoThreshold::Stir,
                SeismoThreshold::Breathe,
                SeismoThreshold::Restless,
                SeismoThreshold::Wake,
            ]
        );
    }
}
