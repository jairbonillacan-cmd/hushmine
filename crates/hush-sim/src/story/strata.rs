//! Estratos geológicos y generación procedural del mundo del modo historia.
//!
//! Adaptación del documento de diseño (sección 7) al mundo finito de Grisú:
//! las cotas se reparten proporcionalmente a la altura del mapa. Incluye
//! cuevas tubulares y bóvedas, vetas por estrato, lagos de magma, bolsas de
//! agua, coníferas de madera fósil, galerías abandonadas apuntaladas y el
//! pozo del ascensor averiado con sus válvulas de gas.

use grisu_voxel::{VoxelWorld, WorldError};
use serde::{Deserialize, Serialize};

use super::noise::value_noise2;
use crate::{BlockKind, Rng};

/// Parámetros del mundo del modo historia.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryTerrain {
    /// Tamaño en bloques (múltiplos de 16, máximo 128 por eje).
    pub size: [u32; 3],
    /// Bloques de aire sobre la superficie media.
    pub sky: u32,
}

impl Default for StoryTerrain {
    fn default() -> Self {
        Self {
            size: [96, 96, 96],
            sky: 20,
        }
    }
}

/// Estrato geológico activo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stratum {
    /// Lecho infranqueable: roca madre indestructible.
    Bedrock,
    /// Manto basáltico: basalto negro, lagos de magma, tungsteno y diamante.
    BasalticMantle,
    /// Estrato abisal: pizarra bituminosa, oro y gemas.
    Abyssal,
    /// Estrato cristalino: granito denso, hierro y agua subterránea.
    Crystalline,
    /// Estrato sedimentario: caliza blanda, carbón y cursos freáticos.
    Sedimentary,
    /// Superficie y colinas: suelo vegetal y coníferas.
    Surface,
}

impl Stratum {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bedrock => "Lecho de roca madre",
            Self::BasalticMantle => "Manto basáltico",
            Self::Abyssal => "Estrato abisal",
            Self::Crystalline => "Estrato cristalino",
            Self::Sedimentary => "Estrato sedimentario",
            Self::Surface => "Superficie",
        }
    }
}

/// Bandas verticales del mundo, calculadas sobre la altura del mapa.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bands {
    pub surface: i32,
    pub sedimentary: i32,
    pub crystalline: i32,
    pub abyssal: i32,
    pub basaltic: i32,
    pub bedrock: i32,
}

impl Bands {
    /// Reparto proporcional de las cotas del documento sobre un mundo de
    /// altura `height` con `sky` bloques de cielo.
    pub fn new(height: u32, sky: u32) -> Self {
        let h = height as i32;
        let surface = h - sky as i32;
        Self {
            surface,
            // Bajo la superficie: mitad sedimentario, luego cristalino,
            // abisal y basáltico hasta el lecho de roca madre.
            sedimentary: surface * 3 / 4,
            crystalline: surface / 2,
            abyssal: surface / 4,
            basaltic: surface / 12 + 2,
            bedrock: 2,
        }
    }

    pub fn at(&self, y: i32) -> Stratum {
        if y < self.bedrock {
            Stratum::Bedrock
        } else if y < self.basaltic {
            Stratum::BasalticMantle
        } else if y < self.abyssal {
            Stratum::Abyssal
        } else if y < self.crystalline {
            Stratum::Crystalline
        } else if y < self.sedimentary {
            Stratum::Sedimentary
        } else {
            Stratum::Surface
        }
    }
}

/// Estructuras fijas del Pozo Ancestral que la expedición debe reparar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AncestralWell {
    /// Centro de la cámara del ascensor.
    pub chamber: [i32; 3],
    /// Las 4 válvulas de gas grisú (bloques de obsidiana que hay que picar
    /// y sellar).
    pub valves: [[i32; 3]; 4],
    /// Los 4 engranajes maestros (bloques de roca a reparar con materiales).
    pub gears: [[i32; 3]; 4],
    /// Núcleo del motor del ascensor.
    pub core: [i32; 3],
    /// Cota a la que hay que volver para ganar (borde inferior de superficie).
    pub escape_y: i32,
}

/// Resultado de la generación: mundo y coordenadas de sus estructuras.
pub struct GeneratedStory {
    pub world: VoxelWorld,
    pub bands: Bands,
    pub well: AncestralWell,
    /// Punto de aparición en la superficie (bloques).
    pub spawn: [i32; 3],
}

