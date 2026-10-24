//! Altar de cada cuadrilla: ofrendas de oro y devolución de fichas de latón.
//!
//! El altar solo decide si una entrega es válida (coste, enfriamiento por
//! equipo y tope de fichas por Turno) y cuánto alivio pide. Aplicar ese
//! alivio al Sismógrafo y cerrar el altar durante el Despertar es cosa de
//! [`crate::Simulation`]. Los valores son provisionales y coinciden con
//! `provisional_relief` en `data/seismograph.ron`.

use serde::{Deserialize, Serialize};

use crate::Team;

/// Oro que consume una ofrenda: un lote indivisible de `100 $` (**provisional**).
pub const OFFERING_GOLD_COST: u32 = 100; // Se moverá a data/*.ron
/// Reducción del Sismógrafo por ofrenda: `-2,500 %` (**provisional**).
pub const OFFERING_REDUCTION: u32 = 2_500; // Se moverá a data/*.ron
/// Ticks de espera entre dos ofrendas del mismo equipo (`5 s`, **provisional**).
pub const OFFERING_COOLDOWN_TICKS: u32 = 150; // Se moverá a data/*.ron
/// Reducción del Sismógrafo por ficha de latón devuelta: `-8,000 %` (**provisional**).
pub const TALLY_RETURN_REDUCTION: u32 = 8_000; // Se moverá a data/*.ron
/// Fichas de latón que se pueden devolver en todo el Turno, sumando ambos equipos (**provisional**).
pub const MAX_TALLIES_PER_SHIFT: u32 = 2; // Se moverá a data/*.ron

/// Motivo por el que el altar rechaza una entrega.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AltarError {
    /// El Despertar ya empezó: el altar no acepta nada hasta el final del Turno.
    AltarClosed,
    /// El equipo no lleva oro suficiente para un lote de ofrenda.
    NotEnoughGold { carried: u32 },
    /// El equipo ofrendó hace muy poco; podrá volver a hacerlo en `ready_at_tick`.
    CoolingDown { ready_at_tick: u32 },
    /// Ya se devolvieron todas las fichas permitidas en este Turno.
    TallyLimitReached,
}

/// Estado de los altares durante un Turno.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Altar {
    /// Tick de la última ofrenda aceptada de Copper y de Cobalt, en ese orden.
    last_offering_tick: [Option<u32>; 2],
    tallies_returned: u32,
}

impl Altar {
    /// Crea los altares vacíos al inicio del Turno.
    pub const fn new() -> Self {
        Self {
            last_offering_tick: [None, None],
            tallies_returned: 0,
        }
    }

    /// Fichas devueltas hasta ahora en el Turno.
    pub const fn tallies_returned(&self) -> u32 {
        self.tallies_returned
    }

    /// Acepta una ofrenda si el equipo lleva oro y no está en enfriamiento.
    /// Devuelve la reducción que se debe pedir al Sismógrafo; el oro consumido
    /// es siempre [`OFFERING_GOLD_COST`].
    pub fn offer(&mut self, tick: u32, team: Team, carried_gold: u32) -> Result<u32, AltarError> {
        if carried_gold < OFFERING_GOLD_COST {
            return Err(AltarError::NotEnoughGold {
                carried: carried_gold,
            });
        }
        let slot = &mut self.last_offering_tick[team_slot(team)];
        if let Some(last) = *slot {
            let ready_at_tick = last.saturating_add(OFFERING_COOLDOWN_TICKS);
            if tick < ready_at_tick {
                return Err(AltarError::CoolingDown { ready_at_tick });
            }
        }
        *slot = Some(tick);
        Ok(OFFERING_REDUCTION)
    }

    /// Acepta una ficha de latón si no se ha alcanzado el tope del Turno.
    /// Devuelve la reducción que se debe pedir al Sismógrafo.
    pub fn return_tally(&mut self) -> Result<u32, AltarError> {
        if self.tallies_returned >= MAX_TALLIES_PER_SHIFT {
            return Err(AltarError::TallyLimitReached);
        }
        self.tallies_returned += 1;
        Ok(TALLY_RETURN_REDUCTION)
    }
}

const fn team_slot(team: Team) -> usize {
    match team {
        Team::Copper => 0,
        Team::Cobalt => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offering_needs_a_full_gold_lot() {
        let mut altar = Altar::new();
        assert_eq!(
            altar.offer(10, Team::Copper, 99),
            Err(AltarError::NotEnoughGold { carried: 99 })
        );
        assert_eq!(altar.offer(10, Team::Copper, 100), Ok(OFFERING_REDUCTION));
    }

    #[test]
    fn cooldown_is_per_team() {
        let mut altar = Altar::new();
        assert_eq!(altar.offer(100, Team::Copper, 500), Ok(2_500));
        assert_eq!(
            altar.offer(249, Team::Copper, 500),
            Err(AltarError::CoolingDown { ready_at_tick: 250 })
        );
        // El rival no comparte el enfriamiento.
        assert_eq!(altar.offer(101, Team::Cobalt, 500), Ok(2_500));
        assert_eq!(altar.offer(250, Team::Copper, 500), Ok(2_500));
    }

    #[test]
    fn only_two_tallies_per_shift() {
        let mut altar = Altar::new();
        assert_eq!(altar.return_tally(), Ok(8_000));
        assert_eq!(altar.return_tally(), Ok(8_000));
        assert_eq!(altar.return_tally(), Err(AltarError::TallyLimitReached));
        assert_eq!(altar.tallies_returned(), MAX_TALLIES_PER_SHIFT);
    }
}
