//! Pruebas de `inventory`.

use super::super::items::{COAL, HANDLE, PLANK_ITEM};
use super::*;

fn stack(id: u16, count: u16) -> ItemStack {
    ItemStack::new(ItemId(id), count)
}

#[test]
fn add_stacks_and_fills_hotbar_then_backpack() {
    let mut inv = Inventory::new();
    assert_eq!(inv.add(stack(PLANK_ITEM, 64)), None);
    assert_eq!(inv.add(stack(PLANK_ITEM, 64)), None);
    assert_eq!(inv.add(stack(PLANK_ITEM, 1)), None);
    // Tres casillas de la barra rápida: dos pilas llenas y una de 1.
    assert_eq!(inv.count_of(ItemId(PLANK_ITEM)), 129);
    let places_with = Place::ALL_HOTBAR
        .iter()
        .filter(|p| inv.get(**p).is_some())
        .count();
    assert_eq!(places_with, 3);
    // Nada quedó sin sitio.
    assert_eq!(inv.add(stack(PLANK_ITEM, 0)), None);
}

#[test]
fn add_overflow_returns_leftover_when_full() {
    let mut inv = Inventory::new();
    // Todo lleno de mangos salvo una pila de carbón a medias y una
    // casilla vacía.
    for slot in inv.hotbar.iter_mut().chain(&mut inv.backpack) {
        *slot = Some(stack(HANDLE, 17));
    }
    inv.hotbar[0] = Some(stack(COAL, 60));
    inv.hotbar[1] = None;
    // Caben 4 en la pila de carbón y 64 en la vacía: sobran 32 de 100.
    let leftover = inv.add(stack(COAL, 100)).unwrap();
    assert_eq!(leftover.count, 32);
    // 60 iniciales más 68 almacenadas.
    assert_eq!(inv.count_of(ItemId(COAL)), 128);
}

#[test]
fn consume_across_slots_and_wear_tools() {
    let mut inv = Inventory::new();
    inv.hotbar[0] = Some(stack(COAL, 30));
    inv.backpack[5] = Some(stack(COAL, 30));
    assert!(inv.consume(ItemId(COAL), 50));
    assert_eq!(inv.count_of(ItemId(COAL)), 10);
    assert!(!inv.consume(ItemId(COAL), 11));

    let pick = super::super::items::tool_item(0, super::super::items::ToolKind::Pickaxe);
    inv.hotbar[1] = Some(ItemStack::tool(pick, 2));
    inv.selected = 1;
    assert!(inv.wear_held().is_none());
    assert!(inv.wear_held().is_some());
    assert!(inv.hotbar[1].is_none());
}

#[test]
fn swap_moves_stacks_between_any_places() {
    let mut inv = Inventory::new();
    inv.hotbar[0] = Some(stack(PLANK_ITEM, 7));
    inv.swap(Place::Hotbar(0), Place::Backpack(9));
    assert!(inv.hotbar[0].is_none());
    assert_eq!(inv.backpack[9].map(|s| s.count), Some(7));
    inv.armor[2] = Some(stack(HANDLE, 2));
    inv.swap(Place::Armor(2), Place::Offhand);
    assert!(inv.armor[2].is_none());
    assert_eq!(inv.offhand.map(|s| s.count), Some(2));
}

#[test]
fn chest_push_and_hopper_transfer() {
    let mut chest = Container::chest();
    assert_eq!(chest.len(), 27);
    assert_eq!(chest.push(stack(COAL, 64)), None);
    assert_eq!(chest.count_of(ItemId(COAL)), 64);

    let mut hopper_into = Container::chest();
    for _ in 0..3 {
        assert!(hopper_transfer(&mut chest, &mut hopper_into));
    }
    assert_eq!(chest.count_of(ItemId(COAL)), 61);
    assert_eq!(hopper_into.count_of(ItemId(COAL)), 3);
}
