//! Catálogo de ítems del modo historia: bloques, materiales y herramientas.
//!
//! Los ítems son enteros de 16 bits. Los ids `0..=28` coinciden con los
//! bloques de la Convención A (así se colocan directamente); los materiales
//! y las herramientas viven a partir de 256. Todo determinista y sin
//! flotantes.

use serde::{Deserialize, Serialize};

use crate::BlockKind;

/// Ítem del modo historia: bloque colocable, material o herramienta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u16);

impl ItemId {
    /// Ítem asociado a un bloque colocable.
    pub const fn of_block(block: BlockKind) -> Self {
        Self(block.id() as u16)
    }

    /// Bloque asociado al ítem, si es colocable.
    pub fn as_block(self) -> Option<BlockKind> {
        BlockKind::from_id(u8::try_from(self.0).ok()?).ok()
    }

    pub fn name(self) -> &'static str {
        if let Some(block) = self.as_block() {
            return block.name();
        }
        if let Some((kind, tier)) = tool_info(self.0) {
            return match (tier, kind) {
                (0, ToolKind::Pickaxe) => "wood_pickaxe",
                (0, ToolKind::Shovel) => "wood_shovel",
                (0, ToolKind::Axe) => "wood_axe",
                (0, ToolKind::Sword) => "wood_sword",
                (1, ToolKind::Pickaxe) => "stone_pickaxe",
                (1, ToolKind::Shovel) => "stone_shovel",
                (1, ToolKind::Axe) => "stone_axe",
                (1, ToolKind::Sword) => "stone_sword",
                (2, ToolKind::Pickaxe) => "iron_pickaxe",
                (2, ToolKind::Shovel) => "iron_shovel",
                (2, ToolKind::Axe) => "iron_axe",
                (2, ToolKind::Sword) => "iron_sword",
                (3, ToolKind::Pickaxe) => "steel_pickaxe",
                (3, ToolKind::Shovel) => "steel_shovel",
                (3, ToolKind::Axe) => "steel_axe",
                (3, ToolKind::Sword) => "steel_sword",
                _ => match kind {
                    ToolKind::Pickaxe => "diamond_pickaxe",
                    ToolKind::Shovel => "diamond_shovel",
                    ToolKind::Axe => "diamond_axe",
                    ToolKind::Sword => "diamond_sword",
                },
            };
        }
        match self.0 {
            HANDLE => "handle",
            COAL => "coal",
            IRON_INGOT => "iron_ingot",
            GOLD_INGOT => "gold_ingot",
            DIAMOND_GEM => "diamond_gem",
            COAL_BLOCK => "coal_block",
            STEEL_INGOT => "steel_ingot",
            RATION => "ration",
            _ => "unknown",
        }
    }

    /// Máximo de unidades por casilla: 64 a granel, 16 recipientes y 1 para
    /// herramientas con desgaste.
    pub fn stack_max(self) -> u16 {
        if self.is_tool() { 1 } else { 64 }
    }

    pub fn is_tool(self) -> bool {
        tool_info(self.0).is_some()
    }

    /// Nivel de la herramienta (0 madera .. 4 diamante).
    pub fn tool_tier(self) -> Option<u8> {
        tool_info(self.0).map(|(_, tier)| tier)
    }

    /// Clase de la herramienta.
    pub fn tool_kind(self) -> Option<ToolKind> {
        tool_info(self.0).map(|(kind, _)| kind)
    }

    /// Eficiencia de minado de la herramienta en Q8 (256 = 1.0×): 2, 4, 6,
    /// 8 y 10 veces según el nivel.
    pub fn efficiency_q8(self) -> u16 {
        if !self.is_tool() {
            return 256;
        }
        512 * u16::from(self.tool_tier().unwrap_or(0) + 1)
    }

    /// Usos restantes de una herramienta recién fabricada por nivel:
    /// 59, 131, 250, 1 561 y 2 031.
    pub fn max_durability(self) -> u16 {
        const DURABILITY: [u16; 5] = [59, 131, 250, 1_561, 2_031];
        DURABILITY[usize::from(self.tool_tier().unwrap_or(0))]
    }

    /// Ticks de horno que aporta como combustible (0 = no arde):
    /// astillas 100, tablón 300, hulla 1 600 y bloque de carbón 16 000.
    pub fn fuel_ticks(self) -> u32 {
        match self.0 {
            HANDLE => 100,
            PLANK_ITEM | LOG_ITEM => 300,
            COAL => 1_600,
            COAL_BLOCK => 16_000,
            _ => 0,
        }
    }
}

impl From<BlockKind> for ItemId {
    fn from(block: BlockKind) -> Self {
        Self::of_block(block)
    }
}