/// Genera el mundo del modo historia a partir de la semilla.
pub fn generate(seed: u64, config: &StoryTerrain) -> Result<GeneratedStory, WorldError> {
    let mut world = VoxelWorld::new(config.size)?;
    let [sx, sy, sz] = config.size.map(|s| s as i32);
    let bands = Bands::new(config.size[1], config.sky);
    let surface = bands.surface;

    // Relieve suave de la superficie.
    let heights = Heightmap::new(&mut Rng::new(seed ^ 0x57E5_0000_0000_0001), sx, sz);

    for z in 0..sz {
        for x in 0..sx {
            let top = (surface + heights.at(x, z)).min(sy - 2);
            for y in 0..=top {
                let block = column_block(seed, x, y, z, top, &bands);
                world.set(x, y, z, block.id());
            }
        }
    }

    // Cuevas, vetas y estructuras.
    carve_caves(seed, &mut world, &bands);
    scatter_veins(seed, &mut world, &bands);
    let well = build_ancestral_well(&mut world, &bands, seed);
    build_galleries(seed, &mut world, &bands);
    plant_trees(seed, &mut world, &bands);

    // Aparición: primera columna con suelo real cerca del centro (el pozo
    // ancestral perfora la del medio con su chimenea de ventilación).
    let mut spawn = [sx / 2, surface + 3, sz / 2];
    'outer: for r in 0..(sx.min(sz) / 2).max(1) {
        for &(dx, dz) in &[
            (r, 0),
            (-r, 0),
            (0, r),
            (0, -r),
            (r, r),
            (-r, -r),
            (r, -r),
            (-r, r),
        ] {
            let (x, z) = (sx / 2 + dx, sz / 2 + dz);
            if x < 2 || z < 2 || x >= sx - 2 || z >= sz - 2 {
                continue;
            }
            let Some(ground) = (2..surface + 4).rev().find(|&y| world.is_solid(x, y, z)) else {
                continue;
            };
            if ground < surface - 4 {
                continue; // Fragua del pozo o cueva: no es superficie.
            }
            spawn = [x, ground + 1, z];
            break 'outer;
        }
    }
    Ok(GeneratedStory {
        world,
        bands,
        well,
        spawn,
    })
}

/// Bloque natural de una columna según la cota.
fn column_block(seed: u64, x: i32, y: i32, z: i32, top: i32, bands: &Bands) -> BlockKind {
    match bands.at(y) {
        Stratum::Bedrock => BlockKind::HardRock,
        Stratum::Surface => {
            if y > top - 4 {
                BlockKind::Dirt
            } else {
                BlockKind::Rock
            }
        }
        Stratum::Sedimentary => BlockKind::Rock,
        Stratum::Crystalline => {
            if value_noise2(seed, 0x0BE5, x, y, z, 12) < -90 {
                BlockKind::HardRock
            } else {
                BlockKind::Rock
            }
        }
        Stratum::Abyssal => BlockKind::Rock,
        Stratum::BasalticMantle => {
            if y < bands.basaltic - 2 && value_noise2(seed, 0x1A7A, x, y, z, 8) < -60 {
                BlockKind::Lava
            } else {
                BlockKind::HardRock
            }
        }
    }
}

/// Cuevas tubulares (dos ruidos con banda estrecha) y bóvedas grandes.
fn carve_caves(seed: u64, world: &mut VoxelWorld, bands: &Bands) {
    let [sx, _, sz] = world.size().map(|s| s as i32);
    for y in bands.bedrock + 1..bands.surface - 2 {
        for z in 0..sz {
            for x in 0..sx {
                let n1 = value_noise2(seed, 0xCA7E, x, y, z, 28);
                let n2 = value_noise2(seed, 0xCA7F, x, y, z, 28);
                // Túneles: intersección de dos bandas estrechas.
                let tube = n1.abs() < 22 && n2.abs() < 22;
                // Bóvedas: densidad negativa de baja frecuencia, más anchas
                // cuanto más hondo.
                let room_for_vault = y < bands.crystalline;
                let vault = room_for_vault && value_noise2(seed, 0xB0_17, x, y, z, 36) < -78;
                if tube || vault {
                    world.set(x, y, z, BlockKind::Air.id());
                }
            }
        }
    }
}

/// Vetas minerales por estrato: carbón arriba, hierro, oro y diamante.
fn scatter_veins(seed: u64, world: &mut VoxelWorld, bands: &Bands) {
    let [sx, _, sz] = world.size().map(|s| s as i32);
    let mut rng = Rng::new(seed ^ 0x0E0E_0000_0000_0002);
    let volume = sx * sz * (bands.surface - bands.bedrock);
    // (bloque, cotas de aparición, vetas, largo de veta)
    let plans = [
        (
            BlockKind::CoalOre,
            bands.crystalline..bands.sedimentary,
            volume / 900,
            14,
        ),
        (
            BlockKind::Ore,
            bands.abyssal..bands.crystalline,
            volume / 1_500,
            12,
        ),
        (
            BlockKind::Gold,
            bands.basaltic..bands.abyssal,
            volume / 3_500,
            8,
        ),
        (
            BlockKind::Diamond,
            bands.bedrock + 1..bands.basaltic,
            volume / 8_000,
            5,
        ),
        (
            BlockKind::Gravel,
            bands.abyssal..bands.sedimentary,
            volume / 1_800,
            10,
        ),
    ];
    for (block, range, count, length) in plans {
        if range.is_empty() {
            continue;
        }
        for _ in 0..count.max(1) {
            let mut p = [
                rng.below(sx as u32) as i32,
                range.start + rng.below((range.end - range.start) as u32) as i32,
                rng.below(sz as u32) as i32,
            ];
            for _ in 0..length {
                let current = world.get(p[0], p[1], p[2]);
                if current == BlockKind::Rock.id() || current == BlockKind::HardRock.id() {
                    world.set(p[0], p[1], p[2], block.id());
                }
                let axis = rng.below(3) as usize;
                p[axis] += if rng.below(2) == 0 { 1 } else { -1 };
                p[1] = p[1].clamp(range.start, range.end - 1);
            }
        }
    }
}

#[path = "strata_build.rs"]
mod build;
use build::{build_ancestral_well, build_galleries, plant_trees};

/// Relieve suave de la superficie (rejilla interpolada).
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
            .map(|_| rng.below(7) as i32 - 3)
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
        (total + n * n / 2) / (n * n)
    }
}

#[cfg(test)]
#[path = "strata_tests.rs"]
mod tests;
