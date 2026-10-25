//! Mallas de chunks por eliminación de caras ocultas.
//!
//! Cada cara visible se empaqueta en 8 bytes ([`PackedFace`]) y el shader la
//! expande en dos triángulos («vertex pulling» por instancias, que también
//! funciona en WebGL2). La oclusión ambiental se calcula por vértice.

use crate::{AIR, CHUNK_SIZE, ChunkPos, DualLightMap, LightMap, MAX_LIGHT, Voxels};

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
/// La posición es **local al chunk** (0-15 por eje): el render suma el
/// origen del chunk relativo a la cámara, así que las coordenadas del
/// mundo pueden ser enormes o negativas sin perder precisión.
///
/// - `a`: x (4 bits) · y (4) · z (4) · dirección (3) · oclusión (4 × 2 bits).
/// - `b`: tipo de bloque (8 bits) · luz de bloque (4) · luz de cielo (4) de la
///   celda de delante. Sin mapa dual las dos luces van iguales.
///
/// La oclusión de cada esquina va de 0 (muy ocluida) a 3 (libre).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedFace {
    pub a: u32,
    pub b: u32,
}

impl PackedFace {
    pub fn new(
        local: [u32; 3],
        dir: Direction,
        ao: [u8; 4],
        block: u8,
        light: u8,
        sky_light: u8,
    ) -> Self {
        debug_assert!(local.iter().all(|&c| c < CHUNK_SIZE as u32));
        debug_assert!(ao.iter().all(|&o| o <= 3));
        debug_assert!(light <= MAX_LIGHT && sky_light <= MAX_LIGHT);
        let ao_bits = ao
            .iter()
            .enumerate()
            .fold(0, |acc, (i, &o)| acc | (u32::from(o) << (2 * i)));
        Self {
            a: local[0]
                | (local[1] << 4)
                | (local[2] << 8)
                | ((dir as u32) << 12)
                | (ao_bits << 15),
            b: u32::from(block)
                | (u32::from(light & 0xF) << 8)
                | (u32::from(sky_light & 0xF) << 12),
        }
    }

    /// Posición del bloque dentro de su chunk.
    pub fn local(self) -> [u32; 3] {
        [self.a & 0xF, (self.a >> 4) & 0xF, (self.a >> 8) & 0xF]
    }

    pub fn direction(self) -> Direction {
        // Los 3 bits solo pueden contener valores escritos por `new`.
        Direction::from_index(((self.a >> 12) & 0x7) as u8).unwrap_or(Direction::PosY)
    }

    pub fn ao(self) -> [u8; 4] {
        std::array::from_fn(|i| ((self.a >> (15 + 2 * i)) & 0x3) as u8)
    }

    pub fn block(self) -> u8 {
        (self.b & 0xFF) as u8
    }

    /// Luz de bloque de la celda de delante.
    pub fn light(self) -> u8 {
        ((self.b >> 8) & 0xF) as u8
    }

    /// Luz de cielo de la celda de delante.
    pub fn sky_light(self) -> u8 {
        ((self.b >> 12) & 0xF) as u8
    }
}

/// Genera las caras visibles de un chunk. Una cara es visible si el bloque
/// vecino en esa dirección es aire (también fuera del mundo). Sin mapa de
/// luz, todas las caras llevan luz completa.
pub fn mesh_chunk<W: Voxels + ?Sized>(
    world: &W,
    light: Option<&LightMap>,
    chunk: ChunkPos,
) -> Vec<PackedFace> {
    mesh_with(world, chunk, |p| {
        let level = light.map_or(MAX_LIGHT, |l| l.get(p[0], p[1], p[2]));
        (level, level)
    })
}

/// Como [`mesh_chunk`], pero con el mapa de luz doble: cada cara guarda por
/// separado la luz de bloque y la de cielo de la celda de delante, y el
/// shader las combina con la hora del día.
pub fn mesh_chunk_dual<W: Voxels + ?Sized>(
    world: &W,
    light: Option<&DualLightMap>,
    chunk: ChunkPos,
) -> Vec<PackedFace> {
    mesh_with(world, chunk, |p| {
        light.map_or((MAX_LIGHT, MAX_LIGHT), |l| {
            (l.block(p[0], p[1], p[2]), l.sky(p[0], p[1], p[2]))
        })
    })
}

/// Recorre el chunk y empaqueta cada cara visible con la luz `(bloque,
/// cielo)` que da `light_at` para la celda de delante.
fn mesh_with<W: Voxels + ?Sized>(
    world: &W,
    chunk: ChunkPos,
    light_at: impl Fn([i32; 3]) -> (u8, u8),
) -> Vec<PackedFace> {
    let mut faces = Vec::new();
    if world.chunk_is_empty(chunk) {
        return faces;
    }
    let [cx, cy, cz] = chunk.origin();
    for ly in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let p = [cx + lx as i32, cy + ly as i32, cz + lz as i32];
                let block = world.get(p[0], p[1], p[2]);
                if block == AIR {
                    continue;
                }
                for dir in Direction::ALL {
                    let n = dir.normal();
                    let front = add(p, n);
                    if world.is_solid(front[0], front[1], front[2]) {
                        continue;
                    }
                    let ao = face_ao(world, front, dir);
                    let local = [lx, ly, lz].map(|c| c as u32);
                    let (block_light, sky_light) = light_at(front);
                    faces.push(PackedFace::new(
                        local,
                        dir,
                        ao,
                        block,
                        block_light,
                        sky_light,
                    ));
                }
            }
        }
    }
    faces
}

