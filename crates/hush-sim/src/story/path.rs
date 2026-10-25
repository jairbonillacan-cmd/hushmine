//! Búsqueda de rutas vóxel 3D (A*) sobre los chunks cargados.
//!
//! Un nodo es transitable si tiene apoyo sólido debajo y dos bloques de
//! altura libres. La búsqueda se limita a una caja local alrededor de la
//! criatura para acotar el coste; todo determinista (costes enteros y
//! desempates por orden de inserción).

use std::collections::BinaryHeap;

use grisu_voxel::VoxelWorld;

/// Caja de búsqueda por eje (la caja mide 2*HALF+1).
pub const HALF: i32 = 8;
/// Máximo de nodos expandidos por búsqueda.
pub const MAX_NODES: usize = 700;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Node {
    cost: u32,
    cell: [i32; 3],
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // BinaryHeap es un montículo máximo: invertimos para sacar primero
        // el coste menor.
        other.cost.cmp(&self.cost).then(self.cell.cmp(&other.cell))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// ¿La celda es transitable (suelo sólido debajo y dos bloques libres)?
pub fn walkable(world: &VoxelWorld, x: i32, y: i32, z: i32) -> bool {
    world.blocks_movement(x, y - 1, z)
        && world.get(x, y, z) == grisu_voxel::AIR
        && world.get(x, y + 1, z) == grisu_voxel::AIR
}

/// Siguiente paso hacia el destino según A* local. Devuelve la celda a la
/// que moverse este tick (adyacente a `from`) o `None` si no hay ruta.
pub fn next_step(world: &VoxelWorld, from: [i32; 3], to: [i32; 3]) -> Option<[i32; 3]> {
    if from == to {
        return None;
    }
    // Primera caja ajustada a la distancia directa; si el camino está
    // cortado (muros), se reintentan cajas crecientes. El límite de nodos
    // acota el coste en cualquier caso.
    for scale in [1, 3, 6] {
        if let Some(step) = next_step_boxed(world, from, to, scale) {
            return Some(step);
        }
    }
    None
}

fn next_step_boxed(
    world: &VoxelWorld,
    from: [i32; 3],
    to: [i32; 3],
    scale: i32,
) -> Option<[i32; 3]> {
    let mut came_from = std::collections::HashMap::new();
    let mut heap = BinaryHeap::new();
    let h = |c: [i32; 3]| -> u32 {
        (c[0] - to[0]).unsigned_abs()
            + (c[1] - to[1]).unsigned_abs()
            + (c[2] - to[2]).unsigned_abs()
    };
    heap.push(Node {
        cost: h(from),
        cell: from,
    });
    let mut expanded = 0;
    let mut found = false;
    let mut reached: std::collections::HashSet<[i32; 3]> = std::collections::HashSet::new();
    reached.insert(from);
    // La caja de búsqueda cubre el segmento origen-destino (con margen y
    // el factor de reintento).
    let origin = std::array::from_fn(|i| (from[i] + to[i]) / 2);
    let half_axis = |a: usize| (from[a] - to[a]).abs() + 2;
    let half = (HALF.max(half_axis(0)).max(half_axis(2)) * scale).min(48);
    while let Some(Node { cost, cell }) = heap.pop() {
        expanded += 1;
        if expanded > MAX_NODES {
            break;
        }
        if cell == to {
            found = true;
            break;
        }
        for step in steps(world, cell, origin, half) {
            if reached.insert(step) {
                came_from.insert(step, cell);
                heap.push(Node {
                    cost: cost - h(cell) + 1 + h(step),
                    cell: step,
                });
            }
        }
    }
    if !found {
        return None;
    }
    // Retrocede desde el destino hasta el sucesor inmediato de `from`.
    let mut current = to;
    while let Some(&prev) = came_from.get(&current) {
        if prev == from {
            return Some(current);
        }
        current = prev;
    }
    None
}

/// Vecinos transitables (incluido subir y bajar un bloque), recortados a
/// la caja de búsqueda anclada en `origin`.
fn steps(world: &VoxelWorld, cell: [i32; 3], origin: [i32; 3], half: i32) -> Vec<[i32; 3]> {
    const PLANAR: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let mut out = Vec::with_capacity(12);
    for [dx, dz] in PLANAR {
        let x = cell[0] + dx;
        let z = cell[2] + dz;
        for dy in [0, 1, -1] {
            let y = cell[1] + dy;
            if (x - origin[0]).abs() > half || (z - origin[2]).abs() > half {
                continue;
            }
            // Fuera del mundo no se camina (los bordes cuentan como sólido
            // para la colisión, pero la IA no debe salir del mapa).
            if !world.in_bounds(x, y, z) || !world.in_bounds(x, y + 1, z) {
                continue;
            }
            if walkable(world, x, y, z) {
                out.push([x, y, z]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> VoxelWorld {
        let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
        w.fill([0, 0, 0], [31, 2, 31], 1);
        w
    }

    #[test]
    fn walks_a_straight_corridor() {
        let w = floor();
        let step = next_step(&w, [5, 3, 5], [12, 3, 5]).unwrap();
        assert_eq!(step, [6, 3, 5]);
    }

    #[test]
    fn routes_around_a_wall() {
        let mut w = floor();
        // Muro en x=8 con hueco en z=20.
        for z in 0..32 {
            if (18..=22).contains(&z) {
                continue;
            }
            w.set(8, 3, z, 1);
            w.set(8, 4, z, 1);
        }
        let step = next_step(&w, [4, 3, 4], [14, 3, 4]);
        assert!(step.is_some(), "no encontró ruta");
        // Simulamos la ruta completa: debe llegar sin atravesar el muro.
        let mut pos = [4, 3, 4];
        let target = [14, 3, 4];
        for _ in 0..200 {
            if pos == target {
                break;
            }
            let Some(next) = next_step(&w, pos, target) else {
                panic!("atasado en {pos:?}");
            };
            // Solo se puede cruzar el plano x = 8 por el hueco.
            if next[0] == 8 {
                assert!((18..=22).contains(&next[2]), "atravesó el muro en {next:?}");
            }
            pos = next;
        }
        assert_eq!(pos, target);
    }

    #[test]
    fn unreachable_returns_none() {
        let mut w = floor();
        // El destino encerrado en roca.
        for dx in -1..=1 {
            for dz in -1..=1 {
                for dy in 0..3 {
                    w.set(
                        20 + dx,
                        3 + dy,
                        20 + dz,
                        if dx == 0 && dz == 0 && dy == 0 { 0 } else { 1 },
                    );
                }
            }
        }
        w.set(20, 3, 20, 0);
        w.set(20, 4, 20, 1);
        assert!(next_step(&w, [4, 3, 4], [20, 3, 20]).is_none());
    }
}
