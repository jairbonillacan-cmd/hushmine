//! Generación del monte Anima: un mapa nuevo en cada partida.
//!
//! Todo es determinista a partir de la semilla (enteros y [`Rng`]), para que
//! servidor y cliente obtengan exactamente el mismo terreno.

use grisu_voxel::{VoxelWorld, WorldError};

use crate::{BlockKind, Rng};

/// Parámetros del mapa.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TerrainConfig {
    /// Tamaño en bloques (múltiplos de 16, máximo 128).
    pub size: [u32; 3],
    /// Bloques de aire por encima de la superficie media.
    pub sky: u32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            size: [64, 64, 64],
            sky: 12,
        }
    }
}

/// Genera el terreno de una partida.
pub fn generate(seed: u64, config: &TerrainConfig) -> Result<VoxelWorld, WorldError> {
    let mut world = VoxelWorld::new(config.size)?;
    // Generadores separados por capa: cambiar una no altera las demás.
    let mut rng = Rng::new(seed ^ 0x7E44_A1A5_0000_0001);
    let [sx, sy, sz] = config.size.map(|s| s as i32);
    let surface = sy - config.sky as i32;
    let heights = Heightmap::new(&mut rng, sx, sz);
    let hard_rock_top = sy / 5;

    for z in 0..sz {
        for x in 0..sx {
            let top = surface + heights.at(x, z);
            let dirt_depth = 3 + (hash2(seed, x, z) % 3) as i32;
            for y in 0..=top.min(sy - 1) {
                let block = if y > top - dirt_depth {
                    BlockKind::Dirt
                } else if y < hard_rock_top + (hash2(seed ^ 1, x, z) % 3) as i32 {
                    BlockKind::HardRock
                } else {
                    BlockKind::Rock
                };
                world.set(x, y, z, block.id());
            }
        }
    }

    let volume = (sx * sy * sz) as u32;
    let mut veins = Rng::new(seed ^ 0x0E0E_0000_0000_0002);
    // Mena abundante en toda la roca; oro más hondo; diamante en el fondo.
    scatter(
        &mut world,
        &mut veins,
        BlockKind::Ore,
        volume / 3000,
        6..18,
        4..surface - 2,
    );
    scatter(
        &mut world,
        &mut veins,
        BlockKind::Gold,
        volume / 9000,
        3..8,
        4..surface - 8,
    );
    scatter(
        &mut world,
        &mut veins,
        BlockKind::Diamond,
        volume / 40000,
        1..3,
        2..surface / 3,
    );

    let mut caves = Rng::new(seed ^ 0xCA7E_0000_0000_0003);
    for _ in 0..(volume / 40000).max(1) {
        carve_cave(&mut world, &mut caves, 2..surface - 6);
    }

    let mut lava = Rng::new(seed ^ 0x1A7A_0000_0000_0004);
    for _ in 0..(volume / 30000).max(1) {
        place_lava_pocket(&mut world, &mut lava, 2..surface / 2);
    }
    Ok(world)
}

/// Relieve suave de la superficie: alturas aleatorias en una rejilla de 8
/// bloques, interpoladas en enteros. Varía entre -2 y +2.
struct Heightmap {
    cells: Vec<i32>,
    stride: i32,
}

const HEIGHT_STEP: i32 = 8;

impl Heightmap {
    fn new(rng: &mut Rng, sx: i32, sz: i32) -> Self {
        let stride = sx / HEIGHT_STEP + 2;
        let rows = sz / HEIGHT_STEP + 2;
        let cells = (0..stride * rows)
            .map(|_| rng.below(5) as i32 - 2)
            .collect();
        Self { cells, stride }
    }

    fn at(&self, x: i32, z: i32) -> i32 {
        let (gx, gz) = (x / HEIGHT_STEP, z / HEIGHT_STEP);
        let (fx, fz) = (x % HEIGHT_STEP, z % HEIGHT_STEP);
        let h = |i: i32, j: i32| self.cells[((gz + j) * self.stride + gx + i) as usize];
        let n = HEIGHT_STEP;
        let top = h(0, 0) * (n - fx) + h(1, 0) * fx;
        let bottom = h(0, 1) * (n - fx) + h(1, 1) * fx;
        let total = top * (n - fz) + bottom * fz;
        // Redondeo hacia el entero más cercano, también con negativos.
        (total + if total >= 0 { n * n / 2 } else { -n * n / 2 }) / (n * n)
    }
}

/// Coloca `count` vetas como paseos aleatorios que solo sustituyen roca o
/// tierra (nunca aire ni otras vetas).
fn scatter(
    world: &mut VoxelWorld,
    rng: &mut Rng,
    block: BlockKind,
    count: u32,
    length: std::ops::Range<u32>,
    height: std::ops::Range<i32>,
) {
    if height.is_empty() {
        return;
    }
    let [sx, _, sz] = world.size().map(|s| s as i32);
    for _ in 0..count.max(1) {
        let mut p = [
            rng.below(sx as u32) as i32,
            height.start + rng.below((height.end - height.start) as u32) as i32,
            rng.below(sz as u32) as i32,
        ];
        let steps = length.start + rng.below(length.end - length.start);
        for _ in 0..steps {
            let current = world.get(p[0], p[1], p[2]);
            if current == BlockKind::Rock.id() || current == BlockKind::Dirt.id() {
                world.set(p[0], p[1], p[2], block.id());
            }
            let axis = rng.below(3) as usize;
            p[axis] += if rng.below(2) == 0 { 1 } else { -1 };
        }
    }
}