/// Oclusión de las cuatro esquinas de una cara, mirando la capa de aire
/// que tiene delante (`front`).
fn face_ao<W: Voxels + ?Sized>(world: &W, front: [i32; 3], dir: Direction) -> [u8; 4] {
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
    use crate::{SparseWorld, VoxelWorld, VoxelsMut};

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
        let face = PackedFace::new([15, 5, 0], Direction::NegZ, [0, 1, 2, 3], 200, 9, 12);
        assert_eq!(face.local(), [15, 5, 0]);
        assert_eq!(face.direction(), Direction::NegZ);
        assert_eq!(face.ao(), [0, 1, 2, 3]);
        assert_eq!(face.block(), 200);
        assert_eq!(face.light(), 9);
        assert_eq!(face.sky_light(), 12);
        assert_eq!(size_of::<PackedFace>(), 8);
    }

    #[test]
    fn dual_mesh_separates_sky_and_block_light() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([0, 0, 0], [15, 4, 15], 1);
        // Cielo abierto sobre la superficie y una antorcha (bloque 7, nivel
        // 12) en el suelo que ilumina el canal de bloque.
        w.set(8, 5, 8, 7);
        let light = DualLightMap::new(&w, crate::emission_of(7, 12));
        let faces = mesh_chunk_dual(&w, Some(&light), ChunkPos::default());
        let top = faces
            .iter()
            .find(|f| f.local() == [8, 5, 8] && f.direction() == Direction::PosY)
            .unwrap();
        // El aire sobre la antorcha recibe 11 de bloque (12 - 1) y 15 de cielo.
        assert_eq!((top.light(), top.sky_light()), (11, MAX_LIGHT));
        // Sin mapa dual las dos luces van iguales.
        let plain = mesh_chunk(&w, None, ChunkPos::default());
        assert!(plain.iter().all(|f| f.light() == f.sky_light()));
    }

    #[test]
    fn lone_block_has_six_unoccluded_faces() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.set(5, 5, 5, 3);
        let faces = mesh_chunk(&w, None, ChunkPos::default());
        assert_eq!(faces.len(), 6);
        assert!(faces.iter().all(|f| f.ao() == [3; 4] && f.block() == 3));
    }

    #[test]
    fn touching_blocks_hide_shared_faces() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.set(5, 5, 5, 1);
        w.set(6, 5, 5, 1);
        assert_eq!(mesh_chunk(&w, None, ChunkPos::default()).len(), 10);
    }

    #[test]
    fn solid_world_only_shows_its_outer_shell() {
        let mut w = VoxelWorld::new([32, 16, 16]).unwrap();
        w.fill([0, 0, 0], [31, 15, 15], 2);
        let total: usize = (0..w.chunk_count())
            .map(|i| mesh_chunk(&w, None, w.chunk_pos(i)).len())
            .sum();
        let shell = 2 * (32 * 16 + 32 * 16 + 16 * 16);
        assert_eq!(total, shell);
    }

    #[test]
    fn negative_chunks_mesh_with_local_coordinates() {
        let mut w = SparseWorld::new();
        w.set(-1, -1, -1, 4);
        let chunk = ChunkPos::new(-1, -1, -1);
        let faces = mesh_chunk(&w, None, chunk);
        assert_eq!(faces.len(), 6);
        assert!(
            faces
                .iter()
                .all(|f| f.local() == [15, 15, 15] && f.ao() == [3; 4])
        );
        // El vecino en el chunk 0 tapa la cara +X.
        w.set(0, -1, -1, 4);
        assert_eq!(mesh_chunk(&w, None, chunk).len(), 5);
        assert!(mesh_chunk(&w, None, ChunkPos::new(-2, -1, -1)).is_empty());
    }

    #[test]
    fn same_shape_meshes_identically_anywhere() {
        let build = |origin: [i32; 3]| {
            let mut w = SparseWorld::new();
            for (dx, dy, dz) in [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 0, 1)] {
                w.set(origin[0] + dx, origin[1] + dy, origin[2] + dz, 2);
            }
            mesh_chunk(&w, None, ChunkPos::containing(origin))
        };
        let near = build([0, 0, 0]);
        assert_eq!(build([7_499_984, -4096, -7_500_000]), near);
        assert_eq!(build([-7_500_000, 1008, 7_499_984]), near);
    }

    #[test]
    fn floor_next_to_wall_is_occluded() {
        let mut w = VoxelWorld::new([16, 16, 16]).unwrap();
        w.fill([0, 0, 0], [15, 0, 15], 1);
        w.set(5, 1, 5, 1);
        let top_of = |x: u32, z: u32| {
            mesh_chunk(&w, None, ChunkPos::default())
                .into_iter()
                .find(|f| f.local() == [x, 0, z] && f.direction() == Direction::PosY)
                .unwrap()
        };
        // El suelo junto al bloque tiene dos esquinas ocluidas.
        let ao = top_of(6, 5).ao();
        assert_eq!(ao.iter().filter(|&&o| o < 3).count(), 2, "{ao:?}");
        // Lejos del bloque no hay oclusión.
        assert_eq!(top_of(12, 12).ao(), [3; 4]);
    }
}
