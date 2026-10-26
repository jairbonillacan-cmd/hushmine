//! Pruebas de `crafting`.

use super::*;
use crate::BlockKind;

fn cell(id: Option<u16>) -> Option<ItemStack> {
    id.map(|i| ItemStack::new(ItemId(i), 1))
}

#[test]
fn portable_recipes_match_anywhere_in_the_grid() {
    // 1 tronco -> 4 tablones, colocado en la esquina inferior derecha.
    let mut grid = [[None; GRID]; GRID];
    grid[2][2] = cell(Some(u16::from(BlockKind::Log.id())));
    let recipe = match_recipe(&grid.map(|row| row.map(|s| s.map(|s| s.id))), false).unwrap();
    assert_eq!(recipe.name, "planks");

    // Carbón sobre mango -> antorchas.
    let mut grid = [[None; GRID]; GRID];
    grid[0][1] = cell(Some(COAL));
    grid[1][1] = cell(Some(HANDLE));
    let ids = grid.map(|row| row.map(|s| s.map(|s| s.id)));
    assert_eq!(match_recipe(&ids, false).map(|r| r.name), Some("torches"));

    // 4 tablones en cuadrado -> banco de trabajo.
    let mut grid = [[None; GRID]; GRID];
    for row in grid.iter_mut().take(2) {
        for slot in row.iter_mut().take(2) {
            *slot = cell(Some(PLANK_ITEM));
        }
    }
    let ids = grid.map(|row| row.map(|s| s.map(|s| s.id)));
    assert_eq!(match_recipe(&ids, false).map(|r| r.name), Some("workbench"));
}

#[test]
fn pickaxe_requires_workbench_and_material() {
    let m = u16::from(BlockKind::Rock.id());
    let mut grid = [[None; GRID]; GRID];
    for slot in grid[0].iter_mut() {
        *slot = cell(Some(m));
    }
    grid[1][1] = cell(Some(HANDLE));
    grid[2][1] = cell(Some(HANDLE));
    let ids = grid.map(|row| row.map(|s| s.map(|s| s.id)));
    // Sin banco, el 3×3 no encaja.
    assert_eq!(match_recipe(&ids, false), None);
    let recipe = match_recipe(&ids, true).unwrap();
    assert_eq!(recipe.name, "pickaxe");
    assert_eq!(recipe.output.id, tool_item(1, ToolKind::Pickaxe));
}

#[test]
fn craft_consumes_one_per_used_cell() {
    let mut grid = [[None; GRID]; GRID];
    let log = u16::from(crate::BlockKind::Log.id());
    grid[0][0] = Some(ItemStack::new(ItemId(log), 3));
    let out = craft(&mut grid, false).unwrap();
    assert_eq!(out.id, ItemId(PLANK_ITEM));
    assert_eq!(out.count, 4);
    assert_eq!(grid[0][0].map(|s| s.count), Some(2));
}

#[test]
fn furnace_smelts_after_two_hundred_ticks() {
    let mut furnace = FurnaceState::new(FurnaceKind::Stone);
    let mut ore = ItemStack::new(ItemId::of_block(BlockKind::Ore), 1);
    assert!(feed_furnace(&mut furnace, &mut ore));
    let mut coal = ItemStack::new(ItemId(COAL), 1);
    assert!(feed_furnace(&mut furnace, &mut coal));
    let mut smelted = None;
    for t in 0..SMELT_TICKS + 2 {
        smelted = furnace.step();
        if smelted.is_some() {
            assert_eq!(t + 1, SMELT_TICKS, "fundió en el tick {}", t + 1);
            break;
        }
    }
    let out = smelted.expect("no fundió");
    assert_eq!(out.id, ItemId(IRON_INGOT));
    assert_eq!(furnace.fuel, None);
}

#[test]
fn blast_furnace_is_double_speed_and_picky() {
    let mut blast = FurnaceState::new(FurnaceKind::Blast);
    let mut wood = ItemStack::new(ItemId(PLANK_ITEM), 1);
    assert!(feed_furnace(&mut blast, &mut wood)); // como combustible
    let mut ore = ItemStack::new(ItemId::of_block(BlockKind::Ore), 1);
    assert!(feed_furnace(&mut blast, &mut ore));
    let ticks = (1..).find(|_| blast.step().is_some()).unwrap();
    assert_eq!(ticks, SMELT_TICKS / 2);
}

#[test]
fn smelt_result_covers_metals_only() {
    assert_eq!(
        smelt_result(ItemId::of_block(BlockKind::Ore)),
        Some(ItemId(IRON_INGOT))
    );
    assert_eq!(smelt_result(ItemId(PLANK_ITEM)), None);
}
