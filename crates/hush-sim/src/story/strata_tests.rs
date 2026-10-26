//! Pruebas de `strata`.

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
