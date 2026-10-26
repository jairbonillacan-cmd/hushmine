//! Fabricación matricial y metalurgia térmica del modo historia.
//!
//! Recetas por patrón espacial en cuadrícula (2×2 portátil y 3×3 de banco)
//! y horno de fundición con combustible, curva térmica de 200 ticks y
//! variantes de alto horno y ahumador al doble de velocidad.

use serde::{Deserialize, Serialize};

use super::inventory::{Container, ItemStack, Slot};
use super::items::{
    COAL, COAL_BLOCK, DIAMOND_GEM, GOLD_INGOT, HANDLE, IRON_INGOT, ItemId, PLANK_ITEM, RATION,
    STEEL_INGOT, TORCH_ITEM, ToolKind, tool_item,
};

/// Lado máximo de la matriz de fabricación.
pub const GRID: usize = 3;

/// Ticks de simulación que dura una calcinación: 10 segundos exactos.
pub const SMELT_TICKS: u32 = 200;

/// Una receta: patrón rectangular de ítems y su producto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub name: &'static str,
    /// Patrón de `width` × `height`, leído por filas.
    pub cells: Vec<Option<ItemId>>,
    pub width: usize,
    pub height: usize,
    pub output: ItemStack,
}

impl Recipe {
    fn new(
        name: &'static str,
        cells: &[Option<ItemId>],
        width: usize,
        output_id: u16,
        output_count: u16,
    ) -> Self {
        Self {
            name,
            cells: cells.to_vec(),
            width,
            height: cells.len() / width,
            output: ItemStack::new(ItemId(output_id), output_count),
        }
    }
}

/// Catálogo completo de recetas (del documento de diseño, secciones 4.1-4.2).
pub fn recipes() -> &'static [Recipe] {
    use super::items::LOG_ITEM;
    static RECIPES: std::sync::OnceLock<Vec<Recipe>> = std::sync::OnceLock::new();
    RECIPES.get_or_init(|| {
        let plank = ItemId(PLANK_ITEM);
        let log = ItemId(LOG_ITEM);
        let handle = ItemId(HANDLE);
        let coal = ItemId(COAL);
        let mut list = vec![
            // 4.1: fabricación portátil.
            Recipe::new("planks", &[Some(log)], 1, PLANK_ITEM, 4),
            Recipe::new("handles", &[Some(plank), Some(plank)], 1, HANDLE, 4),
            Recipe::new("torches", &[Some(coal), Some(handle)], 1, TORCH_ITEM, 4),
            Recipe::new(
                "workbench",
                &[Some(plank), Some(plank), Some(plank), Some(plank)],
                2,
                u16::from(crate::BlockKind::Workbench.id()),
                1,
            ),
            Recipe::new("ration", &[Some(plank), Some(coal)], 1, RATION, 1),
            Recipe::new(
                "coal_block",
                &[Some(coal); GRID * GRID],
                GRID,
                COAL_BLOCK,
                1,
            ),
            Recipe::new(
                "chest",
                &[
                    Some(plank),
                    Some(plank),
                    Some(plank),
                    Some(plank),
                    None,
                    Some(plank),
                    Some(plank),
                    Some(plank),
                    Some(plank),
                ],
                GRID,
                u16::from(crate::BlockKind::Chest.id()),
                1,
            ),
            Recipe::new(
                "furnace",
                &[
                    Some(rock()),
                    Some(rock()),
                    Some(rock()),
                    Some(rock()),
                    None,
                    Some(rock()),
                    Some(rock()),
                    Some(rock()),
                    Some(rock()),
                ],
                GRID,
                u16::from(crate::BlockKind::Furnace.id()),
                1,
            ),
            // Acero: lingote de hierro reforzado con carbón.
            Recipe::new(
                "steel_ingot",
                &[Some(ItemId(IRON_INGOT)), Some(coal), Some(coal)],
                3,
                STEEL_INGOT,
                1,
            ),
        ];
        // 4.2: herramientas de los cinco materiales (pico, pala, hacha y
        // espada) con los esquemas tradicionales.
        let materials = [
            (0u8, plank),
            (1u8, rock()),
            (2u8, ItemId(IRON_INGOT)),
            (3u8, ItemId(STEEL_INGOT)),
            (4u8, ItemId(DIAMOND_GEM)),
        ];
        for (tier, m) in materials {
            let s = Some(handle);
            let mm = Some(m);
            list.push(Recipe::new(
                "pickaxe",
                &[mm, mm, mm, None, s, None, None, s, None],
                GRID,
                tool_item(tier, ToolKind::Pickaxe).0,
                1,
            ));
            list.push(Recipe::new(
                "shovel",
                &[None, mm, None, None, s, None, None, s, None],
                GRID,
                tool_item(tier, ToolKind::Shovel).0,
                1,
            ));
            list.push(Recipe::new(
                "axe",
                &[mm, mm, None, mm, s, None, None, s, None],
                GRID,
                tool_item(tier, ToolKind::Axe).0,
                1,
            ));
            list.push(Recipe::new(
                "sword",
                &[None, mm, None, None, mm, None, None, s, None],
                GRID,
                tool_item(tier, ToolKind::Sword).0,
                1,
            ));
        }
        list
    })
}

