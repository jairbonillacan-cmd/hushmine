//! Accesores de solo lectura de la expedición para el cliente, el HUD y las pruebas.

use super::*;

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
    pub fn teleport(&mut self, feet: [i64; 3]) {
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
        self.body.feet.iter().for_each(|&c| mix(c));
        self.body.velocity.iter().for_each(|&c| mix(c.into()));
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
        self.body.feet.map(cell_of)
    }

    /// Celda de la cabeza (para la respiración).
    pub fn head_cell(&self) -> [i32; 3] {
        let eye = self.body.eye();
        eye.map(cell_of)
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
    pub fn save_region(
        &self,
        path: &std::path::Path,
    ) -> Result<(), crate::story::region::RegionError> {
        crate::story::region::save_world(&self.world, path)
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
