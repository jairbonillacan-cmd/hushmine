//! Lo que el HUD del modo Historia necesita saber de la partida, copiado en
//! una estructura plana: así el dibujo no depende de la ventana y la CLI
//! puede capturarlo igual que lo ve el jugador.

use grisu_voxel::{RayHit, cell_of};
use hush_sim::BlockKind;
use hush_sim::story::items::ItemId;
use hush_sim::story::vitals::Vitals;
use hush_sim::story::{DayCycle, Expedition, Milestones, MiningState, Stratum};

/// Una casilla ocupada de la barra rápida.
#[derive(Clone, Debug, PartialEq)]
pub struct HotbarSlot {
    pub name: &'static str,
    pub count: u16,
    pub durability: u16,
    /// Durabilidad máxima; 0 si el objeto no se desgasta.
    pub max_durability: u16,
}

/// Una receta del menú de fabricación, ya resuelta contra el inventario.
#[derive(Clone, Debug, PartialEq)]
pub struct CraftRow {
    pub label: String,
    pub available: bool,
}

/// Todo lo que dibuja el HUD en un fotograma.
#[derive(Clone, Debug)]
pub struct HudView {
    pub paused: bool,
    pub f3_open: bool,
    pub dead: bool,
    pub craft_open: bool,
    pub craft_rows: Vec<CraftRow>,
    pub craft_selected: usize,
    /// Aviso grande (hito, muerte, asedio) y los segundos que le quedan.
    pub banner: Option<(&'static str, f32)>,
    pub vitals: Vitals,
    pub hotbar: Vec<Option<HotbarSlot>>,
    pub selected: usize,
    pub milestones: Milestones,
    pub tick: u32,
    pub clock: DayCycle,
    pub feet: [i64; 3],
    pub stratum: Stratum,
    pub sky_light: u8,
    pub block_light: u8,
    pub mobs: usize,
    pub target: Option<RayHit>,
    pub target_name: &'static str,
    /// Dureza Q10 del bloque apuntado.
    pub target_hardness: u32,
    /// Nivel mínimo de pico para que suelte material.
    pub target_tier: u8,
    pub mining: MiningState,
    // Telemetría (F3): la rellena la ventana.
    pub mesh_faces: usize,
    pub mesh_millis: f32,
    pub mesh_chunks: (usize, usize),
    pub fps: (f32, f32, f32),
    pub facing: &'static str,
    pub pitch: f32,
    pub yaw: f32,
    pub state_hash: u64,
}

impl HudView {
    /// Lo que sale de la expedición; lo propio de la ventana (pausa, menús,
    /// telemetría) queda en sus valores neutros para que lo complete ella.
    pub fn from_expedition(e: &Expedition, target: Option<RayHit>) -> Self {
        let feet = e.body().feet;
        let cell = feet.map(cell_of);
        let target_kind = target.and_then(|hit| {
            BlockKind::from_id(e.world().get(hit.block[0], hit.block[1], hit.block[2])).ok()
        });
        let hotbar = e
            .inventory()
            .hotbar
            .iter()
            .map(|slot| {
                slot.map(|s| HotbarSlot {
                    name: item_name(s.id),
                    count: s.count,
                    durability: s.durability,
                    // Solo las herramientas se desgastan.
                    max_durability: if s.id.tool_tier().is_some() {
                        s.id.max_durability()
                    } else {
                        0
                    },
                })
            })
            .collect();
        Self {
            paused: false,
            f3_open: false,
            dead: e.is_dead(),
            craft_open: false,
            craft_rows: Vec::new(),
            craft_selected: 0,
            banner: None,
            vitals: *e.vitals(),
            hotbar,
            selected: usize::from(e.inventory().selected),
            milestones: e.milestones().clone(),
            tick: e.tick(),
            clock: *e.clock(),
            feet,
            stratum: e.bands().at(cell[1]),
            sky_light: e.sky_light_at(cell),
            block_light: e.block_light_at(cell),
            mobs: e.mobs().list.len(),
            target,
            target_name: target_kind.map_or("nada", block_name),
            target_hardness: target_kind.map_or(0, hush_sim::story::mining::hardness_q10),
            target_tier: target_kind.map_or(0, hush_sim::story::mining::required_tier),
            mining: *e.mining(),
            mesh_faces: 0,
            mesh_millis: 0.0,
            mesh_chunks: (0, 0),
            fps: (0.0, 0.0, 0.0),
            facing: "",
            pitch: 0.0,
            yaw: 0.0,
            state_hash: e.state_hash(),
        }
    }
}

/// Nombre de un bloque como objeto.
fn block_name(kind: BlockKind) -> &'static str {
    item_name(ItemId::of_block(kind))
}

