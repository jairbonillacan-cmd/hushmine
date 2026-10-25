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
            Self::Bedrock => "LECHO DE ROCA MADRE",
            Self::BasalticMantle => "MANTO BASALTICO",
            Self::Abyssal => "ESTRATO ABISAL",
            Self::Crystalline => "ESTRATO CRISTALINO",
            Self::Sedimentary => "ESTRATO SEDIMENTARIO",
            Self::Surface => "SUPERFICIE",
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

/// Excava el pozo del ascensor averiado en el fondo del mapa.
fn build_ancestral_well(world: &mut VoxelWorld, bands: &Bands, seed: u64) -> AncestralWell {
    let [sx, _, sz] = world.size().map(|s| s as i32);
    let mut rng = Rng::new(seed ^ 0xA5CE_0000_0000_0003);
    let cx = sx / 2 + rng.below(9) as i32 - 4;
    let cz = sz / 2 + rng.below(9) as i32 - 4;
    let floor = bands.bedrock + 1;
    let chamber_y = floor + 1;
    // Chimenea de ventilación hasta el estrato sedimentario.
    world.fill(
        [cx - 1, floor, cz - 1],
        [cx + 1, bands.sedimentary, cz + 1],
        BlockKind::Air.id(),
    );
    // Cámara del ascensor: 7×5×7 despejada.
    world.fill(
        [cx - 3, chamber_y, cz - 3],
        [cx + 3, chamber_y + 4, cz + 3],
        BlockKind::Air.id(),
    );
    // Suelo de obsidiana y marco de puntales.
    world.fill(
        [cx - 3, chamber_y - 1, cz - 3],
        [cx + 3, chamber_y - 1, cz + 3],
        BlockKind::Obsidian.id(),
    );
    for &(dx, dz) in &[(-3, -3), (3, -3), (-3, 3), (3, 3)] {
        for dy in 0..4 {
            world.set(cx + dx, chamber_y + dy, cz + dz, BlockKind::PropBeam.id());
        }
    }
    // Núcleo del motor en el centro.
    world.set(cx, chamber_y, cz, BlockKind::PropBeam.id());
    // Válvulas de gas (obsidiana a picar) y engranajes (roca a reparar).
    let valves = [
        [cx - 2, chamber_y, cz - 2],
        [cx + 2, chamber_y, cz - 2],
        [cx - 2, chamber_y, cz + 2],
        [cx + 2, chamber_y, cz + 2],
    ];
    for v in valves {
        world.set(v[0], v[1], v[2], BlockKind::Obsidian.id());
    }
    let gears = [
        [cx - 2, chamber_y + 2, cz],
        [cx + 2, chamber_y + 2, cz],
        [cx, chamber_y + 2, cz - 2],
        [cx, chamber_y + 2, cz + 2],
    ];
    for g in gears {
        world.set(g[0], g[1], g[2], BlockKind::Rock.id());
    }
    AncestralWell {
        chamber: [cx, chamber_y, cz],
        valves,
        gears,
        core: [cx, chamber_y, cz],
        escape_y: bands.sedimentary,
    }
}

/// Galerías de prospección abandonadas: túneles apuntalados con raíles.
fn build_galleries(seed: u64, world: &mut VoxelWorld, bands: &Bands) {
    let [sx, _, sz] = world.size().map(|s| s as i32);
    let mut rng = Rng::new(seed ^ 0x6A1E_0000_0000_0004);
    let depths = [
        bands.sedimentary - 3,
        bands.crystalline + 3,
        bands.abyssal + 3,
    ];
    for &depth in &depths {
        let mut x = 8 + rng.below((sx - 16) as u32) as i32;
        let mut z = 8 + rng.below((sz - 16) as u32) as i32;
        let mut heading = rng.below(4);
        for step in 0..40 + rng.below(30) {
            // Sección de 3 de ancho por 3 de alto.
            world.fill(
                [x - 1, depth, z - 1],
                [x + 1, depth + 2, z + 1],
                BlockKind::Air.id(),
            );
            // Puntales cada 4 pasos y raíl en el suelo.
            if step % 4 == 0 {
                for &(dx, dz) in &[(-1, -1), (1, 1)] {
                    world.set(x + dx, depth + 2, z + dz, BlockKind::PropBeam.id());
                }
            }
            world.set(x, depth, z, BlockKind::Rail.id());
            // Arcón con suministros en una alcoba lateral periódica.
            if step % 17 == 16 && x > 2 && x < sx - 3 && z > 2 && z < sz - 3 {
                let side = if heading.is_multiple_of(2) {
                    [0, 2]
                } else {
                    [2, 0]
                };
                let alcove = [x + side[0], depth, z + side[1]];
                world.set(alcove[0], alcove[1], alcove[2], BlockKind::PropBeam.id());
                world.set(alcove[0], alcove[1] + 1, alcove[2], BlockKind::Chest.id());
            }
            if rng.below(4) == 0 {
                heading = (heading + 1 + rng.below(3)) % 4;
            }
            match heading {
                0 => x += 1,
                1 => z += 1,
                2 => x -= 1,
                _ => z -= 1,
            }
            x = x.clamp(4, sx - 5);
            z = z.clamp(4, sz - 5);
        }
    }
}

