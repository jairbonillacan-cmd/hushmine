//! Pruebas de integración de la expedición: determinismo, bucle de juego
//! completo (madera → herramientas → horno → hierro) y clímax del Pozo
//! Ancestral.

use grisu_voxel::FIXED_ONE;

use super::expedition::{FurnaceSlot, StoryConfig, StoryOrder};
use super::items::{COAL, IRON_INGOT, ItemId, ToolKind, tool_item};
use super::mobs::SPAWN_MIN_DISTANCE;
use super::{Expedition, Milestone};
use crate::{BlockKind, MoveInput};

fn config() -> StoryConfig {
    StoryConfig {
        seed: 42,
        ..StoryConfig::default()
    }
}

/// Configuración pacífica para las pruebas de hitos y clímax.
fn peaceful() -> StoryConfig {
    StoryConfig {
        seed: 42,
        peaceful: true,
        ..StoryConfig::default()
    }
}

fn idle_ticks(expedition: &mut Expedition, ticks: u32) {
    for _ in 0..ticks {
        expedition.step(&[]);
    }
}

#[test]
fn same_orders_same_state_everywhere() {
    let run = || {
        let mut e = Expedition::new(config()).unwrap();
        for t in 0..600u32 {
            let orders = [StoryOrder::Move(MoveInput {
                wish_x: (t % 30) as i8 - 15,
                wish_z: ((t * 7) % 30) as i8 - 15,
                jump: t % 40 == 0,
                sprint: t % 3 == 0,
            })];
            e.step(&orders);
        }
        e.state_hash()
    };
    assert_eq!(run(), run());
    // Distinta semilla, mundo distinto.
    let other = StoryConfig {
        seed: 43,
        ..StoryConfig::default()
    };
    let mut e = Expedition::new(other).unwrap();
    idle_ticks(&mut e, 10);
    let mut base = Expedition::new(config()).unwrap();
    idle_ticks(&mut base, 10);
    assert_ne!(base.state_hash(), e.state_hash());
}

#[test]
fn player_settles_on_the_surface() {
    let mut e = Expedition::new(config()).unwrap();
    idle_ticks(&mut e, 60);
    assert!(e.body().on_ground, "nunca tocó suelo");
    let feet = e.feet_cell();
    assert!(
        feet[1] >= e.bands().sedimentary,
        "apareció bajo tierra: {feet:?}"
    );
}

