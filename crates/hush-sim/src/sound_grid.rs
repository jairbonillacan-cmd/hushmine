//! Campo de sonido en cuadrícula.
//!
//! Un evento sonoro (una detonación, un pico contra la roca) se difumina por
//! el mundo con una búsqueda de coste mínimo por cubetas enteras: entrar en
//! una celda cuesta según su bloque y una celda «oye» mientras le quede
//! intensidad residual. Cada celda oída recuerda hacia qué vecino le llegó,
//! el primer paso de vuelta hacia la fuente; es la información que usa la IA
//! de las criaturas para acudir al ruido.

use std::collections::{BTreeMap, VecDeque};

use crate::BlockKind;
use grisu_voxel::Voxels;
use serde::{Deserialize, Serialize};

/// Intensidad máxima de un evento de sonido; lo que la supere se satura.
pub const MAX_INTENSITY: u16 = 64; // Se moverá a data/*.ron

/// Desfases de los 6 vecinos en orden fijo: `+x, -x, +y, -y, +z, -z`.
const NEIGHBORS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// Coste de propagación por entrar en una celda de ese bloque.
// Se moverá a data/*.ron
pub fn block_sound_cost(block: BlockKind) -> u16 {
    match block {
        // El sonido corre por los túneles.
        BlockKind::Air | BlockKind::Ladder | BlockKind::Rail => 1,
        // La tierra y la arcilla amortiguan el ruido.
        BlockKind::Dirt => 5,
        // El geófono y las criaturas oyen a través de la roca.
        BlockKind::Rock
        | BlockKind::Ore
        | BlockKind::Gold
        | BlockKind::Diamond
        | BlockKind::TeamBlock => 3,
        BlockKind::HardRock | BlockKind::ForceField => 6,
        // Sin entrada en la tabla original del diseño; asignación propia.
        BlockKind::Lava => 6, // Líquido denso.
        // Construcciones de equipo, a la altura del bloque de equipo.
        BlockKind::JumpPad
        | BlockKind::Electric
        | BlockKind::Beacon
        | BlockKind::Explosive
        | BlockKind::BucketStation
        | BlockKind::BucketPost => 3,
    }
}

/// Un sonido emitido en una celda del mundo.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SoundEvent {
    /// Celda de emisión; el coste del origen es 0 sea cual sea su bloque.
    pub origin: [i32; 3],
    /// Intensidad del evento; `0` no se oye en ninguna parte y lo que supere
    /// [`MAX_INTENSITY`] se satura.
    pub intensity: u16,
}

/// Lo que percibe una celda que oye el sonido.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HeardSound {
    /// Intensidad residual que llega a la celda: `intensidad - coste` (`>= 1`).
    pub intensity: u16,
    /// Desfase desde esta celda hacia el vecino del que le llega el sonido
    /// (el paso hacia la fuente); `[0, 0, 0]` en el propio origen.
    pub first_step_dir: [i32; 3],
}

/// Propaga un evento de sonido por el mundo y devuelve las celdas que lo oyen.
///
/// Búsqueda de coste mínimo tipo Dijkstra con cubetas enteras (una cola por
/// coste `0..intensidad`), sin montones binarios ni coma flotante. El origen
/// cuesta 0; entrar en la celda vecina cuesta [`block_sound_cost`] de su tipo,
/// y los identificadores que no estén en el catálogo suenan como
/// [`BlockKind::Rock`] (nunca se entra en pánico con datos externos). Una
/// celda oye si su coste es estrictamente menor que la intensidad, es decir,
/// si le queda intensidad residual; con `intensity == 0` el mapa queda vacío.
///
/// El desempate es determinista: vecinos en el orden fijo de [`NEIGHBORS`] y
/// una celda solo se mejora si el nuevo coste es estrictamente menor, así que
/// a la igualdad gana el primero que llegó y dos ejecuciones sobre el mismo
/// mundo devuelven exactamente el mismo `BTreeMap`. Fuera de los límites del
/// mundo `get` devuelve aire (coste 1) y el sonido se difumina por el vacío;
/// el alcance queda acotado por la propia intensidad (coste mínimo 1 por paso:
/// como mucho a 63 celdas del origen).
pub fn propagate_sound(world: &impl Voxels, event: SoundEvent) -> BTreeMap<[i32; 3], HeardSound> {
    let intensity = event.intensity.min(MAX_INTENSITY);
    let mut heard = BTreeMap::new();
    if intensity == 0 {
        return heard;
    }

    // Mejor coste conocido por celda y una cubeta (cola) por coste posible.
    let mut best: BTreeMap<[i32; 3], u16> = BTreeMap::new();
    let mut buckets = vec![VecDeque::new(); usize::from(intensity)];
    best.insert(event.origin, 0);
    heard.insert(
        event.origin,
        HeardSound {
            intensity,
            first_step_dir: [0, 0, 0],
        },
    );
    buckets[0].push_back(event.origin);

    for cost in 0..intensity {
        while let Some(cell) = buckets[usize::from(cost)].pop_front() {
            if best.get(&cell) != Some(&cost) {
                continue; // Entrada caducada: ya se llegó más barato.
            }
            for offset in NEIGHBORS {
                // `saturating_add` para no desbordar `i32` en orígenes extremos.
                let neighbor = [
                    cell[0].saturating_add(offset[0]),
                    cell[1].saturating_add(offset[1]),
                    cell[2].saturating_add(offset[2]),
                ];
                let id = world.get(neighbor[0], neighbor[1], neighbor[2]);
                let kind = BlockKind::from_id(id).unwrap_or(BlockKind::Rock);
                let new_cost = cost + block_sound_cost(kind);
                // Sin intensidad residual la celda no oye ni retransmite.
                if new_cost >= intensity {
                    continue;
                }
                // Solo mejoras estrictas: a la igualdad gana el primero.
                if best.get(&neighbor).is_some_and(|&old| old <= new_cost) {
                    continue;
                }
                best.insert(neighbor, new_cost);
                heard.insert(
                    neighbor,
                    HeardSound {
                        intensity: intensity - new_cost,
                        first_step_dir: [-offset[0], -offset[1], -offset[2]],
                    },
                );
                buckets[usize::from(new_cost)].push_back(neighbor);
            }
        }
    }
    heard
}

