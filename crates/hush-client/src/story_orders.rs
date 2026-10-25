//! Órdenes del modo historia: entrada de movimiento y menú de fabricación
//! compacto (recetas con los materiales del inventario).

use grisu_platform::{Input, KeyCode};
use hush_client::controls::Controls;
use hush_client::story_view::{CraftRow, item_name};
use hush_sim::story::items::{
    COAL, DIAMOND_GEM, HANDLE, IRON_INGOT, ItemId, STEEL_INGOT, ToolKind, tool_item,
};
use hush_sim::story::{Expedition, StoryOrder};
use hush_sim::{BlockKind, MoveInput};

/// Dirección de movimiento en ejes del mundo a partir de la cámara. La
/// simulación solo ve enteros cuantizados.
pub fn move_input_from_camera(
    input: &Input,
    controls: &Controls,
    flat_forward: glam::Vec3,
    flat_right: glam::Vec3,
) -> MoveInput {
    let axis = |pos: KeyCode, neg: KeyCode| {
        f32::from(u8::from(input.is_held(pos))) - f32::from(u8::from(input.is_held(neg)))
    };
    // Las mismas teclas reasignables que el modo Turno (out/controls.cfg).
    let forward = axis(controls.forward, controls.back);
    let right = axis(controls.right, controls.left);
    let wish = flat_forward * forward + flat_right * right;
    let crouch = input.is_held(controls.crouch);
    MoveInput::from_direction(wish.x, wish.z, input.is_held(controls.jump))
        .with_sprint(!crouch && input.is_held(controls.sprint))
        .with_crouch(crouch)
}

/// Una receta del menú compacto.
struct CompactRecipe {
    /// Matriz 3×3 en orden de fila (`None` = vacía).
    cells: [Option<u16>; 9],
    output: ItemId,
    count: u16,
}

fn compact_recipes() -> Vec<CompactRecipe> {
    let log = BlockKind::Log.id() as u16;
    let plank = BlockKind::Plank.id() as u16;
    let rock = BlockKind::Rock.id() as u16;
    let (handle, coal, iron, steel, diamond) = (HANDLE, COAL, IRON_INGOT, STEEL_INGOT, DIAMOND_GEM);
    let torch = BlockKind::Torch.id() as u16;
    let simple = |cells: &[Option<u16>], out: u16, count: u16| CompactRecipe {
        cells: pad(cells),
        output: ItemId(out),
        count,
    };
    let mut list = vec![
        simple(&[Some(log)], plank, 4),
        simple(&[Some(plank), Some(plank)], handle, 4),
        simple(&[Some(coal), Some(handle)], torch, 4),
        simple(&[Some(plank); 4], BlockKind::Workbench.id() as u16, 1),
        simple(&[Some(rock); 8], BlockKind::Furnace.id() as u16, 1),
        simple(&[Some(plank); 8], BlockKind::Chest.id() as u16, 1),
        simple(&[Some(iron), Some(coal), Some(coal)], steel, 1),
    ];
    // Herramientas de los cinco materiales con los esquemas tradicionales.
    let materials = [plank, rock, iron, steel, diamond];
    for (tier, material) in materials.iter().enumerate() {
        for kind in [
            ToolKind::Pickaxe,
            ToolKind::Shovel,
            ToolKind::Axe,
            ToolKind::Sword,
        ] {
            let m = Some(*material);
            let s = Some(handle);
            let cells = match kind {
                ToolKind::Pickaxe => [m, m, m, None, s, None, None, s, None],
                ToolKind::Shovel => [None, m, None, None, s, None, None, s, None],
                ToolKind::Axe => [m, m, None, m, s, None, None, s, None],
                ToolKind::Sword => [None, m, None, None, m, None, None, s, None],
            };
            let output = tool_item(tier as u8, kind);
            list.push(CompactRecipe {
                cells,
                output,
                count: 1,
            });
        }
    }
    list
}

/// Rellena una matriz corta hasta 9 casillas.
fn pad(cells: &[Option<u16>]) -> [Option<u16>; 9] {
    let mut grid = [None; 9];
    for (slot, cell) in grid.iter_mut().zip(cells) {
        *slot = *cell;
    }
    grid
}

/// Menú de fabricación compacto: F abre, flechas eligen, Enter fabrica.
pub struct CraftMenu {
    open: bool,
    selected: usize,
    recipes: Vec<CompactRecipe>,
    /// Orden de fabricar pendiente: sale con el tick siguiente, como el
    /// resto. (Antes se aplicaba al momento con un `step` extra, que hacía
    /// avanzar la simulación un tick de más en cada fabricación.)
    pending: Option<StoryOrder>,
}

impl Default for CraftMenu {
    fn default() -> Self {
        Self {
            open: false,
            selected: 0,
            recipes: compact_recipes(),
            pending: None,
        }
    }
}

impl CraftMenu {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// La orden de fabricar pendiente, si la hay.
    pub fn take_pending(&mut self) -> Option<StoryOrder> {
        self.pending.take()
    }

    pub fn update(&mut self, input: &Input) {
        if input.was_pressed(KeyCode::KeyF) {
            self.open = !self.open;
            return;
        }
        if !self.open {
            return;
        }
        if input.was_pressed(KeyCode::ArrowDown) {
            self.selected = (self.selected + 1) % self.recipes.len();
        }
        if input.was_pressed(KeyCode::ArrowUp) {
            self.selected = (self.selected + self.recipes.len() - 1) % self.recipes.len();
        }
        if input.was_pressed(KeyCode::Enter) {
            let recipe = &self.recipes[self.selected];
            let cells = recipe.cells;
            let grid = [
                [cells[0], cells[1], cells[2]],
                [cells[3], cells[4], cells[5]],
                [cells[6], cells[7], cells[8]],
            ];
            self.pending = Some(StoryOrder::Craft {
                grid,
                workbench: true,
            });
        }
    }

    /// Filas del menú: nombre del producto (con la cantidad si da más de
    /// uno) y si el inventario tiene todos los materiales en su cantidad.
    pub fn rows(&self, expedition: &Expedition) -> Vec<CraftRow> {
        self.recipes
            .iter()
            .map(|recipe| {
                let mut needed: Vec<(u16, u32)> = Vec::new();
                for &id in recipe.cells.iter().flatten() {
                    match needed.iter_mut().find(|(n, _)| *n == id) {
                        Some((_, count)) => *count += 1,
                        None => needed.push((id, 1)),
                    }
                }
                let available = needed
                    .iter()
                    .all(|&(id, n)| expedition.inventory().count_of(ItemId(id)) >= n);
                let name = item_name(recipe.output);
                let label = if recipe.count > 1 {
                    format!("{name} ×{}", recipe.count)
                } else {
                    name.to_owned()
                };
                CraftRow { label, available }
            })
            .collect()
    }

    pub fn selected(&self) -> usize {
        self.selected
    }
}
