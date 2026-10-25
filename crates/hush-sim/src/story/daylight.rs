//! Ciclo diurno del modo historia: 24 000 ticks (20 minutos a 20 Hz).
//!
//! El multiplicador de la luz de cielo se guarda en Q8 (256 = 1.0): día
//! pleno 1.0, ocaso progresivo hasta 0.08, noche cerrada con luz lunar
//! tenue y amanecer de vuelta a 1.0.

use serde::{Deserialize, Serialize};

/// Duración del ciclo orbital, en ticks de simulación.
pub const DAY_TICKS: u32 = 24_000;
/// Fin del día pleno.
pub const DUSK_START: u32 = 12_000;
/// Comienzo de la noche cerrada.
pub const NIGHT_START: u32 = 13_800;
/// Comienzo del amanecer.
pub const DAWN_START: u32 = 22_200;
/// Luz lunar tenue de la noche (0.08 en Q8).
pub const NIGHT_SKY_Q8: u16 = 20;
/// Luz cenital plena (1.0 en Q8).
pub const DAY_SKY_Q8: u16 = 256;

/// Fase del día.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Day,
    Dusk,
    Night,
    Dawn,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Day => "DIA",
            Self::Dusk => "OCASO",
            Self::Night => "NOCHE",
            Self::Dawn => "AMANECER",
        }
    }
}

/// Reloj de día de la expedición.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayCycle {
    tick: u32,
}

impl Default for DayCycle {
    fn default() -> Self {
        // La expedición empieza por la mañana.
        Self { tick: 1_000 }
    }
}

impl DayCycle {
    pub fn new(tick: u32) -> Self {
        Self {
            tick: tick % DAY_TICKS,
        }
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn advance(&mut self) {
        self.tick = (self.tick + 1) % DAY_TICKS;
    }

    pub fn phase(&self) -> Phase {
        match self.tick {
            t if t < DUSK_START => Phase::Day,
            t if t < NIGHT_START => Phase::Dusk,
            t if t < DAWN_START => Phase::Night,
            _ => Phase::Dawn,
        }
    }

    /// Multiplicador de la luz de cielo en Q8 (256 = 1.0), con las
    /// transiciones lineales del documento.
    pub fn sky_factor_q8(&self) -> u16 {
        match self.tick {
            t if t < DUSK_START => DAY_SKY_Q8,
            t if t < NIGHT_START => {
                // Ocaso: de 1.0 a 0.08 entre 12 000 y 13 800.
                let span = NIGHT_START - DUSK_START;
                let at = t - DUSK_START;
                let drop = u32::from(DAY_SKY_Q8 - NIGHT_SKY_Q8) * at / span;
                DAY_SKY_Q8 - drop as u16
            }
            t if t < DAWN_START => NIGHT_SKY_Q8,
            t => {
                // Amanecer: de 0.08 a 1.0 entre 22 200 y 24 000.
                let span = DAY_TICKS - DAWN_START;
                let at = t - DAWN_START;
                let rise = u32::from(DAY_SKY_Q8 - NIGHT_SKY_Q8) * at / span;
                NIGHT_SKY_Q8 + rise as u16
            }
        }
    }

    /// La noche activa la hostilidad en el exterior.
    pub fn is_night(&self) -> bool {
        matches!(self.phase(), Phase::Night)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phases_follow_the_document_schedule() {
        assert_eq!(DayCycle::new(0).phase(), Phase::Day);
        assert_eq!(DayCycle::new(11_999).phase(), Phase::Day);
        assert_eq!(DayCycle::new(12_000).phase(), Phase::Dusk);
        assert_eq!(DayCycle::new(13_799).phase(), Phase::Dusk);
        assert_eq!(DayCycle::new(13_800).phase(), Phase::Night);
        assert_eq!(DayCycle::new(22_199).phase(), Phase::Night);
        assert_eq!(DayCycle::new(22_200).phase(), Phase::Dawn);
        assert_eq!(DayCycle::new(23_999).phase(), Phase::Dawn);
    }

    #[test]
    fn sky_factor_curve_hits_the_key_points() {
        assert_eq!(DayCycle::new(0).sky_factor_q8(), 256);
        assert_eq!(DayCycle::new(12_000).sky_factor_q8(), 256);
        assert_eq!(DayCycle::new(13_800).sky_factor_q8(), 20);
        assert_eq!(DayCycle::new(18_000).sky_factor_q8(), 20);
        // Mitad del ocaso: aproximadamente 0.54.
        assert!((120..150).contains(&DayCycle::new(12_900).sky_factor_q8()));
        // Mitad del amanecer: aproximadamente 0.54.
        assert!((120..150).contains(&DayCycle::new(23_100).sky_factor_q8()));
        assert_eq!(DayCycle::new(23_999).sky_factor_q8(), 255);
    }

    #[test]
    fn cycle_wraps_at_twenty_four_thousand() {
        let mut clock = DayCycle::new(DAY_TICKS - 1);
        clock.advance();
        assert_eq!(clock.tick(), 0);
        assert_eq!(clock.phase(), Phase::Day);
        assert!(DayCycle::new(15_000).is_night());
    }
}
