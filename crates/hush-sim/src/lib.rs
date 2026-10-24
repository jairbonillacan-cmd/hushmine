//! Simulación determinista de HUSHMINE.
//!
//! Reglas en enteros y punto fijo (`Q16.16`), sin gráficos ni dependencias de
//! plataforma: el mismo código corre en el servidor, en el cliente y en
//! `hush-cli sim`. Incluye el catálogo de bloques (Convención A), el Sismógrafo
//! y el ciclo del Despertar, el registro de eventos de partida, el reloj de
//! ticks, el generador aleatorio determinista y el informe de simulación.

pub mod altar;
pub mod block;
pub mod events;
pub mod fixed;
mod report;
mod rng;
pub mod seismograph;
mod sim;

pub use altar::{Altar, AltarError};
pub use block::{BlockKind, InvalidBlockId};
pub use events::{EventLog, MatchEvent, MatchEventKind, Team};
pub use fixed::{Fixed, FixedError};
pub use report::SimReport;
pub use rng::Rng;
pub use seismograph::{NoiseSource, SeismoEvent, SeismoThreshold, Seismograph};
pub use sim::{MatchConfig, Simulation, TICK_RATE};

/// Ejecuta una partida completa sin gráficos y devuelve el informe.
pub fn run(config: MatchConfig) -> SimReport {
    Simulation::new(config).run_to_end()
}