/// Excava una cueva serpenteante de 3 bloques de sección.
fn carve_cave(world: &mut VoxelWorld, rng: &mut Rng, height: std::ops::Range<i32>) {
    if height.is_empty() {
        return;
    }
    let [sx, _, sz] = world.size().map(|s| s as i32);
    let mut p = [
        4 + rng.below((sx - 8) as u32) as i32,
        height.start + rng.below((height.end - height.start) as u32) as i32,
        4 + rng.below((sz - 8) as u32) as i32,
    ];
    let mut heading = rng.below(4);
    for _ in 0..(20 + rng.below(30)) {
        world.fill(
            [p[0] - 1, p[1] - 1, p[2] - 1],
            [p[0] + 1, p[1] + 1, p[2] + 1],
            0,
        );
        if rng.below(5) == 0 {
            heading = (heading + if rng.below(2) == 0 { 1 } else { 3 }) % 4;
        }
        match heading {
            0 => p[0] += 1,
            1 => p[2] += 1,
            2 => p[0] -= 1,
            _ => p[2] -= 1,
        }
        if rng.below(4) == 0 {
            p[1] += rng.below(3) as i32 - 1;
        }
        p[0] = p[0].clamp(2, sx - 3);
        p[1] = p[1].clamp(height.start, height.end - 1);
        p[2] = p[2].clamp(2, sz - 3);
    }
}

/// Bolsa de lava cerrada en la roca profunda.
fn place_lava_pocket(world: &mut VoxelWorld, rng: &mut Rng, height: std::ops::Range<i32>) {
    if height.is_empty() {
        return;
    }
    let [sx, _, sz] = world.size().map(|s| s as i32);
    let c = [
        3 + rng.below((sx - 6) as u32) as i32,
        height.start + rng.below((height.end - height.start) as u32) as i32,
        3 + rng.below((sz - 6) as u32) as i32,
    ];
    let r = 1 + rng.below(2) as i32;
    world.fill(
        [c[0] - r, c[1], c[2] - r],
        [c[0] + r, c[1], c[2] + r],
        BlockKind::Lava.id(),
    );
}

/// Hash entero de una columna, para detalles que no necesitan secuencia.
fn hash2(seed: u64, x: i32, z: i32) -> u64 {
    let mut rng = Rng::new(seed ^ ((x as u64) << 32) ^ (z as u64 & 0xFFFF_FFFF));
    rng.next_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(world: &VoxelWorld, block: BlockKind) -> usize {
        (0..world.chunk_count())
            .map(|i| {
                world
                    .chunk(i)
                    .as_bytes()
                    .iter()
                    .filter(|&&b| b == block.id())
                    .count()
            })
            .sum()
    }

    #[test]
    fn same_seed_same_world() {
        let config = TerrainConfig::default();
        let a = generate(42, &config).unwrap();
        let b = generate(42, &config).unwrap();
        assert_eq!(a.content_hash(), b.content_hash());
        let c = generate(43, &config).unwrap();
        assert_ne!(a.content_hash(), c.content_hash());
    }

    #[test]
    fn has_every_natural_block_and_sky() {
        let world = generate(7, &TerrainConfig::default()).unwrap();
        for block in BlockKind::ALL.into_iter().filter(|b| b.is_natural()) {
            assert!(count(&world, block) > 0, "falta {block:?}");
        }
        assert!(count(&world, BlockKind::Rock) > count(&world, BlockKind::Ore));
        assert!(count(&world, BlockKind::Ore) > count(&world, BlockKind::Gold));
        // El cielo está vacío.
        for x in 0..64 {
            assert!(!world.is_solid(x, 63, x));
        }
        // El fondo es sólido salvo cuevas: casi todo roca dura.
        assert!(world.is_solid(32, 0, 32) || world.is_solid(10, 0, 10));
    }

    #[test]
    fn surface_is_gentle() {
        let world = generate(3, &TerrainConfig::default()).unwrap();
        let top = |x: i32, z: i32| (0..64).rev().find(|&y| world.is_solid(x, y, z));
        for z in 0..64 {
            for x in 0..64 {
                let y = top(x, z).expect("columna sin suelo");
                assert!((49..=54).contains(&y), "altura {y} en ({x}, {z})");
            }
        }
    }

    #[test]
    fn large_map_fits() {
        let config = TerrainConfig {
            size: [96, 64, 96],
            sky: 12,
        };
        assert!(generate(1, &config).is_ok());
    }
}
