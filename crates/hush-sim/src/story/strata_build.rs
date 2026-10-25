//! Estructuras del mundo del modo Historia: el pozo ancestral, las galerías antiguas y los árboles.

use super::*;

/// Excava el pozo del ascensor averiado en el fondo del mapa.
pub(super) fn build_ancestral_well(
    world: &mut VoxelWorld,
    bands: &Bands,
    seed: u64,
) -> AncestralWell {
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
pub(super) fn build_galleries(seed: u64, world: &mut VoxelWorld, bands: &Bands) {
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
pub(super) fn plant_trees(seed: u64, world: &mut VoxelWorld, bands: &Bands) {
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