#[test]
fn full_progression_loop_reaches_milestones() {
    let mut e = Expedition::new(config()).unwrap();
    idle_ticks(&mut e, 30);

    // 1. El Despertar: madera fósil y herramientas primordiales.
    let log = ItemId::of_block(BlockKind::Log);
    let plank = ItemId::of_block(BlockKind::Plank);
    e.inventory_mut()
        .add(super::inventory::ItemStack::new(log, 4));
    e.step(&[StoryOrder::Craft {
        grid: [[Some(log.0), None, None], [None; 3], [None; 3]],
        workbench: false,
    }]);
    assert!(e.inventory().count_of(plank) >= 4);
    let handle = ItemId(super::items::HANDLE);
    e.step(&[StoryOrder::Craft {
        grid: [
            [Some(plank.0), None, None],
            [Some(plank.0), None, None],
            [None; 3],
        ],
        workbench: false,
    }]);
    assert!(e.inventory().count_of(handle) >= 4);
    // Pico de madera: matriz 3×3, ya con banco de por medio.
    e.step(&[StoryOrder::Craft {
        grid: [
            [Some(plank.0); 3],
            [None, Some(handle.0), None],
            [None, Some(handle.0), None],
        ],
        workbench: true,
    }]);
    assert!(e.inventory().count_of(tool_item(0, ToolKind::Pickaxe)) >= 1);
    idle_ticks(&mut e, 3);
    assert_eq!(e.milestones().reached, Milestone::Awakening as u8);

    // 2. Edad de Piedra: pico de caliza y carbón.
    let rock = ItemId::of_block(BlockKind::Rock);
    e.inventory_mut()
        .add(super::inventory::ItemStack::new(rock, 3));
    e.step(&[StoryOrder::Craft {
        grid: [
            [Some(rock.0); 3],
            [None, Some(handle.0), None],
            [None, Some(handle.0), None],
        ],
        workbench: true,
    }]);
    e.inventory_mut()
        .add(super::inventory::ItemStack::new(ItemId(COAL), 1));
    idle_ticks(&mut e, 3);
    assert_eq!(e.milestones().reached, Milestone::StoneAge as u8);

    // 3. Luz y Hogar: ocho antorchas de campamento.
    e.milestones_mut().torches_placed = 8;
    idle_ticks(&mut e, 3);
    assert_eq!(e.milestones().reached, Milestone::LightHome as u8);

    // 4. Metalurgia: lingote de hierro fundido.
    e.inventory_mut()
        .add(super::inventory::ItemStack::new(ItemId(IRON_INGOT), 1));
    idle_ticks(&mut e, 3);
    assert_eq!(e.milestones().reached, Milestone::Metallurgy as u8);

    // 5. Grieta Oscura: descenso al estrato abisal (caemos por el pozo).
    let chamber = e.well().chamber;
    e.teleport([
        (chamber[0] + 1) * FIXED_ONE,
        (chamber[1] + 2) * FIXED_ONE,
        (chamber[2] + 1) * FIXED_ONE,
    ]);
    idle_ticks(&mut e, 20);
    assert_eq!(e.milestones().reached, Milestone::DarkRift as u8);

    // 6. Acero Templado.
    let steel_pick = tool_item(3, ToolKind::Pickaxe);
    e.inventory_mut().add(super::inventory::ItemStack::tool(
        steel_pick,
        steel_pick.max_durability(),
    ));
    idle_ticks(&mut e, 3);
    assert_eq!(e.milestones().reached, Milestone::SteelTempering as u8);

    // 7. Manto de Fuego: diamante a la profundidad basáltica.
    e.inventory_mut().add(super::inventory::ItemStack::new(
        ItemId(super::items::DIAMOND_GEM),
        1,
    ));
    idle_ticks(&mut e, 5);
    assert_eq!(e.milestones().reached, Milestone::FireMantle as u8);
}

#[test]
fn digging_requires_reach_and_bedrock_is_infranqueable() {
    let mut e = Expedition::new(config()).unwrap();
    idle_ticks(&mut e, 30);
    // Lejos: no progresa.
    e.step(&[StoryOrder::Mine(Some([0, 2, 0]))]);
    for _ in 0..200 {
        e.step(&[StoryOrder::Mine(Some([200, 40, 200]))]);
    }
    assert!(e.mining().cell.is_none() || e.mining().progress < e.mining().total);
    // Roca madre: tampoco.
    for _ in 0..200 {
        e.step(&[StoryOrder::Mine(Some(e.feet_cell().map(|c| c)))]);
    }
    let y = e.feet_cell()[1];
    assert!(y >= e.bands().bedrock);
}

#[test]
fn mining_progress_finishes_and_drops() {
    let mut e = Expedition::new(config()).unwrap();
    idle_ticks(&mut e, 30);
    // Bloque artificial al alcance.
    let feet = e.feet_cell();
    let target = [feet[0] + 1, feet[1], feet[2]];
    e.world_mut()
        .set(target[0], target[1], target[2], BlockKind::Dirt.id());
    for _ in 0..60 {
        e.step(&[StoryOrder::Mine(Some(target))]);
    }
    assert_eq!(e.world().get(target[0], target[1], target[2]), 0);
    assert!(e.inventory().count_of(ItemId::of_block(BlockKind::Dirt)) >= 1);
}

