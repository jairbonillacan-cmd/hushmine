//! Sismógrafo, altar y registro de eventos dentro de la [`Simulation`].

use crate::altar::OFFERING_GOLD_COST;
use crate::{AltarError, EventLog, MatchEventKind, NoiseSource, Seismograph, Simulation, Team};

impl Simulation {
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
}
