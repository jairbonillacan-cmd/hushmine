//! Inventario del modo historia: barra rápida, mochila, armadura y mano
//! secundaria, más los contenedores del mundo (arcones y hornos).

use serde::{Deserialize, Serialize};

use super::items::ItemId;

/// Una casilla del inventario: apilamiento con desgaste y metadatos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub id: ItemId,
    pub count: u16,
    /// Usos restantes para herramientas y blindaje.
    pub durability: u16,
    /// Bandera de bits para estados especiales.
    pub metadata: u16,
}

impl ItemStack {
    pub fn new(id: ItemId, count: u16) -> Self {
        Self {
            id,
            count,
            durability: id.max_durability(),
            metadata: 0,
        }
    }

    /// Pila de una unidad con la durabilidad indicada (para herramientas).
    pub fn tool(id: ItemId, durability: u16) -> Self {
        Self {
            id,
            count: 1,
            durability,
            metadata: 0,
        }
    }

    /// Resta un uso a la herramienta; `true` si se ha roto.
    pub fn wear(&mut self) -> bool {
        if self.durability == 0 {
            return true;
        }
        self.durability -= 1;
        self.durability == 0
    }

    fn can_merge_with(self, other: &Self) -> bool {
        self.id == other.id && self.metadata == other.metadata && self.id.stack_max() > 1
    }
}

/// Casilla ocupada o vacía.
pub type Slot = Option<ItemStack>;

/// Almacenamiento personal del minero.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// Barra de acceso rápido, 9 casillas (índices 0..=8).
    pub hotbar: Vec<Slot>,
    /// Mochila de expedición, 27 casillas en 9×3.
    pub backpack: Vec<Slot>,
    /// Casco, peto, pantalón y botas.
    pub armor: Vec<Slot>,
    /// Mano secundaria (antorcha, escudo, brújula, dinamita...).
    pub offhand: Slot,
    /// Casilla de la barra rápida seleccionada.
    pub selected: u8,
}

/// Índice de una casilla del inventario personal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Hotbar(u8),
    Backpack(u8),
    Armor(u8),
    Offhand,
}

/// Sección de almacenamiento (para tomar dos préstamos disjuntos).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Hotbar,
    Backpack,
    Armor,
    Offhand,
}

impl Place {
    fn parts(self) -> (Section, u8) {
        match self {
            Self::Hotbar(i) => (Section::Hotbar, i),
            Self::Backpack(i) => (Section::Backpack, i),
            Self::Armor(i) => (Section::Armor, i),
            Self::Offhand => (Section::Offhand, 0),
        }
    }
}

