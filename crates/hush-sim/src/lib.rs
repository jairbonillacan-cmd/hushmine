//! Simulación de HUSHMINE.
//!
//! Reglas en enteros y punto fijo, sin gráficos ni dependencias de
//! plataforma: el mismo código corre en el servidor, en el cliente y en
//! `hush-cli sim`. En el hito 0 solo existe el esqueleto: reloj de ticks,
//! generador aleatorio determinista e informe.

mod report;
mod rng;
mod sim;

pub use report::SimReport;
pub use rng::Rng;
pub use sim::{MatchConfig, Simulation, TICK_RATE};

/// Ejecuta una partida completa sin gráficos y devuelve el informe.
pub fn run(config: MatchConfig) -> SimReport {
    Simulation::new(config).run_to_end()
}
