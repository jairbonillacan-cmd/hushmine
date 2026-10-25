//! Expedición: la simulación determinista del modo historia.
//!
//! Reúne el mundo estratificado, la fisiología del minero, el inventario,
//! el ciclo diurno, los hornos y arcones, los fluidos, los enemigos y el
//! árbol de hitos. Todo avanza a 20 Hz con órdenes validadas y eventos; el
//! estado solo contiene enteros.

use std::collections::BTreeMap;

use grisu_voxel::{FIXED_ONE, VoxelWorld, WorldError};
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
            generated.spawn[0] * FIXED_ONE + FIXED_ONE / 2,
            generated.spawn[1] * FIXED_ONE,
            generated.spawn[2] * FIXED_ONE + FIXED_ONE / 2,
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

// Accesores de solo lectura para el cliente.
impl Expedition {
    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn world(&self) -> &VoxelWorld {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut VoxelWorld {
        &mut self.world
    }

    pub fn body(&self) -> &PlayerBody {
        &self.body
    }

    pub fn vitals(&self) -> &Vitals {
        &self.vitals
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }

    pub fn clock(&self) -> &DayCycle {
        &self.clock
    }

    pub fn bands(&self) -> &Bands {
        &self.bands
    }

    pub fn well(&self) -> &AncestralWell {
        &self.well
    }

    pub fn milestones(&self) -> &Milestones {
        &self.milestones
    }

    pub fn milestones_mut(&mut self) -> &mut Milestones {
        &mut self.milestones
    }

    pub fn furnaces_mut(&mut self) -> &mut BTreeMap<[i32; 3], FurnaceState> {
        &mut self.furnaces
    }

    /// Traslada al minero a una posición (para pruebas y reaparición).
    /// Reinicia el rastreo de caída para no aplicar daño fantasma.
    pub fn teleport(&mut self, feet: [i32; 3]) {
        self.body = PlayerBody::new(feet);
        self.vitals.stop_fall();
    }

    pub fn mining(&self) -> &MiningState {
        &self.mining
    }

    pub fn mobs(&self) -> &Mobs {
        &self.mobs
    }

    pub fn furnaces(&self) -> &BTreeMap<[i32; 3], FurnaceState> {
        &self.furnaces
    }

    pub fn chests(&self) -> &BTreeMap<[i32; 3], Container> {
        &self.chests
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }

    /// Punto de aparición en la superficie (bloques).
    pub fn spawn(&self) -> [i32; 3] {
        self.spawn
    }

    pub fn config(&self) -> &StoryConfig {
        &self.config
    }

    /// Huella completa del estado (mundo incluido); dos expediciones con
    /// las mismas órdenes deben dar la misma huella.
    pub fn state_hash(&self) -> u64 {
        let mut hash = self.state_hash ^ self.world.content_hash();
        let mut mix = |v: i64| {
            hash = (hash ^ (v as u64)).wrapping_mul(0x0000_0100_0000_01B3);
        };
        self.body
            .feet
            .iter()
            .chain(&self.body.velocity)
            .for_each(|&c| mix(c.into()));
        mix(i64::from(self.vitals.health) << 8 | i64::from(self.vitals.food));
        mix(i64::from(self.milestones.reached));
        mix(self.mobs.list.len() as i64);
        for mob in &self.mobs.list {
            mob.cell.iter().for_each(|&c| mix(c.into()));
        }
        hash
    }

    /// Celda de bloques donde están los pies.
    pub fn feet_cell(&self) -> [i32; 3] {
        self.body.feet.map(|c| c.div_euclid(FIXED_ONE))
    }

    /// Celda de la cabeza (para la respiración).
    pub fn head_cell(&self) -> [i32; 3] {
        let eye = self.body.eye();
        eye.map(|c| c.div_euclid(FIXED_ONE))
    }

    /// Ítem empuñado en versión ligera para el minado.
    pub fn held_lite(&self) -> Option<ItemStackLite> {
        self.inventory
            .held()
            .map(|stack| ItemStackLite::of(stack.id))
    }

    /// Fase de fractura del bloque en minado (0..=10).
    pub fn fracture(&self) -> u8 {
        match self.mining.cell {
            Some(_) if self.mining.total > 0 => {
                fracture_stage(self.mining.progress, self.mining.total)
            }
            _ => 0,
        }
    }

    /// Eventos del último tick.
    pub fn take_events(&mut self) -> Vec<StoryEvent> {
        std::mem::take(&mut self.events)
    }

    pub(crate) fn push_event(&mut self, kind: StoryEventKind) {
        self.events.push(StoryEvent {
            tick: self.tick,
            kind,
        });
    }

    /// Ticks de rotura estimados con la herramienta actual (para la UI).
    pub fn estimate_break_ticks(&self, block: BlockKind) -> Option<u32> {
        break_ticks(block, self.held_lite())
    }

    /// Guarda el mundo en un archivo de región `.hush`.
    pub fn save_region(&self, path: &std::path::Path) -> Result<(), super::region::RegionError> {
        super::region::save_world(&self.world, path)
    }

    /// Luz de cielo de una celda (columna abierta = 15).
    pub fn sky_light_at(&self, cell: [i32; 3]) -> u8 {
        self.sky_light(cell)
    }

    /// Luz de bloque de una celda (emisores cercanos por BFS local).
    pub fn block_light_at(&self, cell: [i32; 3]) -> u8 {
        self.block_light(cell)
    }

    /// Luz efectiva de una celda para reglas de juego (no para render):
    /// máximo entre el cielo modulado por la hora y la luz de bloque de los
    /// emisores cercanos.
    pub fn game_light(&self, cell: [i32; 3]) -> u8 {
        let sky = self.sky_light(cell);
        let day = self.clock.sky_factor_q8();
        let sky_effective = (u16::from(sky) * day / 256) as u8;
        sky_effective.max(self.block_light(cell))
    }

    /// Luz de cielo bruta de una celda: 15 si la columna está abierta.
    fn sky_light(&self, cell: [i32; 3]) -> u8 {
        let [_, sy, _] = self.world.size().map(|s| s as i32);
        let open = (cell[1] + 1..sy).all(|y| !self.world.is_solid(cell[0], y, cell[2]));
        if open { 15 } else { 0 }
    }

    /// Luz de bloque por BFS local desde los emisores cercanos.
    fn block_light(&self, cell: [i32; 3]) -> u8 {
        use std::collections::VecDeque;
        const RADIUS: i32 = 8;
        let mut best = 0u8;
        let mut queue = VecDeque::from([cell]);
        let mut visited = std::collections::BTreeSet::from([cell]);
        let mut level = 0u8;
        while !queue.is_empty() && i32::from(level) < RADIUS {
            let mut next = VecDeque::new();
            for p in queue {
                for d in [
                    [1, 0, 0],
                    [-1, 0, 0],
                    [0, 1, 0],
                    [0, -1, 0],
                    [0, 0, 1],
                    [0, 0, -1],
                ] {
                    let q = [p[0] + d[0], p[1] + d[1], p[2] + d[2]];
                    if (q[0] - cell[0]).abs() > RADIUS
                        || (q[1] - cell[1]).abs() > RADIUS
                        || (q[2] - cell[2]).abs() > RADIUS
                        || !visited.insert(q)
                    {
                        continue;
                    }
                    let block = self.world.get(q[0], q[1], q[2]);
                    let emission = emission_of(block);
                    if emission > 0 {
                        best = best.max(emission.saturating_sub(level + 1));
                    }
                    if block == grisu_voxel::AIR {
                        next.push_back(q);
                    }
                }
            }
            queue = next;
            level += 1;
        }
        best
    }
}

/// Emisión de luz de cada bloque para las reglas del modo historia.
pub fn emission_of(block: u8) -> u8 {
    match BlockKind::from_id(block) {
        Ok(BlockKind::Torch) => 14,
        Ok(BlockKind::Lava) => 15,
        _ => 0,
    }
}