// Ítems de bloque (los ids coinciden con la Convención A).
pub const PLANK_ITEM: u16 = BlockKind::Plank.id() as u16;
pub const LOG_ITEM: u16 = BlockKind::Log.id() as u16;
pub const TORCH_ITEM: u16 = BlockKind::Torch.id() as u16;

/// Materiales y provisiones (ids 256+).
pub const HANDLE: u16 = 256;
pub const COAL: u16 = 257;
pub const IRON_INGOT: u16 = 258;
pub const GOLD_INGOT: u16 = 259;
pub const DIAMOND_GEM: u16 = 260;
pub const COAL_BLOCK: u16 = 261;
pub const STEEL_INGOT: u16 = 262;
/// Ración de expedición: comida de campo.
pub const RATION: u16 = 270;

/// Primera herramienta: 512 + nivel*4 + clase.
const TOOL_BASE: u16 = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Pickaxe,
    Shovel,
    Axe,
    Sword,
}

impl ToolKind {
    pub const ALL: [Self; 4] = [Self::Pickaxe, Self::Shovel, Self::Axe, Self::Sword];

    pub fn label(self) -> &'static str {
        match self {
            Self::Pickaxe => "pico",
            Self::Shovel => "pala",
            Self::Axe => "hacha",
            Self::Sword => "espada",
        }
    }
}

/// Herramienta del nivel `tier` (0..=4) y clase `kind`.
pub fn tool_item(tier: u8, kind: ToolKind) -> ItemId {
    ItemId(TOOL_BASE + u16::from(tier) * 4 + kind as u16)
}

/// `(clase, nivel)` si el ítem es una herramienta.
pub fn tool_info(id: u16) -> Option<(ToolKind, u8)> {
    if id < TOOL_BASE {
        return None;
    }
    let offset = id - TOOL_BASE;
    let tier = offset / 4;
    if tier > 4 {
        return None;
    }
    let kind = match offset % 4 {
        0 => ToolKind::Pickaxe,
        1 => ToolKind::Shovel,
        2 => ToolKind::Axe,
        _ => ToolKind::Sword,
    };
    Some((kind, tier as u8))
}

/// Nutrición que aporta una ración: 6 puntos de comida y 6 de saturación.
pub const RATION_FOOD: u8 = 6;
pub const RATION_SATURATION: u8 = 6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_items_round_trip() {
        for block in BlockKind::ALL {
            let item = ItemId::of_block(block);
            assert_eq!(item.as_block(), Some(block));
            assert_eq!(item.name(), block.name());
        }
        assert_eq!(ItemId(HANDLE).as_block(), None);
        assert_eq!(ItemId(999).name(), "unknown");
    }

    #[test]
    fn tools_have_five_tiers_and_four_kinds() {
        for tier in 0..5u8 {
            for kind in ToolKind::ALL {
                let item = tool_item(tier, kind);
                assert_eq!(item.tool_tier(), Some(tier));
                assert_eq!(item.tool_kind(), Some(kind));
                assert_eq!(item.stack_max(), 1);
                assert!(item.is_tool());
            }
        }
        // Eficiencia Q8: 2×, 4×, 6×, 8× y 10×.
        let effs: Vec<u16> = (0..5)
            .map(|t| tool_item(t, ToolKind::Pickaxe).efficiency_q8())
            .collect();
        assert_eq!(effs, [512, 1024, 1536, 2048, 2560]);
        // Durabilidad del documento.
        assert_eq!(tool_item(0, ToolKind::Axe).max_durability(), 59);
        assert_eq!(tool_item(3, ToolKind::Pickaxe).max_durability(), 1_561);
        assert_eq!(tool_item(4, ToolKind::Sword).max_durability(), 2_031);
        assert_eq!(ItemId(PLANK_ITEM).efficiency_q8(), 256);
    }

    #[test]
    fn fuel_matches_document_table() {
        assert_eq!(ItemId(HANDLE).fuel_ticks(), 100);
        assert_eq!(ItemId(PLANK_ITEM).fuel_ticks(), 300);
        assert_eq!(ItemId(LOG_ITEM).fuel_ticks(), 300);
        assert_eq!(ItemId(COAL).fuel_ticks(), 1_600);
        assert_eq!(ItemId(COAL_BLOCK).fuel_ticks(), 16_000);
        assert_eq!(ItemId(TORCH_ITEM).fuel_ticks(), 0);
    }

    #[test]
    fn tool_names_are_stable() {
        assert_eq!(tool_item(0, ToolKind::Pickaxe).name(), "wood_pickaxe");
        assert_eq!(tool_item(2, ToolKind::Axe).name(), "iron_axe");
        assert_eq!(tool_item(4, ToolKind::Shovel).name(), "diamond_shovel");
        assert_eq!(ItemId(COAL).name(), "coal");
        assert_eq!(ItemId(STEEL_INGOT).name(), "steel_ingot");
    }
}
