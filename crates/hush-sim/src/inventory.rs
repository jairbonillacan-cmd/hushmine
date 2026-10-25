//! Inventario de ranuras y catálogo de objetos del modo Historia.
//!
//! Cada ranura guarda una pila limitada por `max_stack`; las herramientas
//! (objetos con durabilidad) nunca se apilan. [`Inventory::try_add`] y
//! [`Inventory::try_remove`] son atómicas: si algo no cabe, todo queda como estaba.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Identificador estable de objeto del modo Historia; el `0` está reservado.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u16);

/// Ficha de latón: la moneda de la partida, se devuelve al altar.
pub const BRASS_TALLY: ItemId = ItemId(1); // Se moverá a data/*.ron
/// Lámpara de cobre alimentada con carbón.
pub const LAMP: ItemId = ItemId(2); // Se moverá a data/*.ron
/// Madera de entibar galerías.
pub const TIMBER: ItemId = ItemId(3); // Se moverá a data/*.ron
/// Mena sin fundir.
pub const ORE: ItemId = ItemId(4); // Se moverá a data/*.ron
/// Carbón combustible.
pub const COAL: ItemId = ItemId(5); // Se moverá a data/*.ron
/// Cobre fundido.
pub const COPPER: ItemId = ItemId(6); // Se moverá a data/*.ron
/// Hierro fundido.
pub const IRON: ItemId = ItemId(7); // Se moverá a data/*.ron
/// Oro para las ofrendas del altar.
pub const GOLD: ItemId = ItemId(8); // Se moverá a data/*.ron
/// Diamante.
pub const DIAMOND: ItemId = ItemId(9); // Se moverá a data/*.ron
/// Pico de cobre.
pub const COPPER_PICK: ItemId = ItemId(10); // Se moverá a data/*.ron
/// Pico de hierro.
pub const IRON_PICK: ItemId = ItemId(11); // Se moverá a data/*.ron
/// Llave que habilita una mesa de trabajo.
pub const WORKBENCH_KEY: ItemId = ItemId(12); // Se moverá a data/*.ron
/// Carga silenciosa de los Quedos.
pub const QUIET_CHARGE: ItemId = ItemId(13); // Se moverá a data/*.ron

/// Propiedades estáticas de un objeto del catálogo integrado.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemSpec {
    pub id: ItemId,
    pub max_stack: u16,
    pub max_durability: Option<u16>,
    pub unit_weight: u16,
}

/// Constructor compacto de especificaciones para la tabla integrada.
// Se moverá a data/*.ron
const fn spec(
    id: ItemId,
    max_stack: u16,
    max_durability: Option<u16>,
    unit_weight: u16,
) -> ItemSpec {
    ItemSpec {
        id,
        max_stack,
        max_durability,
        unit_weight,
    }
}

/// Tabla integrada de objetos (`ids 1..=13`); el `0` queda reservado.
// Se moverá a data/*.ron
static BUILTIN_ITEMS: [ItemSpec; 13] = [
    spec(BRASS_TALLY, 64, None, 2),
    spec(LAMP, 4, None, 30),
    spec(TIMBER, 64, None, 10),
    spec(ORE, 64, None, 25),
    spec(COAL, 64, None, 15),
    spec(COPPER, 64, None, 20),
    spec(IRON, 64, None, 25),
    spec(GOLD, 64, None, 30),
    spec(DIAMOND, 16, None, 10),
    spec(COPPER_PICK, 1, Some(120), 80),
    spec(IRON_PICK, 1, Some(300), 100),
    spec(WORKBENCH_KEY, 1, None, 40),
    spec(QUIET_CHARGE, 16, None, 50),
];

/// Catálogo integrado de objetos, en orden de identificador.
pub fn builtin_items() -> &'static [ItemSpec] {
    &BUILTIN_ITEMS
}

/// Busca la especificación de un objeto por su identificador.
pub fn item_spec(id: ItemId) -> Option<&'static ItemSpec> {
    BUILTIN_ITEMS.iter().find(|spec| spec.id == id)
}

/// Pila de unidades idénticas de un objeto en una ranura del inventario.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: ItemId,
    pub count: u16,
    pub durability: Option<u16>,
}

impl ItemStack {
    /// Crea una pila con la durabilidad máxima del catálogo (`None` si el
    /// objeto no es herramienta o no figura en el catálogo).
    pub fn new(item: ItemId, count: u16) -> Self {
        Self {
            item,
            count,
            durability: item_spec(item).and_then(|spec| spec.max_durability),
        }
    }
}

/// Error al modificar un inventario; las operaciones atómicas nunca lo dejan a medias.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InventoryError {
    /// No queda hueco, ni apilando, para toda la pila.
    Full,
    /// No quedan unidades suficientes de ese objeto.
    NotEnoughItem(ItemId),
    /// La ranura no existe o está vacía.
    InvalidSlot,
    /// El objeto de la ranura no sufre desgaste.
    NotATool,
    /// El identificador no figura en el catálogo integrado.
    UnknownItem(ItemId),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "el inventario está lleno"),
            Self::NotEnoughItem(item) => write!(f, "no quedan unidades de {:?}", item),
            Self::InvalidSlot => write!(f, "la ranura no existe o está vacía"),
            Self::NotATool => write!(f, "el objeto no sufre desgaste"),
            Self::UnknownItem(item) => write!(f, "objeto desconocido: {:?}", item),
        }
    }
}

