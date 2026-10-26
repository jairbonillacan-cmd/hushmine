//! Autómata celular de fluidos: agua y magma.
//!
//! El agua se propaga en horizontal hasta 7 bloques reduciendo su nivel;
//! el magma, denso, hasta 3. El agua que cae sobre un manantial de magma
//! crea obsidiana; el choque lateral con magma en movimiento crea roca
//! (cobblestone). El magma quema la madera vecina.

use std::collections::BTreeMap;

use grisu_voxel::{AIR, VoxelWorld};
use serde::{Deserialize, Serialize};

use crate::BlockKind;

/// Nivel máximo de alcance del agua (el manantial es el nivel 0).
pub const WATER_SPREAD: u8 = 7;
/// Nivel máximo de alcance del magma.
pub const LAVA_SPREAD: u8 = 3;
/// Presupuesto de celdas actualizadas por tick.
pub const TICK_BUDGET: usize = 96;

/// Un fluido en una celda: tipo y nivel (0 = manantial).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fluid {
    pub magma: bool,
    pub level: u8,
}

/// Campo de fluidos activo del mundo. Solo guarda las celdas de fluido; el
/// bloque del mundo (`Water`/`Lava`) es la vista para render y colisión.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FluidField {
    cells: BTreeMap<[i32; 3], Fluid>,
    /// Cola de celdas pendientes de actualizar (FIFO).
    pending: Vec<[i32; 3]>,
}

/// Reacción litológica ocurrida al avanzar un tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reaction {
    /// Agua sobre manantial de magma: obsidiana abisal.
    Obsidian([i32; 3]),
    /// Agua contra magma en movimiento: roca cobblestone.
    Cobble([i32; 3]),
    /// El magma prendió un bloque de madera.
    WoodBurned([i32; 3]),
}

impl FluidField {
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn get(&self, cell: &[i32; 3]) -> Option<Fluid> {
        self.cells.get(cell).copied()
    }

    /// Registra un manantial (nivel 0) en una celda vacía.
    pub fn add_source(&mut self, world: &mut VoxelWorld, cell: [i32; 3], magma: bool) {
        self.set(world, cell, Fluid { magma, level: 0 });
    }

