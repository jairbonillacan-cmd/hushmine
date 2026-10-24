//! Sistema del Sismógrafo y ciclo del Despertar.
//!
//! Mide la tensión acumulada en milésimas de punto porcentual (`0..=100_000`).
//! Avanza de forma determinista a 30 Hz por tiempo pasivo, líneas de cangilones
//! activas y eventos ruidosos; puede reducirse antes del Despertar mediante
//! ofrendas de oro o devolución de fichas de latón.

use serde::{Deserialize, Serialize};

/// Nivel máximo del Sismógrafo (`100,000 %` expresado en milésimas de punto porcentual).
pub const MAX_LEVEL: u32 = 100_000; // Se moverá a data/*.ron
/// Ticks necesarios (16 minutos a 30 Hz) para alcanzar el Despertar solo por tiempo pasivo.
pub const PASSIVE_TICKS_TO_WAKE: u32 = 28_800; // Se moverá a data/*.ron
/// Umbral del primer temblor: Revuelo (`25,000 %`).
pub const THRESHOLD_STIR: u32 = 25_000; // Se moverá a data/*.ron
/// Umbral del segundo temblor: Respira (`50,000 %`).
pub const THRESHOLD_BREATHE: u32 = 50_000; // Se moverá a data/*.ron
/// Umbral del tercer temblor: Inquietud (`75,000 %`).
pub const THRESHOLD_RESTLESS: u32 = 75_000; // Se moverá a data/*.ron
/// Umbral del cuarto estado: Despertar (`100,000 %`).
pub const THRESHOLD_WAKE: u32 = 100_000; // Se moverá a data/*.ron
/// Duración total del Despertar en ticks (`90 s` a 30 Hz).
pub const AWAKENING_DURATION_TICKS: u32 = 2_700; // Se moverá a data/*.ron
/// Ticks restantes del Despertar cuando se detienen las líneas de cangilones (`30 s` a 30 Hz).
pub const AWAKENING_BUCKET_STOP_REMAINING_TICKS: u32 = 900; // Se moverá a data/*.ron
/// Incremento de ruido provocado por una explosión (`+3,000 %`).
pub const NOISE_EXPLOSION: u32 = 3_000; // Se moverá a data/*.ron
/// Incremento de ruido provocado por una descarga eléctrica (`+0,200 %`).
pub const NOISE_ELECTRIC_DISCHARGE: u32 = 200; // Se moverá a data/*.ron
/// Incremento de ruido provocado por un evento de lava que fluye (`+0,200 %`).
pub const NOISE_FLOWING_LAVA: u32 = 200; // Se moverá a data/*.ron
/// Incremento de ruido por tick de cada línea de cangilones activa (`+0,300 %` cada `10 s`).
pub const NOISE_BUCKET_LINE_PER_TICK: u32 = 1; // Se moverá a data/*.ron

/// Hitos sísmicos que el Sismógrafo alcanza a lo largo de la partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SeismoThreshold {
    /// Revuelo (`25 %`).
    Stir,
    /// Respira (`50 %`).
    Breathe,
    /// Inquietud (`75 %`).
    Restless,
    /// Despertar (`100 %`).
    Wake,
}

impl SeismoThreshold {
    /// Lista ordenada de todos los umbrales del Sismógrafo.
    pub const ALL: [Self; 4] = [Self::Stir, Self::Breathe, Self::Restless, Self::Wake];

    /// Nivel en milésimas de punto porcentual necesario para cruzar este umbral.
    pub const fn target_level(self) -> u32 {
        match self {
            Self::Stir => THRESHOLD_STIR,
            Self::Breathe => THRESHOLD_BREATHE,
            Self::Restless => THRESHOLD_RESTLESS,
            Self::Wake => THRESHOLD_WAKE,
        }
    }
}