/// Coníferas de madera fósil en la superficie.
fn plant_trees(seed: u64, world: &mut VoxelWorld, bands: &Bands) {
    let [sx, sy, sz] = world.size().map(|s| s as i32);
    let mut rng = Rng::new(seed ^ 0x7EE5_0000_0000_0005);
    for _ in 0..(sx * sz / 90) {
        let x = 4 + rng.below((sx - 8) as u32) as i32;
        let z = 4 + rng.below((sz - 8) as u32) as i32;
        let Some(ground) = (2..sy - 1).rev().find(|&y| world.is_solid(x, y, z)) else {
            continue;
        };
        if ground < bands.surface - 2 || world.get(x, ground, z) != BlockKind::Dirt.id() {
            continue;
        }
        let trunk = 3 + rng.below(3) as i32;
        for dy in 1..=trunk {
            world.set(x, ground + dy, z, BlockKind::Log.id());
        }
    }
}

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
mod tests {
    use super::*;

    fn block_count(world: &VoxelWorld, block: BlockKind) -> usize {
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
    fn same_seed_same_world_and_structures() {
        let cfg = StoryTerrain::default();
        let a = generate(42, &cfg).unwrap();
        let b = generate(42, &cfg).unwrap();
        assert_eq!(a.world.content_hash(), b.world.content_hash());
        assert_eq!(a.well, b.well);
        assert_eq!(a.spawn, b.spawn);
        let c = generate(43, &cfg).unwrap();
        assert_ne!(a.world.content_hash(), c.world.content_hash());
    }

    #[test]
    fn strata_lay_out_proportionally() {
        let bands = Bands::new(96, 20);
        assert_eq!(bands.surface, 76);
        assert_eq!(bands.at(80), Stratum::Surface);
        // Las cotas de corte: 57 (sedimentario), 38 (cristalino), 19
        // (abisal) y 8 (basáltico).
        assert_eq!(bands.sedimentary, 57);
        assert_eq!(bands.crystalline, 38);
        assert_eq!(bands.abyssal, 19);
        assert_eq!(bands.at(50), Stratum::Sedimentary);
        assert_eq!(bands.at(30), Stratum::Crystalline);
        assert_eq!(bands.at(15), Stratum::Abyssal);
        assert_eq!(bands.at(5), Stratum::BasalticMantle);
        assert_eq!(bands.at(1), Stratum::Bedrock);
    }

    #[test]
    fn every_story_resource_appears() {
        let story = generate(7, &StoryTerrain::default()).unwrap();
        for block in [
            BlockKind::Dirt,
            BlockKind::Rock,
            BlockKind::HardRock,
            BlockKind::CoalOre,
            BlockKind::Ore,
            BlockKind::Gold,
            BlockKind::Diamond,
            BlockKind::Lava,
            BlockKind::Gravel,
            BlockKind::Log,
            BlockKind::PropBeam,
            BlockKind::Rail,
            BlockKind::Chest,
            BlockKind::Obsidian,
        ] {
            assert!(block_count(&story.world, block) > 0, "falta {block:?}");
        }
    }

    #[test]
    fn caves_connect_the_depths() {
        let story = generate(9, &StoryTerrain::default()).unwrap();
        let w = &story.world;
        // Alguna columna tiene aire por debajo del estrato cristalino.
        let mut air_below = 0;
        for x in (0..96).step_by(3) {
            for z in (0..96).step_by(3) {
                for y in 8..40 {
                    if !w.is_solid(x, y, z) {
                        air_below += 1;
                        break;
                    }
                }
            }
        }
        assert!(air_below > 10, "casi sin cuevas: {air_below}");
    }

    #[test]
    fn ancestral_well_is_carved_and_spawn_is_on_surface() {
        let story = generate(11, &StoryTerrain::default()).unwrap();
        let well = story.well;
        let w = &story.world;
        assert!(!w.is_solid(well.chamber[0], well.chamber[1] + 1, well.chamber[2]));
        for valve in well.valves {
            assert_eq!(
                w.get(valve[0], valve[1], valve[2]),
                BlockKind::Obsidian.id()
            );
        }
        assert!(story.spawn[1] >= story.bands.surface - 1);
        // Aparece con suelo bajo los pies.
        assert!(w.is_solid(story.spawn[0], story.spawn[1] - 1, story.spawn[2]));
    }
}
