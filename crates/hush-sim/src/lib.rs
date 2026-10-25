//! Simulación determinista de HUSHMINE.
//!
//! Reglas en enteros y punto fijo: posiciones de minero en `i64` Q47.16 (16
//! bits de fracción, hasta ±7,5 millones de bloques) y velocidades y
//! escalares en Q16.16 ([`Fixed`]). Sin gráficos ni dependencias de
//! plataforma: el mismo código corre en el servidor, en el cliente y en
//! `hush-cli sim`. Incluye el catálogo de bloques (Convención A), el terreno
//! (con los tipos de `grisu-voxel`), el movimiento de los mineros, cavar y
//! poner, el Sismógrafo y el ciclo del Despertar, la economía y bolsas de
//! botín, el verificador de invariantes formales, el registro de eventos, el
//! reloj de ticks, el generador aleatorio determinista y el informe de
//! simulación.

mod actions;
pub mod altar;
pub mod block;
mod bot;
pub mod economy;
mod edit;
pub mod events;
pub mod fixed;
pub mod invariants;
pub mod loot;
mod player;
mod report;
mod rng;
pub mod seismograph;
mod sim;
mod sim_seismo;
mod sim_world;
pub mod story;
mod terrain;

pub use altar::{Altar, AltarError};
pub use block::{BlockKind, InvalidBlockId};
pub use bot::DiggerBot;
pub use economy::{DepositOutcome, Economy, EconomyError, EconomyResult, PlayerLoad};
pub use edit::{EditError, REACH, dig as try_dig, place as try_place};
pub use events::{EventLog, MatchEvent, MatchEventKind, Team};
pub use fixed::{Fixed, FixedError};
pub use loot::{GridPos, LootBag, LootContents, LootError, LootPickup, LootStore};
pub use player::{
    EYE_HEIGHT, HALF_WIDTH, HEIGHT, MoveInput, PlayerBody, SPRINT_PER_TICK, WALK_PER_TICK,
    spawn_point,
};
pub use report::SimReport;
pub use rng::Rng;
pub use seismograph::{NoiseSource, SeismoEvent, SeismoThreshold, Seismograph};
pub use sim::{BlockChange, Command, EditStats, MatchConfig, Rejection, Simulation, TICK_RATE};
pub use story::{Expedition, ItemId, Milestone, STORY_TICK_RATE, StoryConfig, StoryOrder};
pub use terrain::{TerrainConfig, generate as generate_terrain};

/// Ejecuta una partida completa sin gráficos y devuelve el informe.
pub fn run(config: MatchConfig) -> Result<SimReport, grisu_voxel::WorldError> {
    Ok(Simulation::new(config)?.run_to_end())
}
