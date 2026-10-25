//! Colisión de cajas en Q47.16 sobre `i64`, preparada para el mundo grande.
//!
//! Es el primer paso del hito W0 (`docs/adr/0002-mundo-grande.md`): las
//! posiciones de entidad pasan de `i32` con 10 bits fraccionarios
//! ([`FIXED_ONE`] = 1024 unidades por bloque, ±32 767 bloques) a `i64` con
//! 16 bits fraccionarios ([`FIXED64_ONE`] = 65 536 unidades por bloque,
//! ±70 billones de bloques). Los bloques siguen siendo coordenadas `i32`.
//!
//! El barrido usa un predicado de solidez en vez de `&VoxelWorld` para que
//! la aritmética se pueda probar en coordenadas negativas y extremas antes
//! de que exista el mundo con chunks negativos (punto 2 de W0).

use crate::world::VoxelWorld;

/// Unidades Q47.16 por bloque.
pub const FIXED64_ONE: i64 = 1 << 16;

/// Capas máximas que se barren por eje y movimiento. Un movimiento legítimo
/// cruza pocas capas (las velocidades del juego son de menos de un bloque
/// por tick); los saltos mayores se tratan como aire y no se frenan, para
/// que ningún delta absurdo dispare un barrido desbocado.
pub const MAX_SWEEP_LAYERS: i64 = 4096;

/// Caja alineada con los ejes, en Q47.16. Ocupa `[min, max)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Aabb64 {
    pub min: [i64; 3],
    pub max: [i64; 3],
}

impl Aabb64 {
    /// Caja con los pies en `feet` (centro de la base) y el tamaño dado.
    pub fn from_feet(feet: [i64; 3], half_width: i64, height: i64) -> Self {
        Self {
            min: [feet[0] - half_width, feet[1], feet[2] - half_width],
            max: [feet[0] + half_width, feet[1] + height, feet[2] + half_width],
        }
    }

    pub fn feet(&self) -> [i64; 3] {
        [
            (self.min[0] + self.max[0]) / 2,
            self.min[1],
            (self.min[2] + self.max[2]) / 2,
        ]
    }

    fn translated(mut self, axis: usize, d: i64) -> Self {
        self.min[axis] += d;
        self.max[axis] += d;
        self
    }

    /// Convierte una caja del punto fijo actual (Q22.10) a Q47.16. La
    /// conversión es exacta: multiplica por 64.
    pub fn from_q10(aabb: crate::Aabb) -> Self {
        Self {
            min: aabb.min.map(|c| i64::from(c) * 64),
            max: aabb.max.map(|c| i64::from(c) * 64),
        }
    }

    /// Convierte la caja de vuelta a Q22.10. Solo es exacta si todos los
    /// valores son múltiplos de 64, que es lo que produce este módulo a
    /// partir de entradas convertidas con [`Aabb64::from_q10`].
    pub fn to_q10(self) -> crate::Aabb {
        crate::Aabb {
            min: self.min.map(|c| (c / 64) as i32),
            max: self.max.map(|c| (c / 64) as i32),
        }
    }

    /// ¿Ocupa la caja alguna parte de la celda `cell`?
    pub fn overlaps_cell(&self, cell: [i32; 3]) -> bool {
        (0..3).all(|i| {
            self.min[i] < i64::from(cell[i] + 1) * FIXED64_ONE
                && self.max[i] > i64::from(cell[i]) * FIXED64_ONE
        })
    }
}

/// Resultado de mover una caja: posición final y ejes bloqueados.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveResult64 {
    pub aabb: Aabb64,
    /// Ejes en los que el movimiento se detuvo contra un bloque.
    pub blocked: [bool; 3],
}

/// Mueve `aabb` según `delta`, eje por eje (Y, X, Z), deteniéndolo contra las
/// celdas sólidas que informe `solid`. Misma semántica que
/// [`crate::move_aabb`]: una caja que ya solapaba un bloque puede salir.
pub fn move_aabb64(
    solid: impl Fn(i32, i32, i32) -> bool,
    aabb: Aabb64,
    delta: [i64; 3],
) -> MoveResult64 {
    let mut aabb = aabb;
    let mut blocked = [false; 3];
    for axis in [1, 0, 2] {
        let (moved, hit) = sweep_axis(&solid, aabb, axis, delta[axis]);
        aabb = moved;
        blocked[axis] = hit;
    }
    MoveResult64 { aabb, blocked }
}

