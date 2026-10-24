//! Colisión de cajas alineadas contra la cuadrícula, en punto fijo.
//!
//! Es la misma en el cliente (predicción) y en el servidor, así que solo usa
//! enteros: [`FIXED_ONE`] unidades equivalen a un bloque.

use crate::VoxelWorld;

/// Unidades de punto fijo por bloque.
pub const FIXED_ONE: i32 = 1024;

/// Caja alineada con los ejes, en punto fijo. Ocupa `[min, max)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Aabb {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl Aabb {
    /// Caja con los pies en `feet` (centro de la base) y el tamaño dado.
    pub fn from_feet(feet: [i32; 3], half_width: i32, height: i32) -> Self {
        Self {
            min: [feet[0] - half_width, feet[1], feet[2] - half_width],
            max: [feet[0] + half_width, feet[1] + height, feet[2] + half_width],
        }
    }

    pub fn feet(&self) -> [i32; 3] {
        [
            (self.min[0] + self.max[0]) / 2,
            self.min[1],
            (self.min[2] + self.max[2]) / 2,
        ]
    }

    fn translated(mut self, axis: usize, d: i32) -> Self {
        self.min[axis] += d;
        self.max[axis] += d;
        self
    }

    /// ¿Toca algún bloque sólido o los límites del mapa?
    pub fn intersects_world(&self, world: &VoxelWorld) -> bool {
        let lo = self.min.map(|c| c.div_euclid(FIXED_ONE));
        let hi = self.max.map(|c| (c - 1).div_euclid(FIXED_ONE));
        (lo[1]..=hi[1]).any(|y| {
            (lo[2]..=hi[2]).any(|z| (lo[0]..=hi[0]).any(|x| world.blocks_movement(x, y, z)))
        })
    }

    /// ¿Ocupa la caja alguna parte de la celda `cell`?
    pub fn overlaps_cell(&self, cell: [i32; 3]) -> bool {
        (0..3).all(|i| self.min[i] < (cell[i] + 1) * FIXED_ONE && self.max[i] > cell[i] * FIXED_ONE)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveResult {
    pub aabb: Aabb,
    /// Ejes en los que el movimiento se detuvo contra un bloque.
    pub blocked: [bool; 3],
}

/// Mueve `aabb` según `delta`, eje por eje (Y, X, Z), deteniéndolo contra los
/// bloques sólidos y los lados y el fondo del mapa. Solo comprueba las celdas en las que entra, así que una
/// caja que ya solapaba un bloque puede salir de él.
pub fn move_aabb(world: &VoxelWorld, aabb: Aabb, delta: [i32; 3]) -> MoveResult {
    let mut aabb = aabb;
    let mut blocked = [false; 3];
    for axis in [1, 0, 2] {
        let (moved, hit) = sweep_axis(world, aabb, axis, delta[axis]);
        aabb = moved;
        blocked[axis] = hit;
    }
    MoveResult { aabb, blocked }
}

fn sweep_axis(world: &VoxelWorld, aabb: Aabb, axis: usize, d: i32) -> (Aabb, bool) {
    if d == 0 {
        return (aabb, false);
    }
    let [a, b] = match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let range = |lo: i32, hi: i32| lo.div_euclid(FIXED_ONE)..=(hi - 1).div_euclid(FIXED_ONE);
    let layer_solid = |layer: i32| {
        range(aabb.min[a], aabb.max[a]).any(|ia| {
            range(aabb.min[b], aabb.max[b]).any(|ib| {
                let mut cell = [0; 3];
                cell[axis] = layer;
                cell[a] = ia;
                cell[b] = ib;
                world.blocks_movement(cell[0], cell[1], cell[2])
            })
        })
    };

    if d > 0 {
        let first = (aabb.max[axis] - 1).div_euclid(FIXED_ONE) + 1;
        let last = (aabb.max[axis] + d - 1).div_euclid(FIXED_ONE);
        for layer in first..=last {
            if layer_solid(layer) {
                let allowed = layer * FIXED_ONE - aabb.max[axis];
                return (aabb.translated(axis, allowed), true);
            }
        }
    } else {
        let first = aabb.min[axis].div_euclid(FIXED_ONE) - 1;
        let last = (aabb.min[axis] + d).div_euclid(FIXED_ONE);
        for layer in (last..=first).rev() {
            if layer_solid(layer) {
                let allowed = (layer + 1) * FIXED_ONE - aabb.min[axis];
                return (aabb.translated(axis, allowed), true);
            }
        }
    }
    (aabb.translated(axis, d), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: i32 = FIXED_ONE;

    fn floor_world() -> VoxelWorld {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([0, 0, 0], [15, 0, 15], 1);
        w
    }

    fn player_at(x: i32, y: i32, z: i32) -> Aabb {
        Aabb::from_feet([x, y, z], 3 * ONE / 10, 17 * ONE / 10)
    }

    #[test]
    fn falls_onto_the_floor_and_stops_exactly() {
        let w = floor_world();
        let start = player_at(5 * ONE, 4 * ONE, 5 * ONE);
        let r = move_aabb(&w, start, [0, -10 * ONE, 0]);
        assert!(r.blocked[1]);
        assert_eq!(r.aabb.min[1], ONE);
        assert!(!r.aabb.intersects_world(&w));
    }

    #[test]
    fn walls_stop_horizontal_movement() {
        let mut w = floor_world();
        w.fill([8, 1, 0], [8, 3, 15], 1);
        let start = player_at(5 * ONE, ONE, 5 * ONE);
        let r = move_aabb(&w, start, [5 * ONE, 0, 0]);
        assert!(r.blocked[0] && !r.blocked[1]);
        assert_eq!(r.aabb.max[0], 8 * ONE);
        // Hacia el otro lado no hay pared.
        let r = move_aabb(&w, start, [-ONE, 0, 0]);
        assert_eq!(r.blocked, [false; 3]);
    }

    #[test]
    fn slides_along_a_wall() {
        let mut w = floor_world();
        w.fill([8, 1, 0], [8, 3, 15], 1);
        let start = player_at(7 * ONE, ONE, 5 * ONE);
        let r = move_aabb(&w, start, [ONE, 0, 2 * ONE]);
        assert!(r.blocked[0]);
        assert_eq!(r.aabb.feet()[2], 7 * ONE);
    }

    #[test]
    fn ceiling_blocks_jump() {
        let mut w = floor_world();
        w.fill([0, 3, 0], [15, 3, 15], 1);
        let start = player_at(5 * ONE, ONE, 5 * ONE);
        let r = move_aabb(&w, start, [0, ONE, 0]);
        assert!(r.blocked[1]);
        assert_eq!(r.aabb.max[1], 3 * ONE);
    }

    #[test]
    fn map_sides_and_bottom_are_walls_but_sky_is_open() {
        let w = floor_world();
        let start = player_at(ONE, ONE, 5 * ONE);
        let r = move_aabb(&w, start, [-5 * ONE, 0, 0]);
        assert!(r.blocked[0]);
        assert_eq!(r.aabb.min[0], 0);
        let r = move_aabb(&w, player_at(15 * ONE, ONE, 5 * ONE), [5 * ONE, 0, 0]);
        assert!(r.blocked[0]);
        assert_eq!(r.aabb.max[0], 16 * ONE);
        let r = move_aabb(&w, start, [0, 30 * ONE, 0]);
        assert!(!r.blocked[1], "el cielo debería estar abierto");
        let empty = VoxelWorld::new([16, 16, 16]).unwrap();
        let r = move_aabb(
            &empty,
            player_at(8 * ONE, 2 * ONE, 8 * ONE),
            [0, -9 * ONE, 0],
        );
        assert!(r.blocked[1]);
        assert_eq!(r.aabb.min[1], 0);
    }

    #[test]
    fn overlaps_cell_matches_bounds() {
        let aabb = player_at(5 * ONE, ONE, 5 * ONE);
        assert!(aabb.overlaps_cell([5, 1, 5]));
        assert!(aabb.overlaps_cell([4, 2, 4]));
        assert!(!aabb.overlaps_cell([5, 0, 5]));
        assert!(!aabb.overlaps_cell([6, 1, 5]));
    }

    #[test]
    fn never_ends_inside_a_block() {
        // Movimientos pseudoaleatorios en una cueva irregular.
        let mut w = floor_world();
        for i in 0..40 {
            w.set((i * 7) % 16, 1 + (i * 3) % 4, (i * 5) % 16, 1);
        }
        let mut aabb = player_at(8 * ONE + 512, ONE, 8 * ONE + 512);
        w.fill([7, 1, 7], [9, 3, 9], 0);
        let mut seed: u32 = 12345;
        for _ in 0..2000 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let d = [
                ((seed >> 8) % 700) as i32 - 350,
                ((seed >> 12) % 700) as i32 - 400,
                ((seed >> 16) % 700) as i32 - 350,
            ];
            aabb = move_aabb(&w, aabb, d).aabb;
            assert!(!aabb.intersects_world(&w), "dentro de un bloque: {aabb:?}");
        }
    }
}
