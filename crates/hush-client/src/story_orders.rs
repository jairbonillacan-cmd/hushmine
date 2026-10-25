//! Órdenes del modo historia: entrada de movimiento y menú de fabricación
//! compacto (recetas con los materiales del inventario).

use grisu_platform::{Input, KeyCode};
use hush_sim::story::items::{ItemId, ToolKind, tool_item};
use hush_sim::story::{Expedition, StoryOrder};
use hush_sim::{BlockKind, MoveInput};

/// Dirección de movimiento en ejes del mundo a partir de la cámara. La
/// simulación solo ve enteros cuantizados.
pub fn move_input_from_camera(
    input: &Input,
    flat_forward: glam::Vec3,
    flat_right: glam::Vec3,
) -> MoveInput {
    let axis = |pos: KeyCode, neg: KeyCode| {
        f32::from(u8::from(input.is_held(pos))) - f32::from(u8::from(input.is_held(neg)))
    };
    let forward = axis(KeyCode::KeyW, KeyCode::KeyS);
    let right = axis(KeyCode::KeyD, KeyCode::KeyA);
    let wish = flat_forward * forward + flat_right * right;
    let jump = input.is_held(KeyCode::Space);
    let mut movement = MoveInput::from_direction(wish.x, wish.z, jump);
    movement.sprint = input.is_held(KeyCode::ControlLeft);
    movement
}

/// Una receta del menú compacto.
struct CompactRecipe {
    label: String,
    /// Matriz 3×3 en orden de fila (`None` = vacía).
    cells: [Option<u16>; 9],
    output: ItemId,
    count: u16,
}

fn compact_recipes() -> Vec<CompactRecipe> {
    let log = BlockKind::Log.id() as u16;
    let plank = BlockKind::Plank.id() as u16;
    let rock = BlockKind::Rock.id() as u16;
    let handle = 256; // HANDLE
    let coal = 257; // COAL
    let torch = BlockKind::Torch.id() as u16;
    let iron = 258; // IRON_INGOT
    let steel = 262; // STEEL_INGOT
    let diamond = 260; // DIAMOND_GEM
    let simple = |label: &str, cells: &[Option<u16>], out: u16, count: u16| CompactRecipe {
        label: label.to_owned(),
        cells: pad(cells),
        output: ItemId(out),
        count,
    };
    let mut list = vec![
        simple("TABLONES", &[Some(log)], plank, 4),
        simple("MANGOS", &[Some(plank), Some(plank)], handle, 4),
        simple("ANTORCHAS", &[Some(coal), Some(handle)], torch, 4),
        simple(
            "BANCO DE TRABAJO",
            &[Some(plank); 4],
            BlockKind::Workbench.id() as u16,
            1,
        ),
        simple(
            "HORNO DE PIEDRA",
            &[Some(rock); 8],
            BlockKind::Furnace.id() as u16,
            1,
        ),
        simple(
            "ARCON DE CARGA",
            &[Some(plank); 8],
            BlockKind::Chest.id() as u16,
            1,
        ),
        simple("ACERO", &[Some(iron), Some(coal), Some(coal)], steel, 1),
    ];
    // Herramientas de los cinco materiales con los esquemas tradicionales.
    let materials = [
        ("MADERA", plank),
        ("PIEDRA", rock),
        ("HIERRO", iron),
        ("ACERO", steel),
        ("DIAMANTE", diamond),
    ];
    for (tier, (material_label, material)) in materials.iter().enumerate() {
        for (kind_label, kind) in [
            ("PICO", ToolKind::Pickaxe),
            ("PALA", ToolKind::Shovel),
            ("HACHA", ToolKind::Axe),
            ("ESPADA", ToolKind::Sword),
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
                label: format!("{kind_label} DE {material_label}"),
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

/// Menú de fabricación compacto: F abre, flechas eligen, ENTER fabrica.
pub struct CraftMenu {
    open: bool,
    selected: usize,
    recipes: Vec<CompactRecipe>,
}

impl Default for CraftMenu {
    fn default() -> Self {
        Self {
            open: false,
            selected: 0,
            recipes: compact_recipes(),
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

    pub fn update(&mut self, input: &Input, expedition: &mut Expedition) {
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
            let before = expedition.inventory().count_of(recipe.output);
            expedition.step(&[StoryOrder::Craft {
                grid,
                workbench: true,
            }]);
            let after = expedition.inventory().count_of(recipe.output);
            if after > before {
                log::info!("fabricado: {}", recipe.label);
            } else {
                log::info!("faltan materiales para: {}", recipe.label);
            }
        }
    }

    /// Filas del menú: (etiqueta con producto, ¿tiene materiales?).
    pub fn rows(&self, expedition: &Expedition) -> Vec<(String, bool)> {
        self.recipes
            .iter()
            .map(|recipe| {
                let has_all = recipe
                    .cells
                    .iter()
                    .flatten()
                    .all(|id| expedition.inventory().count_of(ItemId(*id)) > 0);
                (
                    format!(
                        "{} -> {} x{}",
                        recipe.label,
                        recipe.output.name(),
                        recipe.count
                    ),
                    has_all,
                )
            })
            .collect()
    }

    pub fn selected(&self) -> usize {
        self.selected
    }
}