/// Fuentes de ruido capaces de incrementar el nivel del Sismógrafo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NoiseSource {
    /// Detonación de explosivos (`+3_000`).
    Explosion,
    /// Descarga eléctrica en el subsuelo (`+200`).
    ElectricDischarge,
    /// Flujo activo de lava (`+200`).
    FlowingLava,
    /// Una única línea de cangilones activa durante un tick (`+1`).
    ActiveBucketLine,
    /// Múltiples líneas de cangilones activas durante un tick (`count * 1`).
    BucketLines(u32),
    /// Fuente de ruido arbitraria para pruebas o eventos especiales.
    Custom(u32),
}

impl NoiseSource {
    /// Incremento en milésimas de punto porcentual asociado a la fuente de ruido.
    pub const fn delta(self) -> u32 {
        match self {
            Self::Explosion => NOISE_EXPLOSION,
            Self::ElectricDischarge => NOISE_ELECTRIC_DISCHARGE,
            Self::FlowingLava => NOISE_FLOWING_LAVA,
            Self::ActiveBucketLine => NOISE_BUCKET_LINE_PER_TICK,
            Self::BucketLines(count) => count.saturating_mul(NOISE_BUCKET_LINE_PER_TICK),
            Self::Custom(amount) => amount,
        }
    }
}

/// Eventos discretos emitidos por el Sismógrafo durante la simulación.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SeismoEvent {
    /// Se ha superado un umbral sísmico por primera vez en la partida.
    ThresholdCrossed(SeismoThreshold),
    /// El Sismógrafo ha alcanzado `100_000` y comienza la secuencia del Despertar (`2_700` ticks).
    AwakeningStarted,
    /// Quedan `900` ticks del Despertar y las líneas de cangilones se detienen.
    BucketLinesStopped,
    /// Se han completado los `2_700` ticks del Despertar y la partida concluye.
    AwakeningEnded,
}

/// Estado determinista del Sismógrafo de la partida.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seismograph {
    level: u32,
    passive_remainder: u32,
    active_bucket_lines: u32,
    crossed_thresholds: [bool; 4],
    awakening_ticks_elapsed: Option<u32>,
    bucket_lines_stopped: bool,
    awakening_ended: bool,
}

impl Default for Seismograph {
    fn default() -> Self {
        Self::new()
    }
}

impl Seismograph {
    /// Crea un Sismógrafo en reposo al inicio de la partida.
    pub const fn new() -> Self {
        Self {
            level: 0,
            passive_remainder: 0,
            active_bucket_lines: 0,
            crossed_thresholds: [false; 4],
            awakening_ticks_elapsed: None,
            bucket_lines_stopped: false,
            awakening_ended: false,
        }
    }

    /// Nivel actual del Sismógrafo en milésimas de punto porcentual (`0..=100_000`).
    pub const fn level(&self) -> u32 {
        self.level
    }

    /// Resto entero acumulado de la división del crecimiento pasivo (`0..28_800`).
    pub const fn passive_remainder(&self) -> u32 {
        self.passive_remainder
    }

    /// Número de líneas de cangilones configuradas como activas.
    pub const fn active_bucket_lines(&self) -> u32 {
        self.active_bucket_lines
    }

    /// Define el número de líneas de cangilones activas que aportan ruido cada tick.
    pub fn set_active_bucket_lines(&mut self, lines: u32) {
        self.active_bucket_lines = lines;
    }

    /// Indica si el Despertar está en curso y aún no ha finalizado.
    pub const fn is_awakening(&self) -> bool {
        self.awakening_ticks_elapsed.is_some() && !self.awakening_ended
    }

    /// Indica si la secuencia del Despertar ha concluido por completo.
    pub const fn is_awakening_ended(&self) -> bool {
        self.awakening_ended
    }

    /// Devuelve los ticks restantes del Despertar, o `None` si aún no ha comenzado.
    pub const fn awakening_ticks_remaining(&self) -> Option<u32> {
        match self.awakening_ticks_elapsed {
            Some(elapsed) => Some(AWAKENING_DURATION_TICKS.saturating_sub(elapsed)),
            None => None,
        }
    }

