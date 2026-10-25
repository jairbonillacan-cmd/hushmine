//! Colisión de cajas alineadas contra la cuadrícula, en punto fijo.
//!
//! Es la misma en el cliente (predicción) y en el servidor, así que solo usa
//! enteros. Las posiciones son `i64` con 16 bits de fracción (Q47.16):
//! [`FIXED_ONE`] unidades equivalen a un bloque y el rango llega de sobra a
//! los ±7,5 millones de bloques del mundo grande (ADR 0002).

use crate::Voxels;

/// Bits de fracción de las posiciones.
pub const FIXED_SHIFT: u32 = 16;
/// Unidades de punto fijo por bloque.
pub const FIXED_ONE: i64 = 1 << FIXED_SHIFT;

/// Celda que contiene la coordenada `c` (redondea hacia −∞).
#[inline]
pub fn cell_of(c: i64) -> i32 {
    (c >> FIXED_SHIFT) as i32
}

/// Caja alineada con los ejes, en punto fijo. Ocupa `[min, max)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Aabb {
    pub min: [i64; 3],
    pub max: [i64; 3],
}

impl Aabb {
    /// Caja con los pies en `feet` (centro de la base) y el tamaño dado.
    pub fn from_feet(feet: [i64; 3], half_width: i64, height: i64) -> Self {
        Self {
            min: [feet[0] - half_width, feet[1], feet[2] - half_width],
            max: [feet[0] + half_width, feet[1] + height, feet[2] + half_width],
        }
    }

    pub fn feet(&self) -> [i64; 3] {
        [
            (self.min[0] + self.max[0]).div_euclid(2),
            self.min[1],
            (self.min[2] + self.max[2]).div_euclid(2),
        ]
    }

    fn translated(mut self, axis: usize, d: i64) -> Self {
        self.min[axis] += d;
        self.max[axis] += d;
        self
    }

    /// ¿Toca algún bloque que impida moverse (sólido o límite del mundo)?
    pub fn intersects_world<W: Voxels + ?Sized>(&self, world: &W) -> bool {
        let lo = self.min.map(cell_of);
        let hi = self.max.map(|c| cell_of(c - 1));
        (lo[1]..=hi[1]).any(|y| {
            (lo[2]..=hi[2]).any(|z| (lo[0]..=hi[0]).any(|x| world.blocks_movement(x, y, z)))
        })
    }

