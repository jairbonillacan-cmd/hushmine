//! Equipos, eventos de partida y registro determinista para la Crónica.
//!
//! El [`EventLog`] conserva todos los hitos relevantes del Turno en estricto
//! orden de inserción para alimentar el resumen final y la verificación de
//! partidas simuladas.

use serde::{Deserialize, Serialize};

use crate::seismograph::SeismoThreshold;

/// Cuadrilla minera que compite en el Turno.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Team {
    /// Cuadrilla Cobre (naranja): familias antiguas del pueblo.
    Copper,
    /// Cuadrilla Cobalto (azul): recién llegados con material de la Compañía.
    Cobalt,
}

impl Team {
    /// Devuelve el identificador en minúsculas de la cuadrilla.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Copper => "copper",
            Self::Cobalt => "cobalt",
        }
    }

    /// Devuelve la cuadrilla rival.
    pub const fn opponent(self) -> Self {
        match self {
            Self::Copper => Self::Cobalt,
            Self::Cobalt => Self::Copper,
        }
    }
}

/// Categoría de un evento ocurrido durante la partida.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchEventKind {
    /// El Sismógrafo cruzó por primera vez uno de sus umbrales (25 %, 50 %, 75 % o 100 %).
    ThresholdCrossed(SeismoThreshold),
    /// Comenzó la fase de Despertar (quedan 90 s = 2_700 ticks).
    AwakeningStarted,
    /// Las líneas de cangilones se detuvieron (últimos 30 s = 900 ticks del Despertar).
    BucketLinesStopped,
    /// Terminó el Despertar y el monte Anima colapsó.
    AwakeningEnded,
    /// Se produjo un evento de ruido relevante (p. ej. explosión o descarga).
    NoiseAdded { amount: u32 },
    /// Un equipo realizó una ofrenda de oro en su altar para calmar al Durmiente.
    OfferingMade { gold: u32, reduction: u32 },
    /// Un equipo devolvió una ficha de latón al altar.
    TallyReturned { reduction: u32 },
}

/// Evento individual registrado en un tick concreto del Turno.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEvent {
    pub tick: u32,
    pub kind: MatchEventKind,
    pub team: Option<Team>,
}

/// Registro determinista de eventos de partida en orden de inserción.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventLog {
    events: Vec<MatchEvent>,
}

impl EventLog {
    /// Crea un registro de eventos vacío.
    pub const fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Añade un evento ya construido al final del registro.
    pub fn push(&mut self, event: MatchEvent) {
        self.events.push(event);
    }

    /// Registra un nuevo evento en el tick indicado.
    pub fn record(&mut self, tick: u32, kind: MatchEventKind, team: Option<Team>) {
        self.events.push(MatchEvent { tick, kind, team });
    }

    /// Devuelve la tajada con todos los eventos en orden de inserción.
    pub fn events(&self) -> &[MatchEvent] {
        &self.events
    }

    /// Devuelve el número total de eventos registrados.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Indica si el registro no contiene eventos.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Devuelve un iterador sobre los eventos en orden de inserción.
    pub fn iter(&self) -> std::slice::Iter<'_, MatchEvent> {
        self.events.iter()
    }

    /// Devuelve en orden de aparición los umbrales del Sismógrafo cruzados.
    pub fn thresholds_reached(&self) -> Vec<SeismoThreshold> {
        self.events
            .iter()
            .filter_map(|event| match event.kind {
                MatchEventKind::ThresholdCrossed(threshold) => Some(threshold),
                _ => None,
            })
            .collect()
    }

    /// Devuelve el tick del primer evento de inicio del Despertar, si ocurrió.
    pub fn awakening_tick(&self) -> Option<u32> {
        self.events.iter().find_map(|event| match event.kind {
            MatchEventKind::AwakeningStarted
            | MatchEventKind::ThresholdCrossed(SeismoThreshold::Wake) => Some(event.tick),
            _ => None,
        })
    }
}

impl<'a> IntoIterator for &'a EventLog {
    type Item = &'a MatchEvent;
    type IntoIter = std::slice::Iter<'a, MatchEvent>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log() -> EventLog {
        let mut log = EventLog::new();
        log.record(
            120,
            MatchEventKind::NoiseAdded { amount: 25 },
            Some(Team::Copper),
        );
        log.push(MatchEvent {
            tick: 300,
            kind: MatchEventKind::ThresholdCrossed(SeismoThreshold::Stir),
            team: None,
        });
        log.record(
            450,
            MatchEventKind::OfferingMade {
                gold: 3,
                reduction: 15,
            },
            Some(Team::Cobalt),
        );
        log.record(
            600,
            MatchEventKind::ThresholdCrossed(SeismoThreshold::Breathe),
            None,
        );
        log.record(
            750,
            MatchEventKind::TallyReturned { reduction: 10 },
            Some(Team::Copper),
        );
        log.record(
            900,
            MatchEventKind::ThresholdCrossed(SeismoThreshold::Restless),
            None,
        );
        log.record(
            1_200,
            MatchEventKind::ThresholdCrossed(SeismoThreshold::Wake),
            None,
        );
        log.record(1_200, MatchEventKind::AwakeningStarted, None);
        log.record(3_000, MatchEventKind::BucketLinesStopped, None);
        log.record(3_900, MatchEventKind::AwakeningEnded, None);
        log
    }

    #[test]
    fn team_helpers_return_expected_values() {
        assert_eq!(Team::Copper.name(), "copper");
        assert_eq!(Team::Cobalt.name(), "cobalt");
        assert_eq!(Team::Copper.opponent(), Team::Cobalt);
        assert_eq!(Team::Cobalt.opponent(), Team::Copper);
    }

    #[test]
    fn insertion_order_is_preserved_in_events_and_iter() {
        let log = sample_log();
        assert!(!log.is_empty());
        assert_eq!(log.len(), 10);

        let ticks: Vec<u32> = log.events().iter().map(|e| e.tick).collect();
        assert_eq!(
            ticks,
            vec![120, 300, 450, 600, 750, 900, 1_200, 1_200, 3_000, 3_900]
        );

        let iter_events: Vec<&MatchEvent> = log.iter().collect();
        let slice_events: Vec<&MatchEvent> = log.events().iter().collect();
        assert_eq!(iter_events, slice_events);

        assert_eq!(log.events()[0].team, Some(Team::Copper));
        assert_eq!(log.events()[1].team, None);
        assert_eq!(log.events()[2].team, Some(Team::Cobalt));
    }

    #[test]
    fn json_serialization_is_stable_and_round_trips() {
        let log = sample_log();
        let json_first = serde_json::to_string(&log).unwrap();
        let json_second = serde_json::to_string(&log).unwrap();
        assert_eq!(json_first, json_second);

        let decoded: EventLog = serde_json::from_str(&json_first).unwrap();
        assert_eq!(decoded, log);
    }

    #[test]
    fn thresholds_reached_and_awakening_tick_extract_expected_data() {
        let empty = EventLog::new();
        assert!(empty.is_empty());
        assert!(empty.thresholds_reached().is_empty());
        assert_eq!(empty.awakening_tick(), None);

        let log = sample_log();
        assert_eq!(
            log.thresholds_reached(),
            vec![
                SeismoThreshold::Stir,
                SeismoThreshold::Breathe,
                SeismoThreshold::Restless,
                SeismoThreshold::Wake,
            ]
        );
        assert_eq!(log.awakening_tick(), Some(1_200));

        let mut explicit_awake = EventLog::new();
        explicit_awake.record(2_100, MatchEventKind::AwakeningStarted, None);
        assert_eq!(explicit_awake.awakening_tick(), Some(2_100));
    }
}