fn rock() -> ItemId {
    ItemId::of_block(crate::BlockKind::Rock)
}

/// Busca la receta que encaja en una matriz de 3×3 (o 2×2 en las casillas
/// superiores izquierdas) con traslación libre dentro de la matriz.
///
/// `grid` se recibe por filas; `None` es casilla vacía. Las recetas de
/// matriz 3×3 solo encajan si `grid` es 3×3 (banco de trabajo).
pub fn match_recipe(
    grid: &[[Option<ItemId>; GRID]; GRID],
    workbench: bool,
) -> Option<&'static Recipe> {
    recipes().iter().find(|recipe| {
        if recipe.width > GRID || recipe.height > GRID {
            return false;
        }
        if (recipe.width > 2 || recipe.height > 2) && !workbench {
            return false;
        }
        for dy in 0..=(GRID - recipe.height) {
            for dx in 0..=(GRID - recipe.width) {
                let mut fits = true;
                for (y, row) in grid.iter().enumerate() {
                    for (x, &cell) in row.iter().enumerate() {
                        let in_pattern =
                            x >= dx && x < dx + recipe.width && y >= dy && y < dy + recipe.height;
                        let want = if in_pattern {
                            recipe.cells[(y - dy) * recipe.width + (x - dx)]
                        } else {
                            None
                        };
                        if cell != want {
                            fits = false;
                        }
                    }
                }
                if fits {
                    return true;
                }
            }
        }
        false
    })
}

/// Consume los ingredientes de la matriz y devuelve el producto.
pub fn craft(grid: &mut [[Option<ItemStack>; GRID]; GRID], workbench: bool) -> Option<ItemStack> {
    let ids = grid.map(|row| row.map(|slot| slot.map(|s| s.id)));
    let recipe = match_recipe(&ids, workbench)?;
    let mut remaining = recipe.output;
    for row in grid.iter_mut() {
        for slot in row.iter_mut() {
            if let Some(stack) = slot {
                stack.count -= 1;
                if stack.count == 0 {
                    *slot = None;
                }
            }
        }
    }
    // Las herramientas salen con la durabilidad nueva.
    if recipe.output.id.is_tool() {
        remaining.durability = recipe.output.id.max_durability();
    }
    Some(remaining)
}

/// Qué funde el horno: menas y materiales crudos.
pub fn smelt_result(input: ItemId) -> Option<ItemId> {
    use crate::BlockKind;
    match input {
        id if id == ItemId::of_block(BlockKind::Ore) => Some(ItemId(IRON_INGOT)),
        id if id == ItemId::of_block(BlockKind::Gold) => Some(ItemId(GOLD_INGOT)),
        _ => None,
    }
}

/// Tipo de horno: piedra, alto horno metalúrgico (solo metales, al doble de
/// velocidad) y ahumador (solo comida, al doble de velocidad).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FurnaceKind {
    #[default]
    Stone,
    Blast,
    Smoker,
}