impl std::error::Error for InventoryError {}

/// Estado de una herramienta tras aplicarle desgaste.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolState {
    /// La herramienta sigue operativa.
    Intact,
    /// La herramienta alcanzó durabilidad `0` y se consumió.
    Broken,
}

/// Inventario de capacidad fija: una pila por ranura o `None` si está vacía.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    slots: Vec<Option<ItemStack>>,
}

impl Inventory {
    /// Crea un inventario con `capacity` ranuras, todas vacías.
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: vec![None; capacity],
        }
    }

    /// Número total de ranuras del inventario.
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Contenido de una ranura; `None` si está vacía o el índice se sale del inventario.
    pub fn slot(&self, index: usize) -> Option<ItemStack> {
        self.slots.get(index).copied().flatten()
    }

    /// Peso total: `count * unit_weight` por pila, con aritmética saturante. Un
    /// objeto fuera de catálogo (p. ej. deserializado) pesa `0`, sin pánico.
    pub fn total_weight(&self) -> u32 {
        self.slots
            .iter()
            .flatten()
            .map(|stack| {
                let unit = item_spec(stack.item).map_or(0, |spec| u32::from(spec.unit_weight));
                u32::from(stack.count).saturating_mul(unit)
            })
            .fold(0u32, u32::saturating_add)
    }

    /// Unidades totales de un objeto sumando todas las ranuras.
    pub fn count_item(&self, item: ItemId) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|stack| stack.item == item)
            .map(|stack| u32::from(stack.count))
            .fold(0u32, u32::saturating_add)
    }

    /// Añade una pila entera o nada: operación atómica. Apila primero en
    /// ranuras del mismo objeto con hueco hasta `max_stack` y después
    /// rellena ranuras vacías; las herramientas nunca se apilan. Si no cabe
    /// todo, `Full` sin cambios; un objeto fuera de catálogo siempre da
    /// `UnknownItem` (aunque `count == 0`, que en caso válido no hace nada).
    pub fn try_add(&mut self, stack: ItemStack) -> Result<(), InventoryError> {
        let spec = item_spec(stack.item).ok_or(InventoryError::UnknownItem(stack.item))?;
        if stack.count == 0 {
            return Ok(());
        }

        // Ensayo sobre una copia: solo se compromete si cabe todo.
        let mut staged = self.clone();
        let mut remaining = stack.count;
        let is_tool = spec.max_durability.is_some();

        // 1. Apilar en ranuras del mismo objeto con espacio (no herramientas).
        if !is_tool {
            for slot in staged.slots.iter_mut() {
                if remaining == 0 {
                    break;
                }
                if let Some(existing) = slot
                    && existing.item == stack.item
                    && existing.count < spec.max_stack
                {
                    let moved = (spec.max_stack - existing.count).min(remaining);
                    existing.count += moved;
                    remaining -= moved;
                }
            }
        }

        // 2. Rellenar ranuras vacías; las herramientas van de una en una.
        for slot in staged.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if slot.is_none() {
                let moved = (if is_tool { 1 } else { spec.max_stack }).min(remaining);
                *slot = Some(ItemStack {
                    item: stack.item,
                    count: moved,
                    durability: if is_tool { stack.durability } else { None },
                });
                remaining -= moved;
            }
        }

        if remaining > 0 {
            return Err(InventoryError::Full);
        }
        *self = staged;
        Ok(())
    }

    /// Retira `count` unidades o nada, vaciando primero las ranuras de índice
    /// más bajo (orden determinista); sin suficiente, `NotEnoughItem`.
    pub fn try_remove(&mut self, item: ItemId, count: u16) -> Result<(), InventoryError> {
        if self.count_item(item) < u32::from(count) {
            return Err(InventoryError::NotEnoughItem(item));
        }

        // Ensayo sobre una copia para mantener la operación atómica.
        let mut staged = self.clone();
        let mut remaining = count;
        for slot in staged.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(mut victim) = *slot
                && victim.item == item
            {
                let taken = victim.count.min(remaining);
                remaining -= taken;
                victim.count -= taken;
                *slot = (victim.count > 0).then_some(victim);
            }
        }
        *self = staged;
        Ok(())
    }

    /// Desgasta la herramienta de una ranura con resta saturante; al llegar a
    /// `0` la pieza se consume (ranura vacía) y devuelve [`ToolState::Broken`].
    pub fn damage_slot(&mut self, slot: usize, wear: u16) -> Result<ToolState, InventoryError> {
        let Some(stored) = self.slots.get_mut(slot) else {
            return Err(InventoryError::InvalidSlot);
        };
        let Some(stack) = stored.as_mut() else {
            return Err(InventoryError::InvalidSlot);
        };
        let Some(durability) = stack.durability else {
            return Err(InventoryError::NotATool);
        };
        let remaining = durability.saturating_sub(wear);
        if remaining == 0 {
            *stored = None;
            return Ok(ToolState::Broken);
        }
        stack.durability = Some(remaining);
        Ok(ToolState::Intact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_catalog_is_well_formed() {
        let items = builtin_items();
        assert_eq!(items.len(), 13);
        for (index, entry) in items.iter().enumerate() {
            assert_eq!(entry.id, ItemId(u16::try_from(index + 1).unwrap()));
            assert_eq!(item_spec(entry.id), Some(entry));
            assert!(entry.unit_weight > 0);
        }
        assert_eq!(item_spec(ItemId(0)), None); // identificador reservado
    }

    #[test]
    fn add_stacks_to_max_and_is_atomic() {
        let mut inv = Inventory::new(4);
        inv.try_add(ItemStack::new(ORE, 40)).unwrap();
        inv.try_add(ItemStack::new(ORE, 30)).unwrap(); // 24 apilan, 6 desbordan
        inv.try_add(ItemStack::new(GOLD, 2)).unwrap();
        assert_eq!(inv.capacity(), 4);
        assert_eq!(inv.slot(0).map(|s| s.count), Some(64)); // tope del catálogo
        assert_eq!(inv.slot(1).map(|s| s.count), Some(6));
        assert_eq!(inv.slot(2).map(|s| s.count), Some(2));
        assert_eq!(inv.slot(4), None); // fuera de rango, sin pánico
        assert_eq!(inv.count_item(ORE), 70);
        assert_eq!(inv.total_weight(), 70 * 25 + 2 * 30);

        // La pila que no cabe entera no entra nada (capacidad 1).
        let mut tiny = Inventory::new(1);
        tiny.try_add(ItemStack::new(TIMBER, 30)).unwrap();
        let before = tiny.clone();
        let full = tiny.try_add(ItemStack::new(TIMBER, 64));
        assert_eq!(full, Err(InventoryError::Full));
        assert_eq!(tiny, before); // sin cambios parciales
        let unknown = tiny.try_add(ItemStack::new(ItemId(99), 1));
        assert_eq!(unknown, Err(InventoryError::UnknownItem(ItemId(99))));
        assert_eq!(tiny.try_add(ItemStack::new(ORE, 0)), Ok(())); // pila vacía
    }

    #[test]
    fn try_remove_empties_lowest_slots_first() {
        let mut inv = Inventory::new(4);
        inv.try_add(ItemStack::new(TIMBER, 64)).unwrap(); // llena la ranura 0
        inv.try_add(ItemStack::new(TIMBER, 10)).unwrap(); // va a la ranura 1
        inv.try_remove(TIMBER, 70).unwrap();
        assert_eq!(inv.slot(0), None);
        assert_eq!(inv.slot(1).map(|s| s.count), Some(4));
        let err = inv.try_remove(TIMBER, 5).unwrap_err();
        assert_eq!(err, InventoryError::NotEnoughItem(TIMBER));
        assert!(!err.to_string().is_empty());
        assert_eq!(inv.count_item(TIMBER), 4); // sin cambios
    }

    #[test]
    fn tools_never_stack() {
        let mut inv = Inventory::new(2);
        assert_eq!(ItemStack::new(COPPER_PICK, 1).durability, Some(120));
        assert_eq!(ItemStack::new(IRON_PICK, 1).durability, Some(300));
        inv.try_add(ItemStack::new(COPPER_PICK, 1)).unwrap();
        inv.try_add(ItemStack::new(COPPER_PICK, 1)).unwrap();
        assert_eq!(inv.slot(0).map(|s| s.count), Some(1));
        assert_eq!(inv.slot(1).map(|s| s.count), Some(1));
        let third = inv.try_add(ItemStack::new(COPPER_PICK, 1));
        assert_eq!(third, Err(InventoryError::Full));
    }

    #[test]
    fn damage_slot_wears_and_consumes_the_tool() {
        let mut inv = Inventory::new(2);
        inv.try_add(ItemStack::new(ORE, 5)).unwrap();
        inv.try_add(ItemStack::new(COPPER_PICK, 1)).unwrap();
        assert_eq!(inv.damage_slot(9, 10), Err(InventoryError::InvalidSlot));
        assert_eq!(inv.damage_slot(0, 10), Err(InventoryError::NotATool));
        assert_eq!(inv.damage_slot(1, 100), Ok(ToolState::Intact));
        assert_eq!(inv.slot(1).map(|s| s.durability), Some(Some(20)));
        assert_eq!(inv.damage_slot(1, 20), Ok(ToolState::Broken));
        assert_eq!(inv.slot(1), None); // la pieza se consumió
        assert_eq!(inv.count_item(COPPER_PICK), 0);
    }
}
