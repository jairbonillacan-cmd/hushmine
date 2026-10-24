//! Raycast DDA (Amanatides y Woo) sobre la cuadrícula.
//!
//! Usa `f32` porque sirve al cliente para apuntar. El servidor valida la
//! acción con sus propias reglas en enteros (alcance, bloque objetivo).

use crate::VoxelWorld;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    /// Bloque sólido alcanzado.
    pub block: [i32; 3],
    /// Normal de la cara por la que entró el rayo; `block + normal` es la
    /// celda donde se colocaría un bloque nuevo.
    pub normal: [i32; 3],
    /// Distancia recorrida hasta la cara, en bloques.
    pub distance: f32,
}

/// Lanza un rayo desde `origin` en la dirección `dir` (no hace falta que sea
/// unitaria) y devuelve el primer bloque sólido a menos de `max_distance`.
pub fn raycast(
    world: &VoxelWorld,
    origin: [f32; 3],
    dir: [f32; 3],
    max_distance: f32,
) -> Option<RayHit> {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if !len.is_finite() || len == 0.0 || !origin.iter().all(|c| c.is_finite()) {
        return None;
    }
    let dir = dir.map(|d| d / len);
    let mut cell = origin.map(|c| c.floor() as i32);
    let step = dir.map(|d| if d > 0.0 { 1 } else { -1 });
    // Distancia del rayo para cruzar una celda entera en cada eje.
    let delta = dir.map(|d| {
        if d == 0.0 {
            f32::INFINITY
        } else {
            1.0 / d.abs()
        }
    });
    // Distancia hasta el primer cruce de cada eje.
    let mut next: [f32; 3] = std::array::from_fn(|i| {
        if dir[i] == 0.0 {
            f32::INFINITY
        } else {
            let boundary = if dir[i] > 0.0 {
                cell[i] as f32 + 1.0
            } else {
                cell[i] as f32
            };
            (boundary - origin[i]) / dir[i]
        }
    });

    if world.is_solid(cell[0], cell[1], cell[2]) {
        return Some(RayHit {
            block: cell,
            normal: [0; 3],
            distance: 0.0,
        });
    }
    loop {
        let axis = if next[0] < next[1] {
            if next[0] < next[2] { 0 } else { 2 }
        } else if next[1] < next[2] {
            1
        } else {
            2
        };
        let distance = next[axis];
        if distance > max_distance {
            return None;
        }
        cell[axis] += step[axis];
        next[axis] += delta[axis];
        if world.is_solid(cell[0], cell[1], cell[2]) {
            let mut normal = [0; 3];
            normal[axis] = -step[axis];
            return Some(RayHit {
                block: cell,
                normal,
                distance,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall_world() -> VoxelWorld {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([10, 0, 0], [10, 15, 15], 1);
        w
    }

    #[test]
    fn hits_wall_face_on_the_near_side() {
        let w = wall_world();
        let hit = raycast(&w, [2.5, 4.5, 4.5], [1.0, 0.0, 0.0], 32.0).unwrap();
        assert_eq!(hit.block, [10, 4, 4]);
        assert_eq!(hit.normal, [-1, 0, 0]);
        assert!((hit.distance - 7.5).abs() < 1e-4);
    }

    #[test]
    fn diagonal_ray_and_negative_direction() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.set(1, 1, 1, 1);
        let hit = raycast(&w, [5.5, 5.5, 5.5], [-1.0, -1.0, -1.0], 32.0).unwrap();
        assert_eq!(hit.block, [1, 1, 1]);
        assert_eq!(hit.normal.iter().map(|c| c.abs()).sum::<i32>(), 1);
    }

    #[test]
    fn respects_max_distance_and_misses() {
        let w = wall_world();
        assert!(raycast(&w, [2.5, 4.5, 4.5], [1.0, 0.0, 0.0], 5.0).is_none());
        assert!(raycast(&w, [2.5, 4.5, 4.5], [-1.0, 0.0, 0.0], 64.0).is_none());
        assert!(raycast(&w, [2.5, 4.5, 4.5], [0.0, 0.0, 0.0], 64.0).is_none());
    }

    #[test]
    fn origin_inside_block_hits_immediately() {
        let w = wall_world();
        let hit = raycast(&w, [10.5, 1.5, 1.5], [0.0, 1.0, 0.0], 8.0).unwrap();
        assert_eq!((hit.block, hit.distance), ([10, 1, 1], 0.0));
    }
}