/// Nombre visible de un objeto, en español normal.
pub fn item_name(id: ItemId) -> &'static str {
    match id.name() {
        "dirt" => "Tierra",
        "rock" => "Roca",
        "hard_rock" => "Roca dura",
        "ore" => "Mena",
        "gold" => "Oro",
        "diamond" => "Diamante",
        "plank" => "Tablón",
        "log" => "Tronco",
        "torch" => "Antorcha",
        "workbench" => "Banco de trabajo",
        "furnace" => "Horno",
        "chest" => "Arcón",
        "gravel" => "Grava",
        "water" => "Agua",
        "obsidian" => "Obsidiana",
        "coal_ore" => "Veta de carbón",
        "coal" => "Carbón",
        "handle" => "Mango",
        "iron_ingot" => "Lingote de hierro",
        "gold_ingot" => "Lingote de oro",
        "diamond_gem" => "Gema de diamante",
        "coal_block" => "Bloque de carbón",
        "steel_ingot" => "Lingote de acero",
        "ration" => "Ración",
        "wood_pickaxe" => "Pico de madera",
        "wood_shovel" => "Pala de madera",
        "wood_axe" => "Hacha de madera",
        "wood_sword" => "Espada de madera",
        "stone_pickaxe" => "Pico de piedra",
        "stone_shovel" => "Pala de piedra",
        "stone_axe" => "Hacha de piedra",
        "stone_sword" => "Espada de piedra",
        "iron_pickaxe" => "Pico de hierro",
        "iron_shovel" => "Pala de hierro",
        "iron_axe" => "Hacha de hierro",
        "iron_sword" => "Espada de hierro",
        "steel_pickaxe" => "Pico de acero",
        "steel_shovel" => "Pala de acero",
        "steel_axe" => "Hacha de acero",
        "steel_sword" => "Espada de acero",
        "diamond_pickaxe" => "Pico de diamante",
        "diamond_shovel" => "Pala de diamante",
        "diamond_axe" => "Hacha de diamante",
        "diamond_sword" => "Espada de diamante",
        _ => "Objeto",
    }
}

/// FPS, percentil 1 % bajo (ventana de 120 cuadros) y tiempo de cuadro medio.
#[derive(Default)]
pub struct FpsTracker {
    frames: Vec<f32>,
}

impl FpsTracker {
    pub fn frame(&mut self, dt: f32) {
        self.frames.push(dt);
        if self.frames.len() > 120 {
            self.frames.remove(0);
        }
    }

    pub fn stats(&self) -> (f32, f32, f32) {
        if self.frames.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let total: f32 = self.frames.iter().sum();
        let fps = self.frames.len() as f32 / total.max(1e-6);
        let mut sorted = self.frames.clone();
        sorted.sort_by(f32::total_cmp);
        let cut = (sorted.len() / 100).max(1);
        let worst: f32 = sorted[sorted.len() - cut..].iter().sum();
        let low = cut as f32 / worst.max(1e-6);
        (fps, low, total * 1000.0 / self.frames.len() as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_tracker_window_and_percentile() {
        let mut fps = FpsTracker::default();
        for _ in 0..120 {
            fps.frame(1.0 / 60.0);
        }
        let (avg, low, ms) = fps.stats();
        assert!((avg - 60.0).abs() < 1.0);
        assert!((low - 60.0).abs() < 1.0);
        assert!((ms - 16.67).abs() < 0.2);
        // Un tirón de 200 ms hunde el 1 % bajo.
        fps.frame(0.2);
        assert!(fps.stats().1 < 10.0);
        for _ in 0..500 {
            fps.frame(1.0 / 60.0);
        }
        assert!(fps.frames.len() <= 120);
    }

    #[test]
    fn every_item_has_a_readable_name() {
        use hush_sim::story::items::{ToolKind, tool_item};
        for tier in 0..5 {
            for kind in ToolKind::ALL {
                let name = item_name(tool_item(tier, kind));
                assert_ne!(name, "Objeto", "{tier} {kind:?}");
                assert_ne!(name, name.to_uppercase());
            }
        }
        for kind in BlockKind::ALL.iter().skip(1) {
            let id = ItemId::of_block(*kind);
            // Los bloques sin nombre propio caen en «Objeto», nunca en mayúsculas.
            assert_ne!(item_name(id), item_name(id).to_uppercase());
        }
    }
}