    /// ¿Ocupa la caja alguna parte de la celda `cell`?
    pub fn overlaps_cell(&self, cell: [i32; 3]) -> bool {
        (0..3).all(|i| {
            let start = i64::from(cell[i]) << FIXED_SHIFT;
            self.min[i] < start + FIXED_ONE && self.max[i] > start
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveResult {
    pub aabb: Aabb,
    /// Ejes en los que el movimiento se detuvo contra un bloque.
    pub blocked: [bool; 3],
}

/// Mueve `aabb` según `delta`, eje por eje (Y, X, Z), deteniéndolo contra los
/// bloques que impiden moverse. Solo comprueba las celdas en las que entra,
/// así que una caja que ya solapaba un bloque puede salir de él.
pub fn move_aabb<W: Voxels + ?Sized>(world: &W, aabb: Aabb, delta: [i64; 3]) -> MoveResult {
    let mut aabb = aabb;
    let mut blocked = [false; 3];
    for axis in [1, 0, 2] {
        let (moved, hit) = sweep_axis(world, aabb, axis, delta[axis]);
        aabb = moved;
        blocked[axis] = hit;
    }
    MoveResult { aabb, blocked }
}

fn sweep_axis<W: Voxels + ?Sized>(world: &W, aabb: Aabb, axis: usize, d: i64) -> (Aabb, bool) {
    if d == 0 {
        return (aabb, false);
    }
    let [a, b] = match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let range = |lo: i64, hi: i64| cell_of(lo)..=cell_of(hi - 1);
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
    let start_of = |layer: i32| i64::from(layer) << FIXED_SHIFT;

    if d > 0 {
        let first = cell_of(aabb.max[axis] - 1) + 1;
        let last = cell_of(aabb.max[axis] + d - 1);
        for layer in first..=last {
            if layer_solid(layer) {
                let allowed = start_of(layer) - aabb.max[axis];
                return (aabb.translated(axis, allowed), true);
            }
        }
    } else {
        let first = cell_of(aabb.min[axis]) - 1;
        let last = cell_of(aabb.min[axis] + d);
        for layer in (last..=first).rev() {
            if layer_solid(layer) {
                let allowed = start_of(layer + 1) - aabb.min[axis];
                return (aabb.translated(axis, allowed), true);
            }
        }
    }
    (aabb.translated(axis, d), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SparseWorld, VoxelWorld, VoxelsMut};

    const ONE: i64 = FIXED_ONE;

    fn floor_world() -> VoxelWorld {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([0, 0, 0], [15, 0, 15], 1);
        w
    }

    fn player_at(x: i64, y: i64, z: i64) -> Aabb {
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
        // Centrada en x = z = 0: ocupa de -0,3 a 0,3, en las celdas -1 y 0.
        let neg = player_at(0, -ONE, 0);
        assert!(neg.overlaps_cell([-1, -1, -1]));
        assert!(neg.overlaps_cell([0, -1, 0]));
        assert!(!neg.overlaps_cell([-1, -2, -1]));
    }

    #[test]
    fn cells_round_down_with_negative_positions() {
        assert_eq!(cell_of(0), 0);
        assert_eq!(cell_of(ONE - 1), 0);
        assert_eq!(cell_of(-1), -1);
        assert_eq!(cell_of(-ONE), -1);
        assert_eq!(cell_of(-ONE - 1), -2);
        assert_eq!(cell_of(7_499_999 * ONE + ONE / 2), 7_499_999);
        assert_eq!(cell_of(-7_500_000 * ONE), -7_500_000);
    }

    /// Una cueva irregular alrededor de `base`, igual en cualquier sitio.
    fn cave_at(base: [i32; 3]) -> SparseWorld {
        let mut w = SparseWorld::with_border(7_500_000);
        let at = |x: i32, y: i32, z: i32| [base[0] + x, base[1] + y, base[2] + z];
        let [x0, y0, z0] = at(0, 0, 0);
        let [x1, y1, z1] = at(15, 0, 15);
        w.fill([x0, y0, z0], [x1, y1, z1], 1);
        for i in 0..40 {
            let [x, y, z] = at((i * 7) % 16, 1 + (i * 3) % 4, (i * 5) % 16);
            w.set(x, y, z, 1);
        }
        let [x0, y0, z0] = at(7, 1, 7);
        let [x1, y1, z1] = at(9, 3, 9);
        w.fill([x0, y0, z0], [x1, y1, z1], 0);
        w
    }

    /// Recorrido pseudoaleatorio; devuelve las posiciones relativas a `base`.
    fn wander(base: [i32; 3]) -> Vec<[i64; 3]> {
        let w = cave_at(base);
        let b = base.map(|c| i64::from(c) * ONE);
        let mut aabb = player_at(
            b[0] + 8 * ONE + ONE / 2,
            b[1] + ONE,
            b[2] + 8 * ONE + ONE / 2,
        );
        let mut seed: u32 = 12345;
        let mut path = Vec::new();
        for _ in 0..2000 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let d = [
                i64::from((seed >> 8) % 700) * 64 - 350 * 64,
                i64::from((seed >> 12) % 700) * 64 - 400 * 64,
                i64::from((seed >> 16) % 700) * 64 - 350 * 64,
            ];
            aabb = move_aabb(&w, aabb, d).aabb;
            assert!(!aabb.intersects_world(&w), "dentro de un bloque: {aabb:?}");
            path.push(std::array::from_fn(|i| aabb.min[i] - b[i]));
        }
        path
    }

    #[test]
    fn never_ends_inside_a_block_and_behaves_the_same_anywhere() {
        let origin = wander([0, 0, 0]);
        assert_eq!(wander([-17, -33, -16]), origin);
        // Lejos del origen, pero a 100 bloques del borde para que la pared
        // del mundo no cambie el recorrido.
        assert_eq!(wander([7_499_884, -4096, -7_499_900]), origin);
        assert_eq!(wander([-7_499_900, 1000, 7_499_884]), origin);
    }

    #[test]
    fn world_border_stops_players() {
        let mut w = SparseWorld::with_border(7_500_000);
        w.fill([7_499_990, -1, -2], [7_499_999, -1, 2], 1);
        let start = player_at(7_499_995 * ONE, 0, 0);
        let r = move_aabb(&w, start, [20 * ONE, 0, 0]);
        assert!(r.blocked[0]);
        assert_eq!(r.aabb.max[0], 7_500_000 * ONE);
    }
}