impl FurnaceKind {
    /// Multiplicador de velocidad de cocción (en Q8).
    pub fn speed_q8(self) -> u16 {
        match self {
            Self::Stone => 256,
            Self::Blast | Self::Smoker => 512,
        }
    }

    /// Si este horno acepta el material indicado.
    pub fn accepts(self, input: ItemId) -> bool {
        match self {
            Self::Stone => true,
            // El alto horno no acepta alimentos ni madera: solo metales.
            Self::Blast => smelt_result(input).is_some(),
            // El ahumador solo cocina raciones.
            Self::Smoker => input.0 == RATION,
        }
    }
}

/// Estado de un horno colocado en el mundo.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FurnaceState {
    pub kind: FurnaceKind,
    pub input: Slot,
    pub fuel: Slot,
    pub output: Slot,
    /// Ticks de combustible restantes en el ciclo actual.
    pub burn_left: u32,
    /// Progreso de la calcinación en curso (0..SMELT_TICKS, escalado por la
    /// velocidad del horno).
    pub cook: u32,
}

impl FurnaceState {
    pub fn new(kind: FurnaceKind) -> Self {
        Self {
            kind,
            input: None,
            fuel: None,
            output: None,
            burn_left: 0,
            cook: 0,
        }
    }

    /// Avanza un tick de horno. Devuelve el producto cuando termina una
    /// calcinación.
    pub fn step(&mut self) -> Option<ItemStack> {
        let input = self.input?;
        let product = smelt_result(input.id)?;
        if !self.kind.accepts(input.id) {
            return None;
        }
        // ¿Cabe el producto?
        if let Some(out) = self.output {
            let stackable = out.id == product && out.id.stack_max() > 1;
            let full = !stackable || out.count >= out.id.stack_max();
            if full {
                return None;
            }
        }
        // Encender combustible si hace falta.
        if self.burn_left == 0 {
            let fuel_ticks = self.fuel.map(|f| f.id.fuel_ticks()).unwrap_or(0);
            if fuel_ticks == 0 {
                return None;
            }
            self.burn_left = fuel_ticks;
            if let Some(f) = self.fuel.as_mut() {
                f.count -= 1;
                if f.count == 0 {
                    self.fuel = None;
                }
            }
        }
        self.burn_left = self.burn_left.saturating_sub(1);
        // La curva térmica completa son SMELT_TICKS de trabajo; los hornos
        // rápidos acumulan el doble por tick.
        self.cook = self.cook.saturating_add(u32::from(self.kind.speed_q8()));
        if self.cook < SMELT_TICKS * 256 {
            return None;
        }
        self.cook = 0;
        if let Some(f) = self.input.as_mut() {
            f.count -= 1;
            if f.count == 0 {
                self.input = None;
            }
        }
        let cooked = ItemStack::new(product, 1);
        match self.output.as_mut() {
            Some(out) if out.id == product => out.count += 1,
            None => self.output = Some(cooked),
            _ => return None,
        }
        Some(cooked)
    }
}

/// Introduce un ítem en el horno desde un contenedor de origen (tolva o
/// jugador). Devuelve `true` si lo aceptó.
pub fn feed_furnace(furnace: &mut FurnaceState, stack: &mut ItemStack) -> bool {
    if smelt_result(stack.id).is_some() && furnace.input.is_none() {
        let one = ItemStack {
            id: stack.id,
            count: 1,
            durability: stack.durability,
            metadata: stack.metadata,
        };
        furnace.input = Some(one);
        stack.count -= 1;
        true
    } else if stack.id.fuel_ticks() > 0 && furnace.fuel.is_none() {
        let one = ItemStack {
            id: stack.id,
            count: 1,
            durability: stack.durability,
            metadata: stack.metadata,
        };
        furnace.fuel = Some(one);
        stack.count -= 1;
        true
    } else {
        false
    }
}

/// Receptor de tolva: horno (alimentación o combustible) o arcón.
pub enum HopperTarget<'a> {
    Furnace(&'a mut FurnaceState),
    Chest(&'a mut Container),
}

#[cfg(test)]
#[path = "crafting_tests.rs"]
mod tests;