    /// Indica si las líneas de cangilones han sido detenidas por el avance del Despertar.
    pub const fn are_bucket_lines_stopped(&self) -> bool {
        self.bucket_lines_stopped
    }

    /// Aplica un evento de ruido saturando en `MAX_LEVEL` y devuelve el incremento real aplicado.
    pub fn add_noise(&mut self, source: NoiseSource) -> u32 {
        let is_bucket = matches!(
            source,
            NoiseSource::ActiveBucketLine | NoiseSource::BucketLines(_)
        );
        if self.bucket_lines_stopped && is_bucket {
            return 0;
        }
        let previous = self.level;
        self.level = self.level.saturating_add(source.delta()).min(MAX_LEVEL);
        self.level - previous
    }

    /// Indica si el Despertar ya empezó (aunque haya terminado). Desde ese
    /// momento el altar no acepta ofrendas ni fichas.
    pub const fn has_awakened(&self) -> bool {
        self.awakening_ticks_elapsed.is_some()
    }

    /// Baja el nivel por una ofrenda o una ficha aceptadas por el altar, sin
    /// pasar de `0`. Devuelve la reducción real; tras el Despertar no hace nada.
    pub fn relieve(&mut self, amount: u32) -> u32 {
        if self.has_awakened() {
            return 0;
        }
        let reduction = amount.min(self.level);
        self.level -= reduction;
        reduction
    }

