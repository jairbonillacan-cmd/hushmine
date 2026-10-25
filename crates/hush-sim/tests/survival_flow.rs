//! Pruebas de integración de los sistemas de supervivencia del modo Historia:
//! inventario y herramientas, fabricación en estaciones con su ruido y
//! propagación del sonido por el mundo de vóxeles.

use grisu_voxel::{AIR, VoxelWorld};
use hush_sim::BlockKind;
use hush_sim::crafting::{CraftError, Recipe, StationKind, builtin_recipes, craft};
use hush_sim::inventory::{
    COAL, COPPER, COPPER_PICK, Inventory, InventoryError, ItemStack, ORE, TIMBER, ToolState,
    item_spec,
};
use hush_sim::sound_grid::{MAX_INTENSITY, SoundEvent, propagate_sound};

/// Mundo 32³ macizo del bloque indicado.
fn solid_world(block: BlockKind) -> VoxelWorld {
    let mut world = VoxelWorld::new([32, 32, 32]).unwrap();
    world.fill([0, 0, 0], [31, 31, 31], block.id());
    world
}

/// Mundo de roca con un túnel de aire en `[0..=31, 16, 16]`.
fn tunneled_world() -> VoxelWorld {
    let mut world = solid_world(BlockKind::Rock);
    world.fill([0, 16, 16], [31, 16, 16], AIR);
    world
}

/// Receta del catálogo integrado por identificador.
fn recipe(id: u16) -> Recipe {
    *builtin_recipes().iter().find(|r| r.id == id).unwrap()
}

/// Evento sonoro de intensidad máxima emitido en `[0, 16, 16]`.
fn loud_event() -> SoundEvent {
    SoundEvent {
        origin: [0, 16, 16],
        intensity: MAX_INTENSITY,
    }
}

#[test]
fn try_add_and_craft_fail_atomically_without_losing_items() {
    // Inventario de una ranura con 30 tablones: no queda sitio para 64 más.
    let mut inv = Inventory::new(1);
    inv.try_add(ItemStack::new(TIMBER, 30)).unwrap();
    let before = inv.clone();

    assert_eq!(
        inv.try_add(ItemStack::new(TIMBER, 64)),
        Err(InventoryError::Full)
    );
    assert_eq!(inv, before); // sin cambios parciales: no se pierde nada
    assert_eq!(inv.count_item(TIMBER), 30);

    // Trabajar escombro a mano (2 mena -> 1 tablón) con la única ranura
    // ocupada: la mena sobrevive y el tablón no cabe, todo queda igual.
    let mut inv = Inventory::new(1);
    inv.try_add(ItemStack::new(ORE, 4)).unwrap();
    let before = inv.clone();

    assert_eq!(
        craft(&mut inv, &recipe(1), StationKind::Hand),
        Err(CraftError::OutputDoesNotFit)
    );
    assert_eq!(inv, before);
    assert_eq!(inv.count_item(ORE), 4);
    assert_eq!(inv.count_item(TIMBER), 0);
}

#[test]
fn crafted_pick_uses_catalog_durability_and_breaks() {
    let mut inv = Inventory::new(4);
    inv.try_add(ItemStack::new(COPPER, 2)).unwrap();
    inv.try_add(ItemStack::new(TIMBER, 1)).unwrap();

    // Pico de cobre (2 cobre + 1 tablón) en la mesa de trabajo.
    craft(&mut inv, &recipe(3), StationKind::Workbench).unwrap();

    let expected = item_spec(COPPER_PICK).unwrap().max_durability;
    assert_eq!(expected, Some(120)); // cifra del catálogo integrado
    assert_eq!(inv.slot(0).map(|s| s.durability), Some(expected));
    assert_eq!(inv.count_item(COPPER_PICK), 1);

    // Desgaste parcial: la herramienta sigue operativa.
    assert_eq!(inv.damage_slot(0, 100), Ok(ToolState::Intact));
    assert_eq!(inv.slot(0).map(|s| s.durability), Some(Some(20)));

    // Durabilidad agotada: la pieza se consume y la ranura queda vacía.
    assert_eq!(inv.damage_slot(0, 20), Ok(ToolState::Broken));
    assert_eq!(inv.slot(0), None);
    assert_eq!(inv.count_item(COPPER_PICK), 0);
}

