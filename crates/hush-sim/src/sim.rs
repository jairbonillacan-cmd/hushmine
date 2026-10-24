use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::altar::{Altar, AltarError, OFFERING_GOLD_COST};
use crate::{
    Economy, EventLog, LootStore, MatchEventKind, NoiseSource, Rng, SeismoEvent, Seismograph,
    SimReport, Team,
};

/// Ticks de simulación por segundo (servidor autoritativo a 30 Hz).
pub const TICK_RATE: u32 = 30; // Se moverá a data/*.ron

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
    pub(crate) config: MatchConfig,
    pub(crate) tick: u32,
    pub(crate) rng: Rng,
    pub(crate) seismograph: Seismograph,
    pub(crate) altar: Altar,
    pub(crate) economy: Economy,
    pub(crate) loot: LootStore,
    pub(crate) event_log: EventLog,
    pub(crate) awakening_tick: Option<u32>,
    /// Huella acumulada del estado; dos simulaciones con las mismas entradas
    /// deben terminar con la misma huella en cualquier plataforma.
    pub(crate) state_hash: u64,
}

impl Simulation {
    /// Crea una nueva simulación determinista a partir de la configuración dada.
    pub fn new(config: MatchConfig) -> Self {
        Self {
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

    /// Indica si la partida ha concluido (por alcanzar `max_ticks`, por finalizar el
    /// Despertar o por alcanzar la meta económica).
    pub fn is_finished(&self) -> bool {
        self.tick >= self.config.max_ticks
            || self.seismograph.is_awakening_ended()
            || self.economy.is_goal_reached()
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

    /// Ofrenda un lote de oro ([`OFFERING_GOLD_COST`]) en el altar del equipo.
    /// `carried_gold` es el oro que lleva el equipo; si la economía de bancos
    /// está activa, también exige y descuenta el coste del banco del equipo.
    pub fn offer_gold(&mut self, team: Team, carried_gold: u32) -> Result<u32, AltarError> {
        if self.seismograph.has_awakened() {
            return Err(AltarError::AltarClosed);
        }
        let requested = if self.economy.is_active() {
            let available = self.economy.bank(team).min(carried_gold);
            let req = self.altar.offer(self.tick, team, available)?;
            self.economy
                .spend_for_offering(team, OFFERING_GOLD_COST)
                .map_err(|_| AltarError::NotEnoughGold { carried: available })?;
            req
        } else {
            self.altar.offer(self.tick, team, carried_gold)?
        };
        let reduction = self.seismograph.relieve(requested);
        self.event_log.record(
            self.tick,
            MatchEventKind::OfferingMade {
                gold: OFFERING_GOLD_COST,
                reduction,
            },
            Some(team),
        );
        Ok(reduction)
    }

    /// Devuelve una ficha de latón al altar del equipo; devuelve cuánto bajó
    /// el Sismógrafo, o el motivo del rechazo.
    pub fn return_tally(&mut self, team: Team) -> Result<u32, AltarError> {
        if self.seismograph.has_awakened() {
            return Err(AltarError::AltarClosed);
        }
        let requested = self.altar.return_tally()?;
        let reduction = self.seismograph.relieve(requested);
        self.event_log.record(
            self.tick,
            MatchEventKind::TallyReturned { reduction },
            Some(team),
        );
        Ok(reduction)
    }

    /// Avanza la simulación un tick si la partida aún no ha terminado.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }

        self.tick += 1;
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
        if self.economy.is_active() || !self.loot.is_empty() {
            self.state_hash = (self.state_hash ^ u64::from(self.economy.bank(Team::Copper)))
                .wrapping_mul(0x0000_0100_0000_01B3);
            self.state_hash = (self.state_hash ^ u64::from(self.economy.bank(Team::Cobalt)))
                .wrapping_mul(0x0000_0100_0000_01B3);
            self.state_hash =
                (self.state_hash ^ (self.loot.len() as u64)).wrapping_mul(0x0000_0100_0000_01B3);
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
        let broken_invariants = self.check_invariants();
        SimReport::new(
            &self.config,
            self.tick,
            self.state_hash,
            self.event_log.thresholds_reached(),
            self.awakening_tick,
            tick_micros,
            broken_invariants,
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
        assert_eq!(sim.offer_gold(Team::Cobalt, 100), Ok(2_500));
        assert_eq!(sim.return_tally(Team::Copper), Ok(8_000));
        sim.add_noise(NoiseSource::Custom(100_000), Some(Team::Cobalt));
        sim.step();
        // Tras el Despertar el altar queda cerrado.
        assert_eq!(
            sim.offer_gold(Team::Copper, 100),
            Err(AltarError::AltarClosed)
        );
        assert_eq!(sim.return_tally(Team::Cobalt), Err(AltarError::AltarClosed));

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
        assert!(report.broken_invariants.is_empty());
    }

    #[test]
    fn offer_gold_deducts_from_team_bank_and_rejects_insufficient_bank() {
        let mut sim = Simulation::new(MatchConfig {
            seed: 21,
            max_ticks: 1_000,
        });
        sim.add_noise(NoiseSource::Custom(20_000), Some(Team::Copper));
        sim.mine_gold(1, Team::Copper, 1).unwrap();
        sim.deposit_player(1).unwrap();
        assert_eq!(sim.economy().bank(Team::Copper), 100);

        assert_eq!(sim.offer_gold(Team::Copper, 100), Ok(2_500));
        assert_eq!(sim.economy().bank(Team::Copper), 0);
        assert_eq!(sim.economy().total_offered_cash(), 100);
        assert!(sim.check_invariants().is_empty());

        // Avanzamos más allá del enfriamiento (150 ticks), pero el banco está a 0.
        for _ in 0..150 {
            sim.step();
        }
        assert_eq!(
            sim.offer_gold(Team::Copper, 100),
            Err(AltarError::NotEnoughGold { carried: 0 })
        );
    }
}
