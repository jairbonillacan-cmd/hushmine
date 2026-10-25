//! Fabricación de objetos en estaciones y su coste de ruido.
//!
//! Cada receta declara la estación que exige, sus ingredientes, el producto,
//! los ticks de elaboración y el ruido que añade al Sismógrafo en milésimas
//! de punto porcentual (`0..=100_000`, las mismas unidades que
//! [`crate::seismograph`]). [`craft`] es atómico: si la estación no coincide,
//! falta un ingrediente o el producto no cabe, el inventario queda intacto.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::inventory::{
    BRASS_TALLY, COAL, COPPER, COPPER_PICK, IRON, IRON_PICK, Inventory, ItemId, ItemStack, LAMP,
    ORE, QUIET_CHARGE, TIMBER, WORKBENCH_KEY,
};

/// Estación de fabricación donde puede elaborarse una receta.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StationKind {
    /// Trabajo a mano, sin mesa: el minero labra el escombro donde esté.
    Hand,
    /// Mesa de trabajo: lámparas, picos y llaves.
    Workbench,
    /// Forja: fundir mena en metal; la operación más ruidosa.
    Forge,
    /// Banco de los Quedos: fabrica en silencio absoluto.
    QuietBench,
}

/// Receta de fabricación del catálogo integrado.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Recipe {
    pub id: u16,
    pub station: StationKind,
    pub inputs: &'static [(ItemId, u16)],
    pub output: (ItemId, u16),
    pub craft_ticks: u16,
    /// Ruido en milésimas de punto porcentual, las mismas unidades que el Sismógrafo (0..=100_000).
    pub noise_units: u32,
}

/// Error al elaborar una receta; el inventario nunca queda a medias.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CraftError {
    /// La receta exige una estación distinta de la usada.
    WrongStation { required: StationKind },
    /// No quedan suficientes unidades del ingrediente `item`.
    MissingIngredient { item: ItemId },
    /// El inventario no tiene hueco para el producto.
    OutputDoesNotFit,
}

impl fmt::Display for CraftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongStation { required } => write!(
                f,
                "esta receta exige la estación {}",
                station_label(*required)
            ),
            Self::MissingIngredient { item } => write!(f, "falta el ingrediente {:?}", item),
            Self::OutputDoesNotFit => write!(f, "el producto no cabe en el inventario"),
        }
    }
}

impl std::error::Error for CraftError {}

/// Etiqueta en español de cada estación para los mensajes de error.
// Se moverá a data/*.ron junto con el catálogo de recetas
fn station_label(station: StationKind) -> &'static str {
    match station {
        StationKind::Hand => "mano",
        StationKind::Workbench => "mesa de trabajo",
        StationKind::Forge => "forja",
        StationKind::QuietBench => "banco de los Quedos",
    }
}

/// Catálogo integrado de recetas (`ids 1..=8`).
///
/// Las cifras son propuestas ajustables del documento de diseño de S1.
// Se moverá a data/*.ron
pub fn builtin_recipes() -> &'static [Recipe] {
    &[
        // Trabajar escombro a mano para obtener tablones.
        Recipe {
            id: 1,
            station: StationKind::Hand,
            inputs: &[(ORE, 2)],
            output: (TIMBER, 1),
            craft_ticks: 30,
            noise_units: 100,
        },
        // Lámpara de cobre alimentada con carbón.
        Recipe {
            id: 2,
            station: StationKind::Workbench,
            inputs: &[(COPPER, 1), (COAL, 1)],
            output: (LAMP, 1),
            craft_ticks: 60,
            noise_units: 800,
        },
        // Pico de cobre: el primer pico fabricable.
        Recipe {
            id: 3,
            station: StationKind::Workbench,
            inputs: &[(COPPER, 2), (TIMBER, 1)],
            output: (COPPER_PICK, 1),
            craft_ticks: 90,
            noise_units: 1_000,
        },
        // Pico de hierro: más rápido, más ruidoso de forjar.
        Recipe {
            id: 4,
            station: StationKind::Workbench,
            inputs: &[(IRON, 2), (TIMBER, 1)],
            output: (IRON_PICK, 1),
            craft_ticks: 120,
            noise_units: 1_200,
        },
        // Llave de mesa de trabajo a partir de una ficha de latón.
        Recipe {
            id: 5,
            station: StationKind::Workbench,
            inputs: &[(COPPER, 2), (BRASS_TALLY, 1)],
            output: (WORKBENCH_KEY, 1),
            craft_ticks: 60,
            noise_units: 500,
        },
        // Fundir mena en cobre: la forja canta.
        Recipe {
            id: 6,
            station: StationKind::Forge,
            inputs: &[(ORE, 2), (COAL, 1)],
            output: (COPPER, 1),
            craft_ticks: 90,
            noise_units: 1_500,
        },
        // Fundir mena en hierro: aún más ruidoso.
        Recipe {
            id: 7,
            station: StationKind::Forge,
            inputs: &[(ORE, 3), (COAL, 2)],
            output: (IRON, 1),
            craft_ticks: 120,
            noise_units: 2_000,
        },
        // Carga silenciosa de los Quedos: ruido cero.
        Recipe {
            id: 8,
            station: StationKind::QuietBench,
            inputs: &[(IRON, 1), (COAL, 1)],
            output: (QUIET_CHARGE, 1),
            craft_ticks: 180,
            noise_units: 0,
        },
    ]
}

