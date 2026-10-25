//! Raycast DDA (Amanatides y Woo) sobre la cuadrícula.
//!
//! Usa `f64` porque sirve al cliente para apuntar y tiene que ser preciso
//! también a 7,5 millones de bloques del origen (un `f32` ahí solo distingue
//! medios bloques). El servidor valida la acción con sus propias reglas en
//! enteros (alcance, bloque objetivo).

use crate::Voxels;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    /// Bloque sólido alcanzado.
    pub block: [i32; 3],
    /// Normal de la cara por la que entró el rayo; `block + normal` es la
    /// celda donde se colocaría un bloque nuevo.
    pub normal: [i32; 3],
    /// Distancia recorrida hasta la cara, en bloques.
    pub distance: f64,
}

/// Lanza un rayo desde `origin` en la dirección `dir` (no hace falta que sea
/// unitaria) y devuelve el primer bloque sólido a menos de `max_distance`.
pub fn raycast<W: Voxels + ?Sized>(
    world: &W,
    origin: [f64; 3],
    dir: [f64; 3],
    max_distance: f64,
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
            f64::INFINITY
        } else {
            1.0 / d.abs()
        }
    });
    // Distancia hasta el primer cruce de cada eje.
    let mut next: [f64; 3] = std::array::from_fn(|i| {
        if dir[i] == 0.0 {
            f64::INFINITY
        } else {
            let boundary = if dir[i] > 0.0 {
                f64::from(cell[i]) + 1.0
            } else {
                f64::from(cell[i])
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
    use crate::{SparseWorld, VoxelWorld, VoxelsMut};

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
        assert!((hit.distance - 7.5).abs() < 1e-9);
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

    #[test]
    fn same_hit_at_negative_and_extreme_coordinates() {
        // Pared a 7,5 bloques del ojo, en el origen y lejos de él.
        for base in [[0, 0, 0], [-17, -1, -16], [7_499_990, 1000, -7_500_000]] {
            let mut w = SparseWorld::new();
            let wall_x = base[0] + 10;
            w.fill(
                [wall_x, base[1] - 2, base[2] - 2],
                [wall_x, base[1] + 2, base[2] + 2],
                1,
            );
            let eye = [
                f64::from(base[0]) + 2.5,
                f64::from(base[1]) + 0.5,
                f64::from(base[2]) + 0.5,
            ];
            let hit = raycast(&w, eye, [1.0, 0.0, 0.0], 32.0).unwrap();
            assert_eq!(hit.block, [wall_x, base[1], base[2]], "base {base:?}");
            assert_eq!(hit.normal, [-1, 0, 0]);
            assert!((hit.distance - 7.5).abs() < 1e-6, "{}", hit.distance);
            // Hacia atrás, en dirección negativa, tampoco falla.
            let back = raycast(&w, [eye[0] + 20.0, eye[1], eye[2]], [-1.0, 0.0, 0.0], 32.0);
            assert_eq!(back.map(|h| h.normal), Some([1, 0, 0]));
        }
    }
}