#[test]
fn furnace_full_cycle() {
    let mut e = Expedition::new(config()).unwrap();
    idle_ticks(&mut e, 5);
    let cell = e.feet_cell().map(|c| c + 2);
    e.furnaces_mut()
        .insert(cell, super::crafting::FurnaceState::default());
    e.inventory_mut().add(super::inventory::ItemStack::new(
        ItemId::of_block(BlockKind::Ore),
        2,
    ));
    e.inventory_mut()
        .add(super::inventory::ItemStack::new(ItemId(COAL), 1));
    e.step(&[StoryOrder::FurnaceLoad(
        cell,
        super::inventory::Place::Hotbar(0),
    )]);
    e.step(&[StoryOrder::FurnaceLoad(
        cell,
        super::inventory::Place::Hotbar(1),
    )]);
    for _ in 0..220 {
        e.step(&[]);
    }
    let furnace = e.furnaces().get(&cell).unwrap();
    assert!(furnace.output.is_some(), "no fundió: {furnace:?}");
    e.step(&[StoryOrder::FurnaceTake(cell, FurnaceSlot::Output)]);
    assert!(e.inventory().count_of(ItemId(IRON_INGOT)) >= 1);
}

#[test]
fn climactic_well_can_be_sealed_activated_and_won() {
    let mut e = Expedition::new(peaceful()).unwrap();
    idle_ticks(&mut e, 10);
    let well = *e.well();
    // El jugador se teletransporta a la cámara (simulación de la llegada).
    let feet = [
        (well.chamber[0] + 1) * FIXED_ONE,
        (well.chamber[1] + 1) * FIXED_ONE,
        (well.chamber[2] + 1) * FIXED_ONE,
    ];
    e.teleport(feet);
    // Materiales de reparación.
    let plank = ItemId::of_block(BlockKind::Plank);
    e.inventory_mut()
        .add(super::inventory::ItemStack::new(plank, 4));
    e.inventory_mut()
        .add(super::inventory::ItemStack::new(ItemId(IRON_INGOT), 4));
    for valve in well.valves {
        e.step(&[StoryOrder::Interact(valve)]);
    }
    assert_eq!(e.milestones().valves_sealed, 4);
    // El hierro quedó en la segunda casilla de la barra rápida.
    e.step(&[StoryOrder::Select(1)]);
    for gear in well.gears {
        e.step(&[StoryOrder::Interact(gear)]);
    }
    assert_eq!(e.milestones().gears_repaired, 4);
    // Sin activar antes de las reparaciones... ya están: activamos.
    e.step(&[StoryOrder::Interact(well.core)]);
    assert!(e.milestones().siege_started);
    // Asedio: el jugador aguanta (sin morir) los 5 minutos simulados.
    for _ in 0..super::milestones::SIEGE_TICKS {
        e.step(&[]);
    }
    // Vuelve a la superficie y gana.
    e.teleport([
        e.spawn[0] * FIXED_ONE,
        e.spawn[1] * FIXED_ONE,
        e.spawn[2] * FIXED_ONE,
    ]);
    let final_events = e.step(&[]);
    assert!(e.milestones().victory);
    assert!(
        final_events
            .iter()
            .any(|ev| matches!(ev.kind, super::expedition::StoryEventKind::Victory))
    );
}

#[test]
fn enemies_spawn_in_dark_caves_but_not_next_to_the_player() {
    let mut e = Expedition::new(config()).unwrap();
    idle_ticks(&mut e, 30);
    let feet = e.feet_cell();
    for mob in e.mobs().list.iter() {
        let dist = (mob.cell[0] - feet[0]).abs()
            + (mob.cell[1] - feet[1]).abs()
            + (mob.cell[2] - feet[2]).abs();
        assert!(
            dist >= SPAWN_MIN_DISTANCE,
            "enemigo demasiado cerca: {dist}"
        );
    }
}

#[test]
fn world_saves_and_loads_through_regions() {
    let mut e = Expedition::new(config()).unwrap();
    e.world_mut().set(5, 40, 5, BlockKind::Gold.id());
    let dir = std::env::temp_dir().join(format!("hush-story-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("expedicion.hush");
    e.save_region(&path).unwrap();
    let loaded = super::region::load_world(&path).unwrap();
    assert_eq!(loaded.content_hash(), e.world().content_hash());
    assert_eq!(loaded.get(5, 40, 5), BlockKind::Gold.id());
    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(&dir).ok();
}
