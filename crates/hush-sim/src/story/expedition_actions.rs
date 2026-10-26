//! Órdenes del jugador en la expedición: picar, poner, fabricar,
//! interactuar con hornos y arcones, comer y atacar. Las valida y aplica
//! `expedition_tick` dentro del tick.

use grisu_voxel::AIR;

use super::crafting::{self, GRID};
use super::expedition::{Expedition, FurnaceSlot, StoryEventKind};
use super::inventory::{Container, ItemStack, Place};
use super::items::ItemId;
use super::mining::drops_item;
use crate::BlockKind;

impl Expedition {
    // ---- Minado progresivo ----

    pub(super) fn set_mining(&mut self, cell: Option<[i32; 3]>) {
        let Some(cell) = cell else {
            self.mining = super::expedition::MiningState::default();
            return;
        };
        if self.mining.cell != Some(cell) {
            self.mining.cell = Some(cell);
            self.mining.progress = 0;
            let block = BlockKind::from_id(self.world.get(cell[0], cell[1], cell[2]))
                .unwrap_or(BlockKind::Air);
            self.mining.block = block;
            self.mining.total = 0;
        }
    }

    pub(super) fn step_mining_tick(&mut self) {
        let Some(cell) = self.mining.cell else {
            return;
        };
        if !crate::edit::within_reach(self.body.eye(), cell) {
            self.mining.cell = None;
            return;
        }
        let block =
            BlockKind::from_id(self.world.get(cell[0], cell[1], cell[2])).unwrap_or(BlockKind::Air);
        if !block.is_diggable() || cell[1] < self.bands.bedrock {
            self.mining.cell = None;
            return;
        }
        if self.mining.block != block {
            self.mining.block = block;
            self.mining.progress = 0;
        }
        if self.mining.total == 0 {
            self.mining.total = super::mining::break_ticks(block, self.held_lite()).unwrap_or(1);
        }
        self.mining.progress += 1;
        self.record_noise(cell);
        if self.mining.progress < self.mining.total {
            return;
        }
        // Ruptura del bloque.
        self.mining.cell = None;
        self.mining.progress = 0;
        self.world.set(cell[0], cell[1], cell[2], AIR);
        let held = self.held_lite();
        if drops_item(block, held) {
            if let Some(item) = super::mining::drop_of(block)
                && self.inventory.add(ItemStack::new(item, 1)).is_some()
            {
                self.push_event(StoryEventKind::ItemLost(item.0));
            }
            self.push_event(StoryEventKind::Mined(block));
        } else {
            self.push_event(StoryEventKind::CrushedToDust(block));
        }
        if let Some(broken) = self.inventory.wear_held() {
            self.push_event(StoryEventKind::ToolBroke(broken.id.0));
        }
        // La grava profunda puede albergar un nido de alimañas.
        if block == BlockKind::Gravel && cell[1] < self.bands.crystalline {
            let roll = self.rng.next_u64() % 4;
            if roll == 0 {
                self.mobs.spawn_swarm(cell, 3);
                self.push_event(StoryEventKind::Swarmed);
            }
        }
        self.fluids.wake_around(cell);
        self.vitals.add_fatigue(super::vitals::FATIGUE_DIG);
    }

    pub(super) fn place_block(&mut self, cell: [i32; 3]) {
        let Some(held) = self.inventory.held() else {
            return;
        };
        let Some(block) = held.id.as_block() else {
            return;
        };
        if crate::edit::place(&mut self.world, &self.body, &[], cell, block).is_err() {
            return;
        }
        if self.inventory.consume(held.id, 1) {
            match block {
                BlockKind::Chest => {
                    self.chests.insert(cell, Container::chest());
                }
                BlockKind::Furnace => {
                    self.furnaces.insert(
                        cell,
                        crafting::FurnaceState::new(crafting::FurnaceKind::Stone),
                    );
                }
                BlockKind::Torch => {
                    self.milestones.torches_placed += 1;
                    self.push_event(StoryEventKind::TorchPlaced);
                }
                _ => {}
            }
            self.push_event(StoryEventKind::Placed(block));
        }
    }

    pub(super) fn craft(&mut self, grid: [[Option<u16>; GRID]; GRID], workbench: bool) {
        // Validar que el inventario tiene todos los ítems de la matriz.
        let ids = grid.map(|row| row.map(|cell| cell.map(ItemId)));
        for row in &ids {
            for id in row.iter().flatten() {
                if self.inventory.count_of(*id) < 1 {
                    return;
                }
            }
        }
        let Some(recipe) = crafting::match_recipe(&ids, workbench) else {
            return;
        };
        for row in &ids {
            for id in row.iter().flatten() {
                self.inventory.consume(*id, 1);
            }
        }
        let output = recipe.output;
        if self.inventory.add(output).is_some() {
            self.push_event(StoryEventKind::ItemLost(output.id.0));
        }
        self.push_event(StoryEventKind::Crafted(output.id.0));
    }