/// Variante de [`move_aabb64`] contra un mundo finito. Fuera de los límites
/// del mundo no hay sólidos, igual que en el resto del motor.
pub fn move_aabb64_world(world: &VoxelWorld, aabb: Aabb64, delta: [i64; 3]) -> MoveResult64 {
    move_aabb64(|x, y, z| world.blocks_movement(x, y, z), aabb, delta)
}

fn sweep_axis(
    solid: impl Fn(i32, i32, i32) -> bool,
    aabb: Aabb64,
    axis: usize,
    d: i64,
) -> (Aabb64, bool) {
    if d == 0 {
        return (aabb, false);
    }
    let [a, b] = match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    // Coordenada de bloque desde una coordenada Q47.16; `i32::try_from`
    // devuelve `None` más allá de ±2 147 483 647 bloques, donde ya no hay
    // mundo posible: se trata como aire.
    let block_of = |c: i64| i32::try_from(c.div_euclid(FIXED64_ONE)).ok();
    let range = |lo: i64, hi: i64| {
        let l = block_of(lo)?;
        let h = block_of(hi - 1)?;
        Some(l..=h)
    };
    let layer_solid = |layer: i32| {
        let Some(mut ia_range) = range(aabb.min[a], aabb.max[a]) else {
            return false;
        };
        ia_range.any(|ia| {
            let Some(mut ib_range) = range(aabb.min[b], aabb.max[b]) else {
                return false;
            };
            ib_range.any(|ib| {
                let mut cell = [0; 3];
                cell[axis] = layer;
                cell[a] = ia;
                cell[b] = ib;
                solid(cell[0], cell[1], cell[2])
            })
        })
    };

    if d > 0 {
        let Some(last) = block_of(aabb.max[axis] + d - 1) else {
            return (aabb.translated(axis, d), false);
        };
        // Nacer más allá de ±i32::MAX bloques no es mundo: se empieza en la
        // capa representable más lejana, acotada por MAX_SWEEP_LAYERS.
        let raw_first = i64::from(block_of(aabb.max[axis] - 1).unwrap_or(i32::MIN)) + 1;
        let first = i32::try_from((raw_first).max(i64::from(last) - MAX_SWEEP_LAYERS + 1))
            .unwrap_or(i32::MIN);
        for layer in first..=last {
            if layer_solid(layer) {
                let allowed = i64::from(layer) * FIXED64_ONE - aabb.max[axis];
                return (aabb.translated(axis, allowed), true);
            }
        }
    } else {
        let Some(last) = block_of(aabb.min[axis] + d) else {
            return (aabb.translated(axis, d), false);
        };
        let raw_first = i64::from(block_of(aabb.min[axis]).unwrap_or(i32::MAX)) - 1;
        let first = i32::try_from(raw_first.min(i64::from(last) + MAX_SWEEP_LAYERS - 1))
            .unwrap_or(i32::MAX);
        for layer in (last..=first).rev() {
            if layer_solid(layer) {
                let allowed = (i64::from(layer) + 1) * FIXED64_ONE - aabb.min[axis];
                return (aabb.translated(axis, allowed), true);
            }
        }
    }
    (aabb.translated(axis, d), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Aabb, VoxelWorld, move_aabb};

    const ONE: i64 = FIXED64_ONE;

    fn player_at(feet: [i64; 3]) -> Aabb64 {
        Aabb64::from_feet(feet, 3 * ONE / 10, 17 * ONE / 10)
    }

    /// Suelo infinito por debajo de `y_floor`: sólida toda celda con
    /// `y < y_floor` y `x % 2 == 0` no, para probar columnas variadas.
    fn solid_floor(y_floor: i32) -> impl Fn(i32, i32, i32) -> bool {
        move |_x, y, _z| y < y_floor
    }

    #[test]
    fn falls_onto_floor_at_negative_extreme_coordinate() {
        // Extremo del mundo grande: ±7 499 999 bloques (15 M / 2).
        let x = -7_499_999 * ONE;
        let z = 7_499_999 * ONE;
        let start = player_at([x, 4 * ONE, z]);
        // Caída que intenta atravesar el suelo (bloques con y < 1).
        let result = move_aabb64(solid_floor(1), start, [0, -(3 * ONE + 5), 0]);
        assert!(result.blocked[1], "debería frenar contra el suelo");
        // Los pies quedan exactamente sobre el bloque 0: y == 1 bloque.
        assert_eq!(result.aabb.min[1], ONE);
        assert_eq!(result.aabb.min[0], x - 3 * ONE / 10);
        assert_eq!(result.aabb.min[2], z - 3 * ONE / 10);
    }

    #[test]
    fn horizontal_wall_stops_exactly_in_negative_quadrant() {
        // Muro en las celdas x <= -5 (por debajo de y = 10).
        let solid = |x: i32, y: i32, _z: i32| x <= -5 && y < 10;
        let start = player_at([-3 * ONE, 5 * ONE, -8 * ONE]);
        let result = move_aabb64(solid, start, [-2 * ONE, 0, 0]);
        assert!(result.blocked[0]);
        // La cara de la caja (min[0] = pies - medio ancho) queda pegada a la
        // cara derecha de la celda -5, que es -4 bloques.
        assert_eq!(result.aabb.min[0], -4 * ONE);
    }

    #[test]
    fn ceiling_bump_in_positive_extreme() {
        let solid = |_x: i32, y: i32, _z: i32| y >= 12;
        let start = player_at([7_499_999 * ONE, 8 * ONE, 0]);
        let result = move_aabb64(solid, start, [0, 3 * ONE, 0]);
        assert!(result.blocked[1]);
        assert_eq!(result.aabb.max[1], 12 * ONE);
    }

    #[test]
    fn beyond_i32_blocks_is_air_not_panic() {
        // 100 000 millones de bloques: fuera de i32, sin desbordar i64.
        let far = 100_000_000_000 * ONE;
        let start = player_at([far, far, -far]);
        let result = move_aabb64(|_, _, _| false, start, [-7 * ONE, 0, 7 * ONE]);
        assert!(!result.blocked.into_iter().any(|b| b));
        assert_eq!(result.aabb.min[0], far - 7 * ONE - 3 * ONE / 10);
    }

    #[test]
    fn matches_q10_collision_exactly_on_real_world() {
        // Paridad con la implementación actual en Q22.10: mismas entradas
        // escaladas, mismo resultado al volver a Q22.10.
        let mut world = VoxelWorld::new([16, 16, 16]).unwrap();
        world.fill([0, 0, 0], [15, 0, 15], 1);
        world.fill([6, 1, 6], [6, 3, 6], 1);
        let feet_q10 = [5 * 1024, 4 * 1024, 5 * 1024];
        let delta_q10 = [1024 / 3, -28 * 1024 / 900, 1024 / 3];
        let q10_aabb = Aabb::from_feet(feet_q10, 3 * 1024 / 10, 17 * 1024 / 10);

        let narrow = move_aabb(&world, q10_aabb, delta_q10);
        let wide = move_aabb64_world(
            &world,
            Aabb64::from_q10(q10_aabb),
            delta_q10.map(|c| i64::from(c) * 64),
        );
        assert_eq!(wide.blocked, narrow.blocked);
        assert_eq!(wide.aabb.to_q10(), narrow.aabb);
    }

    #[test]
    fn absurd_delta_is_bounded_and_lands_near_destination() {
        // Un salto de un millón de bloques solo barre MAX_SWEEP_LAYERS capas
        // junto al destino y se frena en la primera sólida de esa ventana.
        let start = player_at([0, 5 * ONE, 0]);
        let dest = 5 * ONE - 1_000_000 * ONE;
        let result = move_aabb64(solid_floor(1), start, [0, -1_000_000 * ONE, 0]);
        assert!(result.blocked[1]);
        // La ventana cubre las capas [dest, dest + MAX_SWEEP_LAYERS): la más
        // alta sólida es dest + MAX_SWEEP_LAYERS - 1, y aterriza encima.
        let land = 5 - 1_000_000 + MAX_SWEEP_LAYERS;
        assert_eq!(result.aabb.min[1], land * ONE);
        assert!(result.aabb.min[1] > dest - ONE);
    }

    #[test]
    fn q10_round_trip_is_exact() {
        let aabb = Aabb::from_feet([3 * 1024, -7 * 1024 + 5, 1024 / 2], 312, 1741);
        let round = Aabb64::from_q10(aabb).to_q10();
        assert_eq!(round, aabb);
    }

    #[test]
    fn overlaps_cell_uses_half_open_ranges() {
        let aabb = Aabb64::from_feet([2 * ONE, 0, 2 * ONE], ONE / 2, ONE);
        // La caja ocupa x y z de 1.5 a 2.5 bloques: solapa las celdas 1 y 2.
        assert!(aabb.overlaps_cell([2, 0, 2]));
        assert!(aabb.overlaps_cell([1, 0, 2]));
        // Tocar el borde superior de la celda no cuenta: [min, max).
        assert!(!aabb.overlaps_cell([2, 1, 2]));
        assert!(!aabb.overlaps_cell([0, 0, 2]));
    }
}