impl Place {
    pub const ALL_HOTBAR: [Self; 9] = [
        Self::Hotbar(0),
        Self::Hotbar(1),
        Self::Hotbar(2),
        Self::Hotbar(3),
        Self::Hotbar(4),
        Self::Hotbar(5),
        Self::Hotbar(6),
        Self::Hotbar(7),
        Self::Hotbar(8),
    ];
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            hotbar: vec![None; 9],
            backpack: vec![None; 27],
            armor: vec![None; 4],
            offhand: None,
            selected: 0,
        }
    }

    pub fn get(&self, place: Place) -> &Slot {
        match place {
            Place::Hotbar(i) => &self.hotbar[usize::from(i)],
            Place::Backpack(i) => &self.backpack[usize::from(i)],
            Place::Armor(i) => &self.armor[usize::from(i)],
            Place::Offhand => &self.offhand,
        }
    }

    pub fn get_mut(&mut self, place: Place) -> &mut Slot {
        match place {
            Place::Hotbar(i) => &mut self.hotbar[usize::from(i)],
            Place::Backpack(i) => &mut self.backpack[usize::from(i)],
            Place::Armor(i) => &mut self.armor[usize::from(i)],
            Place::Offhand => &mut self.offhand,
        }
    }

    /// Ítem empuñado en la mano principal.
    pub fn held(&self) -> Option<ItemStack> {
        self.hotbar[usize::from(self.selected)]
    }

    fn section_mut(&mut self, section: Section) -> &mut Vec<Slot> {
        match section {
            Section::Hotbar => &mut self.hotbar,
            Section::Backpack => &mut self.backpack,
            Section::Armor => &mut self.armor,
            Section::Offhand => {
                unreachable!("la mano secundaria no es una lista; se maneja aparte")
            }
        }
    }

    /// Intercambia el contenido de dos casillas cualesquiera.
    pub fn swap(&mut self, a: Place, b: Place) {
        if a == b {
            return;
        }
        let (sa, ia) = a.parts();
        let (sb, ib) = b.parts();
        if !matches!(sa, Section::Offhand) && !matches!(sb, Section::Offhand) {
            if sa == sb {
                let vec = self.section_mut(sa);
                vec.swap(usize::from(ia), usize::from(ib));
            } else {
                // Préstamos disjuntos por sección: desempaquetar en dos
                // referencias independientes del compilador.
                match (sa, sb) {
                    (Section::Hotbar, Section::Backpack) | (Section::Backpack, Section::Hotbar) => {
                        swap_slots(
                            &mut self.hotbar[usize::from(ia)],
                            &mut self.backpack[usize::from(ib)],
                        );
                    }
                    (Section::Hotbar, Section::Armor) | (Section::Armor, Section::Hotbar) => {
                        swap_slots(
                            &mut self.hotbar[usize::from(ia)],
                            &mut self.armor[usize::from(ib)],
                        );
                    }
                    (Section::Backpack, Section::Armor) | (Section::Armor, Section::Backpack) => {
                        swap_slots(
                            &mut self.backpack[usize::from(ia)],
                            &mut self.armor[usize::from(ib)],
                        );
                    }
                    _ => unreachable!("secciones cubiertas arriba"),
                }
            }
            return;
        }
        let mut tmp = self.offhand;
        let other = if matches!(sa, Section::Offhand) { b } else { a };
        std::mem::swap(self.get_mut(other), &mut tmp);
        self.offhand = tmp;
    }

    /// Añade una pila al inventario (primero apila, luego huecos libres en
    /// barra rápida y mochila). Devuelve lo que no cupo.
    pub fn add(&mut self, stack: ItemStack) -> Option<ItemStack> {
        let mut stack = stack;
        let places = Place::ALL_HOTBAR
            .iter()
            .copied()
            .chain((0..27).map(Place::Backpack))
            .collect::<Vec<_>>();
        // Primero apilar sobre pilas existentes.
        for place in places.iter().copied() {
            let capacity = stack.id.stack_max();
            let slot = self.get_mut(place);
            if let Some(existing) = slot
                && existing.can_merge_with(&stack)
            {
                let room = capacity.saturating_sub(existing.count);
                let move_n = room.min(stack.count);
                existing.count += move_n;
                stack.count -= move_n;
                if stack.count == 0 {
                    return None;
                }
            }
        }
        // Luego, casillas vacías (respetando el máximo de apilamiento).
        for place in places {
            if stack.count == 0 {
                return None;
            }
            let slot = self.get_mut(place);
            if slot.is_none() {
                let capacity = stack.id.stack_max();
                let placed = capacity.min(stack.count);
                *slot = Some(ItemStack {
                    count: placed,
                    ..stack
                });
                stack.count -= placed;
            }
        }
        (stack.count > 0).then_some(stack)
    }

    /// Cuenta cuántas unidades hay de un ítem.
    pub fn count_of(&self, id: ItemId) -> u32 {
        let mut total = 0;
        for slot in self.hotbar.iter().chain(&self.backpack) {
            if let Some(s) = slot
                && s.id == id
            {
                total += u32::from(s.count);
            }
        }
        total
    }

    /// Consume `n` unidades de un ítem de donde estén; `false` si no hay.
    pub fn consume(&mut self, id: ItemId, n: u16) -> bool {
        if self.count_of(id) < u32::from(n) {
            return false;
        }
        let mut left = n;
        for place in Place::ALL_HOTBAR
            .iter()
            .copied()
            .chain((0..27).map(Place::Backpack))
        {
            let slot = self.get_mut(place);
            if let Some(s) = slot
                && s.id == id
            {
                let take = s.count.min(left);
                s.count -= take;
                left -= take;
                if s.count == 0 {
                    *slot = None;
                }
                if left == 0 {
                    return true;
                }
            }
        }
        false
    }

    /// Resta un uso a la herramienta empuñada y la rompe al llegar a cero.
    /// Devuelve el ítem roto, si procede.
    pub fn wear_held(&mut self) -> Option<ItemStack> {
        let selected = self.selected;
        let slot = &mut self.hotbar[usize::from(selected)];
        if let Some(stack) = slot {
            let id = stack.id;
            let broken = stack.wear();
            if broken && stack.count == 1 && id.stack_max() == 1 {
                let removed = *stack;
                *slot = None;
                return Some(removed);
            }
        }
        None
    }
}

fn swap_slots(a: &mut Slot, b: &mut Slot) {
    std::mem::swap(a, b);
}

/// Contenedor físico del mundo: arcón de 27 casillas (un arcón doble se
/// muestra como dos contenedores de 27, sin fusionar el almacenaje).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Container {
    pub slots: Vec<Slot>,
}

impl Container {
    pub fn chest() -> Self {
        Self {
            slots: vec![None; 27],
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Introduce una pila (apilando primero). Devuelve lo que no cupo.
    pub fn push(&mut self, stack: ItemStack) -> Option<ItemStack> {
        let mut stack = stack;
        for slot in self.slots.iter_mut() {
            if let Some(existing) = slot
                && existing.can_merge_with(&stack)
            {
                let room = stack.id.stack_max().saturating_sub(existing.count);
                let move_n = room.min(stack.count);
                existing.count += move_n;
                stack.count -= move_n;
                if stack.count == 0 {
                    return None;
                }
            }
        }
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(stack);
                return None;
            }
        }
        Some(stack)
    }

    /// Cuenta cuántas unidades hay de un ítem.
    pub fn count_of(&self, id: ItemId) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|s| s.id == id)
            .map(|s| u32::from(s.count))
            .sum()
    }
}

/// Transferencia de tolva: un ítem de `from` a `to`. La tolva del juego
/// mueve un ítem cada 8 ticks de simulación.
pub const HOPPER_INTERVAL_TICKS: u32 = 8;

/// Mueve una unidad del primer ítem de `from` a `to`. Devuelve `true` si
/// movió algo.
pub fn hopper_transfer(from: &mut Container, to: &mut Container) -> bool {
    let Some(index) = from.slots.iter().position(|s| s.is_some()) else {
        return false;
    };
    let Some(mut stack) = from.slots[index] else {
        return false;
    };
    stack.count = 1;
    if to.push(stack).is_some() {
        return false; // El receptor está lleno.
    }
    let Some(orig) = from.slots[index].as_mut() else {
        return false;
    };
    orig.count -= 1;
    if orig.count == 0 {
        from.slots[index] = None;
    }
    true
}

#[cfg(test)]
mod tests {
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
}