    /// Registra el fluido generado por el mundo inicial: bolsa de agua o
    /// lago de magma abiertos al excavar.
    pub fn seed_from_world(&mut self, world: &VoxelWorld) {
        let [sx, sy, sz] = world.size().map(|s| s as i32);
        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let block = world.get(x, y, z);
                    let fluid = match block {
                        b if b == BlockKind::Water.id() => Some(false),
                        b if b == BlockKind::Lava.id() => Some(true),
                        _ => None,
                    };
                    if let Some(magma) = fluid
                        && self.is_open(world, x, y, z)
                    {
                        self.cells
                            .entry([x, y, z])
                            .or_insert(Fluid { magma, level: 0 });
                    }
                }
            }
        }
    }

    /// Una celda de fluido del terreno cuenta como manantial si toca aire
    /// (está abierta a la excavación).
    fn is_open(&self, world: &VoxelWorld, x: i32, y: i32, z: i32) -> bool {
        const N: [[i32; 3]; 6] = [
            [1, 0, 0],
            [-1, 0, 0],
            [0, 1, 0],
            [0, -1, 0],
            [0, 0, 1],
            [0, 0, -1],
        ];
        N.iter()
            .any(|d| world.get(x + d[0], y + d[1], z + d[2]) == AIR)
    }

    fn set(&mut self, world: &mut VoxelWorld, cell: [i32; 3], fluid: Fluid) {
        let block = if fluid.magma {
            BlockKind::Lava.id()
        } else {
            BlockKind::Water.id()
        };
        self.cells.insert(cell, fluid);
        world.set(cell[0], cell[1], cell[2], block);
        self.pending.push(cell);
    }

    /// Avanza el autómata un tick con presupuesto limitado. Devuelve las
    /// reacciones litológicas producidas.
    pub fn step(&mut self, world: &mut VoxelWorld) -> Vec<Reaction> {
        let mut reactions = Vec::new();
        let mut budget = TICK_BUDGET;
        while budget > 0 && !self.pending.is_empty() {
            let cell = self.pending.remove(0);
            budget -= 1;
            let Some(fluid) = self.cells.get(&cell).copied() else {
                continue;
            };
            let spread_max = if fluid.magma {
                LAVA_SPREAD
            } else {
                WATER_SPREAD
            };

            // Reacciones: el agua que toca magma se litifica.
            if let Some(reaction) = self.react(world, cell, fluid) {
                reactions.push(reaction);
                continue;
            }

            // El magma prende la madera vecina.
            if fluid.magma {
                for d in NEIGHBOURS {
                    let n = [cell[0] + d[0], cell[1] + d[1], cell[2] + d[2]];
                    if is_wood(world.get(n[0], n[1], n[2])) {
                        world.set(n[0], n[1], n[2], AIR);
                        reactions.push(Reaction::WoodBurned(n));
                        self.pending.push(cell);
                    }
                }
            }

            // 1) Caída: el fluido baja si hay aire debajo.
            let below = [cell[0], cell[1] - 1, cell[2]];
            if world.get(below[0], below[1], below[2]) == AIR {
                self.set(
                    world,
                    below,
                    Fluid {
                        magma: fluid.magma,
                        level: 1,
                    },
                );
                continue;
            }
            // 2) Flujo horizontal: nace a nivel+1 hasta el alcance máximo,
            // con preferencia por los desniveles cercanos.
            if fluid.level >= spread_max {
                continue;
            }
            let child = Fluid {
                magma: fluid.magma,
                level: fluid.level + 1,
            };
            let mut downhill = Vec::new();
            let mut flat = Vec::new();
            for d in &NEIGHBOURS {
                let n = [cell[0] + d[0], cell[1] + d[1], cell[2] + d[2]];
                if world.get(n[0], n[1], n[2]) != AIR {
                    continue;
                }
                let under = [n[0], n[1] - 1, n[2]];
                if world.get(under[0], under[1], under[2]) == AIR {
                    downhill.push(n);
                } else {
                    flat.push(n);
                }
            }
            let targets = if downhill.is_empty() {
                &flat
            } else {
                &downhill
            };
            for n in targets.iter() {
                self.set(world, *n, child);
            }
        }
        reactions
    }

    /// Reacciones agua-magma en la celda `cell`.
    fn react(&mut self, world: &mut VoxelWorld, cell: [i32; 3], fluid: Fluid) -> Option<Reaction> {
        let water_at = |p: [i32; 3]| world.get(p[0], p[1], p[2]) == BlockKind::Water.id();
        let lava_at = |p: [i32; 3]| world.get(p[0], p[1], p[2]) == BlockKind::Lava.id();
        if fluid.magma {
            // ¿Agua cayendo sobre este magma?
            if water_at([cell[0], cell[1] + 1, cell[2]]) {
                self.cells.remove(&cell);
                world.set(cell[0], cell[1], cell[2], BlockKind::Obsidian.id());
                return Some(Reaction::Obsidian(cell));
            }
            // ¿Agua lateral contra magma en movimiento (nivel > 0)?
            if fluid.level > 0
                && NEIGHBOURS
                    .iter()
                    .any(|d| water_at([cell[0] + d[0], cell[1] + d[1], cell[2] + d[2]]))
            {
                self.cells.remove(&cell);
                world.set(cell[0], cell[1], cell[2], BlockKind::Rock.id());
                return Some(Reaction::Cobble(cell));
            }
            None
        } else {
            // Agua avanzando sobre un manantial de magma: la celda de abajo
            // o de al lado con lava nivel 0 se vuelve obsidiana.
            for d in &NEIGHBOURS {
                let n = [cell[0] + d[0], cell[1] + d[1], cell[2] + d[2]];
                if lava_at(n)
                    && self.get(&n).is_none_or(|f| f.level == 0)
                    && water_at([n[0], n[1] + 1, n[2]])
                {
                    self.cells.remove(&n);
                    world.set(n[0], n[1], n[2], BlockKind::Obsidian.id());
                    return Some(Reaction::Obsidian(n));
                }
            }
            None
        }
    }

    /// Al picar un bloque junto a un fluido, hay que reactivar el flujo.
    pub fn wake_around(&mut self, cell: [i32; 3]) {
        for d in &NEIGHBOURS {
            let n = [cell[0] + d[0], cell[1] + d[1], cell[2] + d[2]];
            if self.cells.contains_key(&n) {
                self.pending.push(n);
            }
        }
    }
}

