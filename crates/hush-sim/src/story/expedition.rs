//! Expedición: la simulación determinista del modo historia.
//!
//! Reúne el mundo estratificado, la fisiología del minero, el inventario,
//! el ciclo diurno, los hornos y arcones, los fluidos, los enemigos y el
//! árbol de hitos. Todo avanza a 20 Hz con órdenes validadas y eventos; el
//! estado solo contiene enteros.

use std::collections::BTreeMap;

use grisu_voxel::{FIXED_ONE, VoxelWorld, WorldError, cell_of};
use serde::{Deserialize, Serialize};

use super::crafting::{FurnaceKind, FurnaceState};
use super::daylight::DayCycle;
use super::fluids::{FluidField, Reaction};
use super::inventory::{Container, Inventory, ItemStack, Place};
use super::items::{ItemId, RATION};
use super::milestones::Milestones;
use super::mining::{ItemStackLite, break_ticks, fracture_stage};
use super::mobs::{MobEvent, Mobs};
use super::strata::{AncestralWell, Bands, StoryTerrain, generate as generate_story};
use super::vitals::Vitals;
use crate::rng::Rng;
use crate::{BlockKind, MoveInput, PlayerBody};

/// Órdenes que el cliente envía a la expedición (una tanda por tick).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryOrder {
    Move(MoveInput),
    Sneak(bool),
    /// Mantener el pico sobre una celda (`None` suelta).
    Mine(Option<[i32; 3]>),
    /// Colocar el bloque empuñado en una celda.
    Place([i32; 3]),
    /// Elegir casilla de la barra rápida (0..=8).
    Select(u8),
    /// Mover una pila dentro del inventario.
    Swap(Place, Place),
    /// Fabricar con una matriz de ids (`None` = casilla vacía). Con
    /// `workbench = true` admite la matriz 3×3 completa.
    Craft {
        grid: [[Option<u16>; 3]; 3],
        workbench: bool,
    },
    /// Interactuar con el bloque apuntado (banco, horno, arcón, válvula,
    /// engranaje, motor).
    Interact([i32; 3]),
    /// Introducir al horno la pila de una casilla del inventario.
    FurnaceLoad([i32; 3], Place),
    /// Retirar del horno la casilla indicada.
    FurnaceTake([i32; 3], FurnaceSlot),
    /// Guardar en un arcón la pila de una casilla del inventario.
    ChestPut([i32; 3], Place),
    /// Retirar de un arcón la casilla indicada.
    ChestTake([i32; 3], u8),
    /// Comer la ración empuñada.
    Eat,
    /// Atacar con la herramienta empuñada hacia una celda.
    Attack([i32; 3]),
}

/// Casilla del horno.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FurnaceSlot {
    Input,
    Fuel,
    Output,
}

/// Evento de un tick de expedición.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryEvent {
    pub tick: u32,
    pub kind: StoryEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoryEventKind {
    Mined(BlockKind),
    /// El bloque picado no suelta material (herramienta insuficiente).
    CrushedToDust(BlockKind),
    Placed(BlockKind),
    Crafted(u16),
    ToolBroke(u16),
    Smelted(u16),
    TorchPlaced,
    ValveSealed,
    GearRepaired,
    ElevatorActivated,
    SiegeStarted,
    /// Fin del asedio con victoria.
    Victory,
    /// El minero ha desfallecido.
    Died,
    Damaged {
        amount: u8,
        cause: DamageCause,
    },
    Mob(MobEvent),
    Fluid(Reaction),
    MilestoneReached(u8),
    /// La colocación o el inventario desbordaron: el ítem se pierde.
    ItemLost(u16),
    Swarmed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageCause {
    Fall,
    Drowning,
    Starvation,
    Lava,
    Mob,
    Projectile,
}

/// Configuración de la expedición.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryConfig {
    pub seed: u64,
    pub terrain: StoryTerrain,
    /// Sin enemigos (pruebas y modo tranquilo).
    pub peaceful: bool,
}

impl Default for StoryConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            terrain: StoryTerrain::default(),
            peaceful: false,
        }
    }
}

/// Estado de la excavación en curso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiningState {
    pub cell: Option<[i32; 3]>,
    pub progress: u32,
    pub total: u32,
    pub block: BlockKind,
}

