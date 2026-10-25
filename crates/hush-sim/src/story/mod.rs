//! Modo historia, aventura y supervivencia: adaptación del documento
//! maestro de diseño al motor Grisú.
//!
//! La simulación corre a 20 Hz con enteros y punto fijo: ítems e
//! inventario, fabricación matricial y horno, fisiología, ciclo diurno con
//! luz dual, estratos geológicos, minado por dureza y tiers, fluidos,
//! bestiario con IA determinista, hitos narrativos con el clímax del Pozo
//! Ancestral y persistencia en regiones `.hush`.

pub mod crafting;
pub mod daylight;
pub mod expedition;
mod expedition_tick;
pub mod fluids;
pub mod inventory;
pub mod items;
pub mod milestones;
pub mod mining;
pub mod mobs;
pub mod noise;
pub mod path;
pub mod region;
pub mod strata;
pub mod vitals;

pub use crafting::{FurnaceKind, FurnaceState, GRID, Recipe, SMELT_TICKS, recipes};
pub use daylight::{DAY_TICKS, DayCycle, Phase};
pub use expedition::{
    DamageCause, Expedition, FurnaceSlot, MiningState, StoryConfig, StoryEvent, StoryEventKind,
    StoryOrder, emission_of,
};
pub use fluids::{FluidField, Reaction};
pub use inventory::{Container, HOPPER_INTERVAL_TICKS, Inventory, ItemStack, Place, Slot};
pub use items::{ItemId, ToolKind, tool_item};
pub use milestones::{Milestone, Milestones, SIEGE_TICKS};
pub use mining::{FRACTURE_STAGES, break_ticks, drop_of, hardness_q10, required_tier};
pub use mobs::{Mob, MobEvent, MobKind, Mobs, Projectile, STORY_TICK_RATE};
pub use strata::{Bands, StoryTerrain, Stratum, generate as generate_terrain};
pub use vitals::Vitals;

#[cfg(test)]
mod expedition_tests;
