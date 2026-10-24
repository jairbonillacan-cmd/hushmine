//! Mallas de chunks por eliminación de caras ocultas.
//!
//! Cada cara visible se empaqueta en 8 bytes ([`PackedFace`]) y el shader la
//! expande en dos triángulos («vertex pulling» por instancias, que también
//! funciona en WebGL2). La oclusión ambiental se calcula por vértice.

use crate::{AIR, CHUNK_SIZE, LightMap, MAX_LIGHT, VoxelWorld};

/// Dirección de una cara. El orden coincide con la tabla del shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

impl Direction {
    pub const ALL: [Self; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];

    pub fn from_index(index: u8) -> Option<Self> {
        Self::ALL.get(usize::from(index)).copied()
    }

    /// Normal de la cara.
    pub fn normal(self) -> [i32; 3] {
        FACE_AXES[self as usize].0
    }

    /// Ejes `u` y `v` de la cara, con `u × v = normal`: recorrer las
    /// esquinas en el orden de [`CORNERS`] es antihorario visto desde fuera.
    pub fn tangents(self) -> ([i32; 3], [i32; 3]) {
        let (_, u, v) = FACE_AXES[self as usize];
        (u, v)
    }
}

/// (normal, u, v) de cada dirección. Debe coincidir con `voxel.wgsl`.
const FACE_AXES: [([i32; 3], [i32; 3], [i32; 3]); 6] = [
    ([1, 0, 0], [0, 0, -1], [0, 1, 0]),
    ([-1, 0, 0], [0, 0, 1], [0, 1, 0]),
    ([0, 1, 0], [1, 0, 0], [0, 0, -1]),
    ([0, -1, 0], [1, 0, 0], [0, 0, 1]),
    ([0, 0, 1], [1, 0, 0], [0, 1, 0]),
    ([0, 0, -1], [-1, 0, 0], [0, 1, 0]),
];

/// Signos (u, v) de las cuatro esquinas de una cara, en orden antihorario.
pub const CORNERS: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];

/// Cara visible empaquetada en 8 bytes.
///
/// - `a`: x (7 bits) · y (7) · z (7) · dirección (3) · oclusión (4 × 2 bits).
/// - `b`: tipo de bloque (8 bits) · luz de la celda de delante (4 bits).
///
/// La oclusión de cada esquina va de 0 (muy ocluida) a 3 (libre).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedFace {
    pub a: u32,
    pub b: u32,
}

impl PackedFace {
    pub fn new(pos: [u32; 3], dir: Direction, ao: [u8; 4], block: u8, light: u8) -> Self {
        debug_assert!(pos.iter().all(|&c| c < 128));
        debug_assert!(ao.iter().all(|&o| o <= 3));
        debug_assert!(light <= MAX_LIGHT);
        let ao_bits = ao
            .iter()
            .enumerate()
            .fold(0, |acc, (i, &o)| acc | (u32::from(o) << (2 * i)));
        Self {
            a: pos[0] | (pos[1] << 7) | (pos[2] << 14) | ((dir as u32) << 21) | (ao_bits << 24),
            b: u32::from(block) | (u32::from(light & 0xF) << 8),
        }
    }

    pub fn pos(self) -> [u32; 3] {
        [self.a & 0x7F, (self.a >> 7) & 0x7F, (self.a >> 14) & 0x7F]
    }

    pub fn direction(self) -> Direction {
        // Los 3 bits solo pueden contener valores escritos por `new`.
        Direction::from_index(((self.a >> 21) & 0x7) as u8).unwrap_or(Direction::PosY)
    }

    pub fn ao(self) -> [u8; 4] {
        std::array::from_fn(|i| ((self.a >> (24 + 2 * i)) & 0x3) as u8)
    }

    pub fn block(self) -> u8 {
        (self.b & 0xFF) as u8
    }

    pub fn light(self) -> u8 {
        ((self.b >> 8) & 0xF) as u8
    }
}