    pub(super) fn interact(&mut self, cell: [i32; 3]) {
        let block = self.world.get(cell[0], cell[1], cell[2]);
        let held = self.inventory.held();
        let held_id = held.map(|s| s.id);
        // Sellar válvula de gas con un tablón.
        if self.well.valves.contains(&cell)
            && block == BlockKind::Obsidian.id()
            && held_id.is_some_and(|id| id == ItemId::of_block(BlockKind::Plank))
            && self
                .inventory
                .consume(ItemId::of_block(BlockKind::Plank), 1)
        {
            self.world
                .set(cell[0], cell[1], cell[2], BlockKind::PropBeam.id());
            self.milestones.valves_sealed += 1;
            self.push_event(StoryEventKind::ValveSealed);
            return;
        }
        // Reparar engranaje con un lingote de hierro.
        if self.well.gears.contains(&cell)
            && held_id.is_some_and(|id| id.0 == super::items::IRON_INGOT)
            && self.inventory.consume(ItemId(super::items::IRON_INGOT), 1)
        {
            self.world
                .set(cell[0], cell[1], cell[2], BlockKind::PropBeam.id());
            self.milestones.gears_repaired += 1;
            self.push_event(StoryEventKind::GearRepaired);
            return;
        }
        // Activar el motor del ascensor.
        if cell == self.well.core && self.milestones.activate_elevator() {
            self.push_event(StoryEventKind::ElevatorActivated);
            self.push_event(StoryEventKind::SiegeStarted);
        }
    }

    pub(super) fn furnace_load(&mut self, cell: [i32; 3], place: Place) {
        let Some(&stack) = self.inventory.get(place).as_ref() else {
            return;
        };
        let mut one = stack;
        one.count = 1;
        let Some(furnace) = self.furnaces.get_mut(&cell) else {
            return;
        };
        if crafting::feed_furnace(furnace, &mut one) {
            self.inventory.consume(stack.id, 1);
        }
    }

    pub(super) fn furnace_take(&mut self, cell: [i32; 3], slot: FurnaceSlot) {
        let taken = self.furnaces.get_mut(&cell).and_then(|furnace| {
            let target = match slot {
                FurnaceSlot::Input => &mut furnace.input,
                FurnaceSlot::Fuel => &mut furnace.fuel,
                FurnaceSlot::Output => &mut furnace.output,
            };
            target.take()
        });
        if let Some(stack) = taken
            && self.inventory.add(stack).is_some()
        {
            // No cupo en el inventario: se devuelve al horno.
            self.push_event(StoryEventKind::ItemLost(stack.id.0));
            if let Some(furnace) = self.furnaces.get_mut(&cell) {
                let target = match slot {
                    FurnaceSlot::Input => &mut furnace.input,
                    FurnaceSlot::Fuel => &mut furnace.fuel,
                    FurnaceSlot::Output => &mut furnace.output,
                };
                *target = Some(stack);
            }
        }
    }

    pub(super) fn chest_put(&mut self, cell: [i32; 3], place: Place) {
        let Some(&stack) = self.inventory.get(place).as_ref() else {
            return;
        };
        let to_store = ItemStack {
            count: stack.count.min(stack.id.stack_max()),
            ..stack
        };
        let stored = match self.chests.get_mut(&cell).map(|chest| chest.push(to_store)) {
            None => return,
            Some(None) => to_store.count,
            Some(Some(back)) => to_store.count - back.count,
        };
        if stored > 0 {
            self.inventory.consume(stack.id, stored);
        }
    }

    pub(super) fn chest_take(&mut self, cell: [i32; 3], slot: u8) {
        let Some(chest) = self.chests.get_mut(&cell) else {
            return;
        };
        if usize::from(slot) >= chest.len() {
            return;
        }
        if let Some(stack) = chest.slots[usize::from(slot)].take()
            && self.inventory.add(stack).is_some()
        {
            chest.slots[usize::from(slot)] = Some(stack);
        }
    }

    pub(super) fn eat(&mut self) {
        let Some(held) = self.inventory.held() else {
            return;
        };
        if held.id.0 != super::items::RATION || !self.inventory.consume(held.id, 1) {
            return;
        }
        self.vitals
            .eat(super::items::RATION_FOOD, super::items::RATION_SATURATION);
    }

    pub(super) fn attack(&mut self, cell: [i32; 3]) {
        let held = self.inventory.held();
        let damage = held
            .filter(|s| s.id.tool_kind() == Some(super::items::ToolKind::Sword))
            .map_or(1, |s| s.id.tool_tier().unwrap_or(0) * 2 + 3);
        if let Some(died) = self.mobs.damage_at(cell, damage) {
            self.push_event(StoryEventKind::Mob(died));
        }
        self.inventory.wear_held();
    }
}
