use serde::{Deserialize, Serialize};

use crate::{EditStats, MatchConfig, SeismoThreshold, TICK_RATE};

/// Informe de una partida simulada. Es lo que lee la IA tras `hush-cli sim`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimReport {
    pub seed: u64,
    pub ticks: u32,
    pub game_seconds: f64,
    /// Huella del estado final, en hexadecimal fijo para comparar a simple vista.
    #[serde(with = "hex_u64")]
    pub state_hash: u64,
    /// Umbrales del Sismógrafo alcanzados durante la partida, en orden de aparición.
    pub thresholds_reached: Vec<SeismoThreshold>,
    /// Tick en el que comenzó el Despertar (`100_000`), si se alcanzó.
    pub awakening_tick: Option<u32>,
    pub tick_p50_us: f64,
    pub tick_p95_us: f64,
    pub tick_max_us: f64,
    /// Bloques cavados por tipo, colocados y órdenes rechazadas.
    pub edits: EditStats,
    /// Invariantes rotas durante la partida. Debe estar vacío.
    pub broken_invariants: Vec<String>,
}

impl SimReport {
    pub(crate) fn new(
        config: &MatchConfig,
        ticks: u32,
        state_hash: u64,
        thresholds_reached: Vec<SeismoThreshold>,
        awakening_tick: Option<u32>,
        mut tick_micros: Vec<f64>,
        broken_invariants: Vec<String>,
    ) -> Self {
        tick_micros.sort_by(f64::total_cmp);
        Self {
            seed: config.seed,
            ticks,
            game_seconds: f64::from(ticks) / f64::from(TICK_RATE),
            state_hash,
            thresholds_reached,
            awakening_tick,
            tick_p50_us: percentile(&tick_micros, 0.50),
            tick_p95_us: percentile(&tick_micros, 0.95),
            tick_max_us: tick_micros.last().copied().unwrap_or(0.0),
            edits: EditStats::default(),
            broken_invariants,
        }
    }
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[index]
}

mod hex_u64 {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(value: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{value:016x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        let text = String::deserialize(d)?;
        u64::from_str_radix(&text, 16).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_round_trips_through_json() {
        let report = crate::run(MatchConfig {
            seed: 9,
            max_ticks: 30,
            bots: 2,
            ..MatchConfig::default()
        })
        .unwrap();
        let json = serde_json::to_string(&report).unwrap();
        let back: SimReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.seed, report.seed);
        assert_eq!(back.ticks, report.ticks);
        assert_eq!(back.state_hash, report.state_hash);
        assert_eq!(back.thresholds_reached, report.thresholds_reached);
        assert_eq!(back.awakening_tick, report.awakening_tick);
        assert_eq!(back.broken_invariants, report.broken_invariants);
        assert!(back.thresholds_reached.is_empty());
        assert_eq!(back.awakening_tick, None);
        assert!(json.contains("\"state_hash\":\""));
        assert!(json.contains("\"thresholds_reached\":[]"));
        assert!(json.contains("\"awakening_tick\":null"));
    }

    #[test]
    fn report_serializes_thresholds_and_awakening_tick() {
        let config = MatchConfig {
            seed: 42,
            max_ticks: 28_800,
            ..MatchConfig::default()
        };
        let report = SimReport::new(
            &config,
            28_800,
            0x0123_4567_89ab_cdef,
            vec![
                SeismoThreshold::Stir,
                SeismoThreshold::Breathe,
                SeismoThreshold::Restless,
                SeismoThreshold::Wake,
            ],
            Some(28_800),
            vec![10.0, 20.0, 30.0],
            Vec::new(),
        );
        let json = serde_json::to_string(&report).unwrap();
        let back: SimReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back, report);
        assert_eq!(
            back.thresholds_reached,
            vec![
                SeismoThreshold::Stir,
                SeismoThreshold::Breathe,
                SeismoThreshold::Restless,
                SeismoThreshold::Wake,
            ]
        );
        assert_eq!(back.awakening_tick, Some(28_800));
        assert!(back.broken_invariants.is_empty());
    }

    #[test]
    fn percentile_of_sorted_values() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&values, 0.5), 3.0);
        assert_eq!(percentile(&values, 0.95), 5.0);
        assert_eq!(percentile(&[], 0.95), 0.0);
    }
}