#[cfg(test)]
mod tests {
    use super::*;
    use grisu_voxel::{AIR, VoxelWorld};

    /// Mundo 32³ macizo del bloque indicado.
    fn solid_world(block: BlockKind) -> VoxelWorld {
        let mut world = VoxelWorld::new([32, 32, 32]).unwrap();
        world.fill([0, 0, 0], [31, 31, 31], block.id());
        world
    }

    /// Mundo de roca con un túnel de aire en `[0..=31, 16, 16]`.
    fn tunneled_world() -> VoxelWorld {
        let mut world = solid_world(BlockKind::Rock);
        world.fill([0, 16, 16], [31, 16, 16], AIR);
        world
    }

    #[test]
    fn cost_table_matches_design() {
        let expected: [u16; BlockKind::COUNT] =
            [1, 5, 3, 6, 3, 3, 3, 6, 3, 6, 1, 3, 3, 1, 3, 3, 3, 3];
        for (kind, cost) in BlockKind::ALL.into_iter().zip(expected) {
            assert_eq!(block_sound_cost(kind), cost, "coste de {:?}", kind);
        }
    }

    #[test]
    fn tunnel_carries_sound_farther_than_solid_rock() {
        let event = SoundEvent {
            origin: [0, 16, 16],
            intensity: MAX_INTENSITY,
        };

        let rock = propagate_sound(&solid_world(BlockKind::Rock), event);
        assert_eq!(rock.get(&[21, 16, 16]).map(|s| s.intensity), Some(1));
        assert!(!rock.contains_key(&[22, 16, 16]));

        let tunnel = propagate_sound(&tunneled_world(), event);
        let cell = tunnel[&[25, 16, 16]];
        assert_eq!(cell.intensity, 39);
        assert_eq!(cell.first_step_dir, [-1, 0, 0]);
    }

    #[test]
    fn dirt_muffles_more_than_rock() {
        let event = SoundEvent {
            origin: [0, 16, 16],
            intensity: MAX_INTENSITY,
        };

        let dirt = propagate_sound(&solid_world(BlockKind::Dirt), event);
        assert_eq!(dirt.get(&[12, 16, 16]).map(|s| s.intensity), Some(4));
        assert!(!dirt.contains_key(&[13, 16, 16]));
        assert!(!dirt.contains_key(&[15, 16, 16]));

        let rock = propagate_sound(&solid_world(BlockKind::Rock), event);
        assert_eq!(rock.get(&[15, 16, 16]).map(|s| s.intensity), Some(19));
    }

    #[test]
    fn origin_hears_full_intensity_and_points_nowhere() {
        let heard = propagate_sound(
            &tunneled_world(),
            SoundEvent {
                origin: [0, 16, 16],
                intensity: 8,
            },
        );
        let origin = heard[&[0, 16, 16]];
        assert_eq!((origin.intensity, origin.first_step_dir), (8, [0, 0, 0]));
    }

    #[test]
    fn same_world_produces_identical_maps() {
        let world = tunneled_world();
        let event = SoundEvent {
            origin: [16, 16, 16],
            intensity: 24,
        };
        assert_eq!(
            propagate_sound(&world, event),
            propagate_sound(&world, event)
        );
    }

    #[test]
    fn zero_intensity_hears_nothing_and_overflow_saturates() {
        let world = tunneled_world();
        let silent = propagate_sound(
            &world,
            SoundEvent {
                origin: [0, 16, 16],
                intensity: 0,
            },
        );
        assert!(silent.is_empty());

        // Saturada a 64: a 63 pasos de aire queda residual 1; a 64, nada.
        let loud = propagate_sound(
            &world,
            SoundEvent {
                origin: [0, 16, 16],
                intensity: 500,
            },
        );
        assert_eq!(loud.get(&[63, 16, 16]).map(|s| s.intensity), Some(1));
        assert!(!loud.contains_key(&[64, 16, 16]));
    }
}