/// Genera las caras visibles de un chunk. Una cara es visible si el bloque
/// vecino en esa dirección es aire (también fuera del mundo). Sin mapa de
/// luz, todas las caras llevan luz completa.
pub fn mesh_chunk(
    world: &VoxelWorld,
    light: Option<&LightMap>,
    chunk_index: usize,
) -> Vec<PackedFace> {
    let chunk = world.chunk(chunk_index);
    let mut faces = Vec::new();
    if chunk.is_empty() {
        return faces;
    }
    let [cx, cy, cz] = world
        .chunk_coords(chunk_index)
        .map(|c| (c * CHUNK_SIZE) as i32);
    for ly in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let block = chunk.get(lx, ly, lz);
                if block == AIR {
                    continue;
                }
                let p = [cx + lx as i32, cy + ly as i32, cz + lz as i32];
                for dir in Direction::ALL {
                    let n = dir.normal();
                    let front = add(p, n);
                    if world.is_solid(front[0], front[1], front[2]) {
                        continue;
                    }
                    let ao = face_ao(world, front, dir);
                    let pos = p.map(|c| c as u32);
                    let level = light.map_or(MAX_LIGHT, |l| l.get(front[0], front[1], front[2]));
                    faces.push(PackedFace::new(pos, dir, ao, block, level));
                }
            }
        }
    }
    faces
}

/// Oclusión de las cuatro esquinas de una cara, mirando la capa de aire
/// que tiene delante (`front`).
fn face_ao(world: &VoxelWorld, front: [i32; 3], dir: Direction) -> [u8; 4] {
    let (u, v) = dir.tangents();
    let solid = |p: [i32; 3]| world.is_solid(p[0], p[1], p[2]);
    CORNERS.map(|(su, sv)| {
        let side_u = add(front, scale(u, su));
        let side_v = add(front, scale(v, sv));
        let corner = add(side_u, scale(v, sv));
        vertex_ao(solid(side_u), solid(side_v), solid(corner))
    })
}

/// Fórmula clásica de oclusión por vértice: dos laterales tapan del todo.
fn vertex_ao(side1: bool, side2: bool, corner: bool) -> u8 {
    if side1 && side2 {
        0
    } else {
        3 - u8::from(side1) - u8::from(side2) - u8::from(corner)
    }
}

fn add(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: [i32; 3], s: i32) -> [i32; 3] {
    a.map(|c| c * s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cross(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    #[test]
    fn tangents_are_right_handed() {
        for dir in Direction::ALL {
            let (u, v) = dir.tangents();
            assert_eq!(cross(u, v), dir.normal(), "{dir:?}");
        }
    }

    #[test]
    fn packing_round_trips() {
        let face = PackedFace::new([127, 5, 64], Direction::NegZ, [0, 1, 2, 3], 200, 9);
        assert_eq!(face.pos(), [127, 5, 64]);
        assert_eq!(face.direction(), Direction::NegZ);
        assert_eq!(face.ao(), [0, 1, 2, 3]);
        assert_eq!(face.block(), 200);
        assert_eq!(face.light(), 9);
        assert_eq!(size_of::<PackedFace>(), 8);
    }

    #[test]
    fn lone_block_has_six_unoccluded_faces() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.set(5, 5, 5, 3);
        let faces = mesh_chunk(&w, None, 0);
        assert_eq!(faces.len(), 6);
        assert!(faces.iter().all(|f| f.ao() == [3; 4] && f.block() == 3));
    }

    #[test]
    fn touching_blocks_hide_shared_faces() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.set(5, 5, 5, 1);
        w.set(6, 5, 5, 1);
        assert_eq!(mesh_chunk(&w, None, 0).len(), 10);
    }

    #[test]
    fn solid_world_only_shows_its_outer_shell() {
        let mut w = VoxelWorld::new([32, 16, 16]).unwrap();
        w.fill([0, 0, 0], [31, 15, 15], 2);
        let total: usize = (0..w.chunk_count())
            .map(|i| mesh_chunk(&w, None, i).len())
            .sum();
        let shell = 2 * (32 * 16 + 32 * 16 + 16 * 16);
        assert_eq!(total, shell);
    }

    #[test]
    fn floor_next_to_wall_is_occluded() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([0, 0, 0], [15, 0, 15], 1);
        w.set(5, 1, 5, 1);
        let top_of = |x: u32, z: u32| {
            mesh_chunk(&w, None, 0)
                .into_iter()
                .find(|f| f.pos() == [x, 0, z] && f.direction() == Direction::PosY)
                .unwrap()
        };
        // El suelo junto al bloque tiene dos esquinas ocluidas.
        let ao = top_of(6, 5).ao();
        assert_eq!(ao.iter().filter(|&&o| o < 3).count(), 2, "{ao:?}");
        // Lejos del bloque no hay oclusión.
        assert_eq!(top_of(12, 12).ao(), [3; 4]);
    }
}