    /// Avanza la simulación del Sismógrafo un tick y devuelve los eventos producidos.
    pub fn step(&mut self) -> Vec<SeismoEvent> {
        let mut events = Vec::new();
        let was_awakening = self.awakening_ticks_elapsed.is_some();

        // 1. Crecimiento pasivo y ruido por líneas activas antes del Despertar.
        if !was_awakening {
            let num = self.passive_remainder + MAX_LEVEL;
            let passive_delta = num / PASSIVE_TICKS_TO_WAKE;
            self.passive_remainder = num % PASSIVE_TICKS_TO_WAKE;
            let lines_delta = if self.bucket_lines_stopped {
                0
            } else {
                self.active_bucket_lines
                    .saturating_mul(NOISE_BUCKET_LINE_PER_TICK)
            };
            self.level = self
                .level
                .saturating_add(passive_delta)
                .saturating_add(lines_delta)
                .min(MAX_LEVEL);
        }

        // 2. Comprobación en orden de los 4 umbrales (cada uno se emite una sola vez).
        for (i, threshold) in SeismoThreshold::ALL.into_iter().enumerate() {
            if self.level >= threshold.target_level() && !self.crossed_thresholds[i] {
                self.crossed_thresholds[i] = true;
                events.push(SeismoEvent::ThresholdCrossed(threshold));
            }
        }

        // 3. Inicio del Despertar al alcanzar el nivel máximo.
        if self.level >= MAX_LEVEL && self.awakening_ticks_elapsed.is_none() {
            self.awakening_ticks_elapsed = Some(0);
            events.push(SeismoEvent::AwakeningStarted);
        }

        // 4. Progreso del Despertar si ya estaba activo en un tick anterior.
        if was_awakening
            && !self.awakening_ended
            && let Some(elapsed) = self.awakening_ticks_elapsed.as_mut()
        {
            *elapsed = elapsed.saturating_add(1);
            let current = *elapsed;
            let rem = AWAKENING_DURATION_TICKS.saturating_sub(current);
            if rem <= AWAKENING_BUCKET_STOP_REMAINING_TICKS && !self.bucket_lines_stopped {
                self.bucket_lines_stopped = true;
                events.push(SeismoEvent::BucketLinesStopped);
            }
            if current >= AWAKENING_DURATION_TICKS {
                self.awakening_ended = true;
                events.push(SeismoEvent::AwakeningEnded);
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passive_growth_reaches_exact_max_at_tick_28_800() {
        let mut seismo = Seismograph::new();
        for _ in 0..28_799 {
            seismo.step();
        }
        assert_eq!(seismo.level(), 99_996);
        assert!(!seismo.is_awakening());

        let events = seismo.step();
        assert_eq!(seismo.level(), MAX_LEVEL);
        assert_eq!(seismo.passive_remainder(), 0);
        assert!(seismo.is_awakening());
        assert_eq!(
            events,
            vec![
                SeismoEvent::ThresholdCrossed(SeismoThreshold::Wake),
                SeismoEvent::AwakeningStarted,
            ]
        );
    }

    #[test]
    fn each_threshold_emitted_only_once_even_after_reduction() {
        let mut seismo = Seismograph::new();
        seismo.add_noise(NoiseSource::Custom(THRESHOLD_BREATHE));
        let first_events = seismo.step();
        assert_eq!(
            first_events,
            vec![
                SeismoEvent::ThresholdCrossed(SeismoThreshold::Stir),
                SeismoEvent::ThresholdCrossed(SeismoThreshold::Breathe),
            ]
        );

        seismo.relieve(THRESHOLD_BREATHE);
        assert!(seismo.level() < THRESHOLD_STIR);

        seismo.add_noise(NoiseSource::Custom(THRESHOLD_RESTLESS));
        let second_events = seismo.step();
        assert_eq!(
            second_events,
            vec![SeismoEvent::ThresholdCrossed(SeismoThreshold::Restless)]
        );
    }

    #[test]
    fn level_stays_clamped_between_zero_and_max() {
        let mut seismo = Seismograph::new();
        assert_eq!(seismo.relieve(1_000), 0);
        assert_eq!(seismo.level(), 0);

        seismo.add_noise(NoiseSource::Explosion);
        assert_eq!(seismo.level(), 3_000);
        assert_eq!(seismo.relieve(8_000), 3_000);
        assert_eq!(seismo.level(), 0);

        for _ in 0..50 {
            seismo.add_noise(NoiseSource::Explosion);
        }
        assert_eq!(seismo.level(), MAX_LEVEL);
    }

    #[test]
    fn explosion_and_three_bucket_lines_add_expected_noise() {
        let mut direct = Seismograph::new();
        assert_eq!(direct.add_noise(NoiseSource::Explosion), 3_000);
        assert_eq!(direct.level(), 3_000);

        let mut bucket_only = Seismograph::new();
        for _ in 0..300 {
            bucket_only.add_noise(NoiseSource::BucketLines(3));
        }
        assert_eq!(bucket_only.level(), 900);

        let mut with_lines = Seismograph::new();
        with_lines.set_active_bucket_lines(3);
        let mut without_lines = Seismograph::new();
        for _ in 0..300 {
            with_lines.step();
            without_lines.step();
        }
        assert_eq!(with_lines.level() - without_lines.level(), 900);
    }

    #[test]
    fn awakening_lasts_2700_ticks_and_stops_lines_with_900_remaining() {
        let mut seismo = Seismograph::new();
        seismo.add_noise(NoiseSource::Custom(MAX_LEVEL));
        assert!(seismo.step().contains(&SeismoEvent::AwakeningStarted));
        assert_eq!(seismo.awakening_ticks_remaining(), Some(2_700));

        for tick in 1..=AWAKENING_DURATION_TICKS {
            let events = seismo.step();
            assert_eq!(seismo.relieve(8_000), 0);
            if tick == 1_800 {
                assert_eq!(events, vec![SeismoEvent::BucketLinesStopped]);
                assert!(seismo.are_bucket_lines_stopped());
                assert_eq!(seismo.awakening_ticks_remaining(), Some(900));
                assert_eq!(seismo.add_noise(NoiseSource::ActiveBucketLine), 0);
            } else if tick == AWAKENING_DURATION_TICKS {
                assert_eq!(events, vec![SeismoEvent::AwakeningEnded]);
                assert!(seismo.is_awakening_ended());
                assert!(!seismo.is_awakening());
            } else {
                assert!(events.is_empty());
            }
        }
    }
}
