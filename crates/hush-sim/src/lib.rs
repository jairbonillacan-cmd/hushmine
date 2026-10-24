//! Simulación de HUSHMINE.
//!
//! Reglas en enteros y punto fijo, sin gráficos ni dependencias de
//! plataforma: el mismo código corre en el servidor, en el cliente y en
//! `hush-cli sim`. El terreno usa los tipos de `grisu-voxel`.

mod block;
mod bot;
mod edit;
mod player;
mod report;
mod rng;
mod sim;
mod terrain;

pub use block::Block;
pub use bot::DiggerBot;
pub use edit::{EditError, REACH, dig as try_dig, place as try_place};
pub use player::{EYE_HEIGHT, HALF_WIDTH, HEIGHT, MoveInput, PlayerBody, spawn_point};
pub use report::SimReport;
pub use rng::Rng;
pub use sim::{BlockChange, Command, EditStats, MatchConfig, Rejection, Simulation, TICK_RATE};
pub use terrain::{TerrainConfig, generate as generate_terrain};

/// Ejecuta una partida completa sin gráficos y devuelve el informe.
pub fn run(config: MatchConfig) -> Result<SimReport, grisu_voxel::WorldError> {
    Ok(Simulation::new(config)?.run_to_end())
}