impl Default for MiningState {
    fn default() -> Self {
        Self {
            cell: None,
            progress: 0,
            total: 0,
            block: BlockKind::Air,
        }
    }
}

/// Simulación del modo historia.
pub struct Expedition {
    pub(crate) config: StoryConfig,
    pub(crate) tick: u32,
    pub(crate) rng: Rng,
    pub(crate) world: VoxelWorld,
    pub(crate) bands: Bands,
    pub(crate) well: AncestralWell,
    pub(crate) spawn: [i32; 3],
    /// Se lee desde el cliente (reaparición) y las pruebas.
    #[allow(dead_code)]
    spawn_marker: (),
    pub(crate) body: PlayerBody,
    pub(crate) vitals: Vitals,
    pub(crate) inventory: Inventory,
    pub(crate) sneak: bool,
    pub(crate) clock: DayCycle,
    pub(crate) mining: MiningState,
    pub(crate) furnaces: BTreeMap<[i32; 3], FurnaceState>,
    pub(crate) chests: BTreeMap<[i32; 3], Container>,
    pub(crate) fluids: FluidField,
    pub(crate) mobs: Mobs,
    pub(crate) milestones: Milestones,
    /// Último ruido emitido (posición en bloques y tick).
    pub(crate) noise: Option<([i32; 3], u32)>,
    pub(crate) pending_input: MoveInput,
    pub(crate) events: Vec<StoryEvent>,
    pub(crate) dead: bool,
    pub(crate) state_hash: u64,
}

impl Expedition {
    /// Crea la expedición: genera el mundo y prepara al minero.
    pub fn new(config: StoryConfig) -> Result<Self, WorldError> {
        let generated = generate_story(config.seed, &config.terrain)?;
        let world = generated.world;
        let mut furnaces = BTreeMap::new();
        let mut chests = BTreeMap::new();
        // Los arcones de las galerías abandonadas llegan con suministros.
        let mut loot_rng = Rng::new(config.seed ^ 0x10_075);
        let [sx, sy, sz] = world.size().map(|s| s as i32);
        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let cell = [x, y, z];
                    match BlockKind::from_id(world.get(x, y, z)) {
                        Ok(BlockKind::Chest) => {
                            let mut chest = Container::chest();
                            seed_loot(&mut loot_rng, &mut chest);
                            chests.insert(cell, chest);
                        }
                        Ok(BlockKind::Furnace) => {
                            furnaces.insert(cell, FurnaceState::new(FurnaceKind::Stone));
                        }
                        _ => {}
                    }
                }
            }
        }
        let mut fluids = FluidField::default();
        fluids.seed_from_world(&world);
        let feet = [
            i64::from(generated.spawn[0]) * FIXED_ONE + FIXED_ONE / 2,
            i64::from(generated.spawn[1]) * FIXED_ONE,
            i64::from(generated.spawn[2]) * FIXED_ONE + FIXED_ONE / 2,
        ];
        Ok(Self {
            rng: Rng::new(config.seed ^ 0x5EED_0000_0000_0E77),
            config,
            tick: 0,
            world,
            bands: generated.bands,
            well: generated.well,
            spawn: generated.spawn,
            spawn_marker: (),
            body: PlayerBody::new(feet),
            vitals: Vitals::default(),
            inventory: Inventory::new(),
            sneak: false,
            clock: DayCycle::default(),
            mining: MiningState::default(),
            furnaces,
            chests,
            fluids,
            mobs: Mobs::default(),
            milestones: Milestones::default(),
            noise: None,
            pending_input: MoveInput::default(),
            events: Vec::new(),
            dead: false,
            state_hash: 0x0FC0_FFEE_0000_BEEF,
        })
    }
}

/// Botín inicial de un arcón de galería abandonada.
fn seed_loot(rng: &mut Rng, chest: &mut Container) {
    use super::items::{COAL, TORCH_ITEM};
    let plans: [(u16, u16); 4] = [
        (TORCH_ITEM, 2 + rng.below(4) as u16),
        (COAL, 2 + rng.below(5) as u16),
        (
            ItemId::of_block(BlockKind::Plank).0,
            1 + rng.below(6) as u16,
        ),
        (RATION, rng.below(3) as u16),
    ];
    for (id, count) in plans {
        if count > 0 {
            chest.push(ItemStack::new(ItemId(id), count));
        }
    }
}

#[path = "expedition_access.rs"]
mod access;
pub use access::emission_of;