#[test]
fn tunnel_carries_sound_farther_than_solid_rock() {
    let event = loud_event();

    // Por el túnel de aire (coste 1 por celda) la celda a 25 del origen oye.
    let tunnel = propagate_sound(&tunneled_world(), event);
    assert_eq!(tunnel[&[25, 16, 16]].intensity, 39);
    assert_eq!(tunnel[&[25, 16, 16]].first_step_dir, [-1, 0, 0]);

    // A través de roca maciza (coste 3 por celda) la celda 21 es la última
    // oída: 63 de coste exacto consume casi toda la intensidad.
    let rock = propagate_sound(&solid_world(BlockKind::Rock), event);
    assert_eq!(rock[&[21, 16, 16]].intensity, 1);
    assert!(!rock.contains_key(&[22, 16, 16]));
    assert!(!rock.contains_key(&[25, 16, 16]));
}

#[test]
fn dirt_muffles_sound_more_than_rock() {
    let event = loud_event();

    // A 15 celdas la tierra (coste 5) ya no se oye: 75 > 64 de intensidad.
    let dirt = propagate_sound(&solid_world(BlockKind::Dirt), event);
    assert!(!dirt.contains_key(&[15, 16, 16]));

    // La roca (coste 3) a la misma distancia conserva intensidad residual.
    let rock = propagate_sound(&solid_world(BlockKind::Rock), event);
    assert_eq!(rock[&[15, 16, 16]].intensity, 19);
}

#[test]
fn propagate_sound_is_deterministic_on_the_same_world() {
    let world = tunneled_world();
    let event = SoundEvent {
        origin: [16, 16, 16],
        intensity: 24,
    };

    let first = propagate_sound(&world, event);
    let second = propagate_sound(&world, event);
    assert!(!first.is_empty()); // la igualdad no debe ser trivial
    assert_eq!(first, second);
}

#[test]
fn mining_flow_from_ore_to_broken_pick() {
    let mut inv = Inventory::new(8);
    inv.try_add(ItemStack::new(ORE, 8)).unwrap();
    inv.try_add(ItemStack::new(COAL, 2)).unwrap();

    let hand_timber = recipe(1); // 2 mena -> 1 tablón, a mano.
    let smelt_copper = recipe(6); // 2 mena + 1 carbón -> 1 cobre, en fragua.
    let copper_pick = recipe(3); // 2 cobre + 1 tablón -> pico, en mesa.

    // Recolectar mena ya hecho; el mango se labra a mano.
    let mut total_noise = craft(&mut inv, &hand_timber, StationKind::Hand).unwrap();
    assert_eq!(inv.count_item(TIMBER), 1);

    // Dos fundiciones en la fragua: la operación más ruidosa del flujo.
    for _ in 0..2 {
        total_noise += craft(&mut inv, &smelt_copper, StationKind::Forge).unwrap();
    }
    assert_eq!(inv.count_item(COPPER), 2);
    assert_eq!(inv.count_item(ORE), 2); // 8 - 2 del tablón - 4 de las fundiciones
    assert_eq!(inv.count_item(COAL), 0);

    // El pico se fabrica en la mesa y arranca con la durabilidad del catálogo.
    total_noise += craft(&mut inv, &copper_pick, StationKind::Workbench).unwrap();
    assert_eq!(
        total_noise,
        hand_timber.noise_units + 2 * smelt_copper.noise_units + copper_pick.noise_units
    );

    let pick_slot = (0..inv.capacity())
        .find(|&i| inv.slot(i).is_some_and(|s| s.item == COPPER_PICK))
        .unwrap();
    assert_eq!(inv.slot(pick_slot).unwrap().durability, Some(120));

    // Cavar desgasta el pico hasta romperlo: 120 puntos en golpes de 3.
    let mut swings = 0u32;
    let mut state = ToolState::Intact;
    while state == ToolState::Intact {
        state = inv.damage_slot(pick_slot, 3).unwrap();
        swings += 1;
    }
    assert_eq!(swings, 40);
    assert_eq!(inv.slot(pick_slot), None);
    assert_eq!(inv.count_item(COPPER_PICK), 0);
}