const NEIGHBOURS: [[i32; 3]; 4] = [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]];

fn is_wood(block: u8) -> bool {
    matches!(
        BlockKind::from_id(block),
        Ok(BlockKind::Log
            | BlockKind::Plank
            | BlockKind::PropBeam
            | BlockKind::Workbench
            | BlockKind::Chest)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_floor() -> VoxelWorld {
        let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
        w.fill([0, 0, 0], [31, 3, 31], BlockKind::Rock.id());
        w
    }

    #[test]
    fn water_spreads_to_seven_blocks_and_stops() {
        let mut w = world_with_floor();
        let mut field = FluidField::default();
        field.add_source(&mut w, [16, 4, 16], false);
        for _ in 0..200 {
            field.step(&mut w);
        }
        // A ambos lados alcanza 7 bloques (niveles 1..7) y no más.
        assert_eq!(w.get(23, 4, 16), BlockKind::Water.id());
        assert_eq!(w.get(24, 4, 16), AIR);
        assert_eq!(w.get(9, 4, 16), BlockKind::Water.id());
        assert_eq!(w.get(8, 4, 16), AIR);
        // Niveles decrecientes.
        assert_eq!(field.get(&[17, 4, 16]).unwrap().level, 1);
        assert_eq!(field.get(&[23, 4, 16]).unwrap().level, 7);
    }

    #[test]
    fn water_falls_down_preferred_over_spreading() {
        let mut w = world_with_floor();
        // Hueco en el suelo al lado del manantial.
        w.set(17, 3, 16, AIR);
        w.set(17, 2, 16, AIR);
        let mut field = FluidField::default();
        field.add_source(&mut w, [16, 4, 16], false);
        for _ in 0..60 {
            field.step(&mut w);
        }
        // El agua cayó al hueco.
        assert_eq!(w.get(17, 3, 16), BlockKind::Water.id());
        assert_eq!(w.get(17, 2, 16), BlockKind::Water.id());
    }

    #[test]
    fn water_over_lava_source_makes_obsidian() {
        let mut w = world_with_floor();
        w.set(16, 4, 16, BlockKind::Lava.id());
        let mut field = FluidField::default();
        field.add_source(&mut w, [16, 5, 16], false);
        let reactions: Vec<Reaction> = (0..10).flat_map(|_| field.step(&mut w)).collect();
        assert!(reactions.contains(&Reaction::Obsidian([16, 4, 16])));
        assert_eq!(w.get(16, 4, 16), BlockKind::Obsidian.id());
    }

    #[test]
    fn magma_spreads_only_three_and_burns_wood() {
        let mut w = world_with_floor();
        let mut field = FluidField::default();
        field.add_source(&mut w, [16, 4, 16], true);
        w.set(20, 4, 16, BlockKind::Plank.id());
        for _ in 0..200 {
            field.step(&mut w);
        }
        assert_eq!(w.get(19, 4, 16), BlockKind::Lava.id());
        assert_eq!(w.get(20, 4, 16), AIR); // la madera se quemó
        assert_eq!(w.get(21, 4, 16), AIR); // y no pasó de 3
    }

    #[test]
    fn digging_next_to_a_pool_wakes_the_flow() {
        let mut w = world_with_floor();
        let mut field = FluidField::default();
        field.add_source(&mut w, [10, 4, 10], false);
        for _ in 0..100 {
            field.step(&mut w);
        }
        // Se estancó.
        assert!(field.step(&mut w).is_empty());
        // Al picar la celda contigua, el agua vuelve a fluir.
        w.set(11, 4, 10, AIR);
        w.set(12, 4, 10, AIR);
        field.wake_around([11, 4, 10]);
        let mut moved = false;
        for _ in 0..50 {
            if w.get(12, 4, 10) == BlockKind::Water.id() {
                moved = true;
                break;
            }
            field.step(&mut w);
        }
        assert!(moved);
    }
}