/// Indica si la receta puede elaborarse ahora mismo en esa estación:
/// ingredientes disponibles y hueco para el producto.
pub fn can_craft(inv: &Inventory, recipe: &Recipe, station: StationKind) -> bool {
    craft(&mut inv.clone(), recipe, station).is_ok()
}

/// Elabora la receta sobre el inventario de forma atómica y devuelve el ruido
/// generado, en milésimas de punto porcentual.
///
/// Comprueba primero la estación; después ensaya todos los cambios sobre una
/// copia y solo compromete el resultado si todo cabe. Ante cualquier error el
/// inventario queda exactamente como estaba.
pub fn craft(
    inv: &mut Inventory,
    recipe: &Recipe,
    station: StationKind,
) -> Result<u32, CraftError> {
    if station != recipe.station {
        return Err(CraftError::WrongStation {
            required: recipe.station,
        });
    }
    let mut staged = inv.clone();
    for &(item, count) in recipe.inputs {
        staged
            .try_remove(item, count)
            .map_err(|_| CraftError::MissingIngredient { item })?;
    }
    staged
        .try_add(ItemStack::new(recipe.output.0, recipe.output.1))
        .map_err(|_| CraftError::OutputDoesNotFit)?;
    *inv = staged;
    Ok(recipe.noise_units)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Receta del catálogo por identificador.
    fn recipe(id: u16) -> Recipe {
        *builtin_recipes().iter().find(|r| r.id == id).unwrap()
    }

    #[test]
    fn wrong_station_fails_without_touching_inventory() {
        let mut inv = Inventory::new(8);
        inv.try_add(ItemStack::new(ORE, 2)).unwrap();
        let hand_timber = recipe(1); // exige Hand

        let err = craft(&mut inv, &hand_timber, StationKind::Workbench).unwrap_err();
        assert_eq!(
            err,
            CraftError::WrongStation {
                required: StationKind::Hand
            }
        );
        assert!(!err.to_string().is_empty());
        assert_eq!(inv.count_item(ORE), 2);
        assert!(!can_craft(&inv, &hand_timber, StationKind::Workbench));
        assert!(can_craft(&inv, &hand_timber, StationKind::Hand));
    }

    #[test]
    fn missing_ingredient_leaves_inventory_intact() {
        let mut inv = Inventory::new(8);
        inv.try_add(ItemStack::new(ORE, 2)).unwrap(); // falta el carbón

        let smelt_copper = recipe(6);
        let err = craft(&mut inv, &smelt_copper, StationKind::Forge).unwrap_err();
        assert_eq!(err, CraftError::MissingIngredient { item: COAL });
        // La mena no se consumió aunque era el primer ingrediente: atómico.
        assert_eq!(inv.count_item(ORE), 2);
        assert_eq!(inv.count_item(COPPER), 0);
        assert!(!can_craft(&inv, &smelt_copper, StationKind::Forge));
    }

    #[test]
    fn smelting_ore_into_copper_returns_recipe_noise() {
        let mut inv = Inventory::new(8);
        inv.try_add(ItemStack::new(ORE, 2)).unwrap();
        inv.try_add(ItemStack::new(COAL, 1)).unwrap();

        let smelt_copper = recipe(6);
        assert_eq!(smelt_copper.noise_units, 1_500);
        let noise = craft(&mut inv, &smelt_copper, StationKind::Forge).unwrap();
        assert_eq!(noise, smelt_copper.noise_units);
        assert_eq!(inv.count_item(ORE), 0);
        assert_eq!(inv.count_item(COAL), 0);
        assert_eq!(inv.count_item(COPPER), 1);
    }

    #[test]
    fn quiet_charge_at_quiet_bench_makes_no_noise() {
        let mut inv = Inventory::new(8);
        inv.try_add(ItemStack::new(IRON, 1)).unwrap();
        inv.try_add(ItemStack::new(COAL, 1)).unwrap();

        let quiet = recipe(8);
        assert_eq!(quiet.station, StationKind::QuietBench);
        assert_eq!(craft(&mut inv, &quiet, StationKind::QuietBench).unwrap(), 0);
        assert_eq!(inv.count_item(QUIET_CHARGE), 1);
        assert!(builtin_recipes().iter().any(|r| r.noise_units == 0));
    }

    #[test]
    fn crafted_pick_enters_with_full_durability() {
        let mut inv = Inventory::new(8);
        inv.try_add(ItemStack::new(IRON, 2)).unwrap();
        inv.try_add(ItemStack::new(TIMBER, 1)).unwrap();

        let iron_pick = recipe(4);
        craft(&mut inv, &iron_pick, StationKind::Workbench).unwrap();
        assert_eq!(inv.count_item(IRON_PICK), 1);

        // `ItemStack::new` ya inicializa la durabilidad máxima de las
        // herramientas (120 según el diseño), así que el pico fabricado debe
        // ser idéntico a una pila recién creada.
        let fresh = ItemStack::new(IRON_PICK, 1);
        assert!((0..8).filter_map(|i| inv.slot(i)).any(|s| s == fresh));
    }

    #[test]
    fn builtin_catalog_is_well_formed() {
        let recipes = builtin_recipes();
        assert_eq!(recipes.len(), 8);
        for (index, recipe) in recipes.iter().enumerate() {
            assert_eq!(recipe.id, u16::try_from(index + 1).unwrap());
            assert!(!recipe.inputs.is_empty());
            assert!(recipe.craft_ticks > 0);
            assert!(recipe.noise_units <= crate::seismograph::MAX_LEVEL);
        }
    }
}
