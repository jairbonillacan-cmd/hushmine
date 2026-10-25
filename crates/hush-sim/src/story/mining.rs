//! Dureza de materiales, tiers de herramientas y progreso de excavación.
//!
//! El daño por tick es `multiplicador / (H × 30)`; si la herramienta no
//! alcanza el nivel requerido, el bloque se rompe a velocidad reducida y no
//! suelta material (polvo estéril). La fractura se muestra en 10 fases.

use crate::BlockKind;

use super::items::{ItemId, ToolKind};

/// Denominador del daño por tick (el 30 del documento, a 20 Hz).
pub const TICKS_FACTOR: u32 = 30;
/// Fases de fractura superpuestas durante la excavación.
pub const FRACTURE_STAGES: u8 = 10;

/// Clase de herramienta eficaz contra cada bloque.
pub fn tool_class(block: BlockKind) -> Option<ToolKind> {
    use BlockKind::*;
    match block {
        Rock | HardRock | Ore | Gold | Diamond | CoalOre | Obsidian | Furnace => {
            Some(ToolKind::Pickaxe)
        }
        Dirt | Gravel => Some(ToolKind::Shovel),
        Log | Plank | PropBeam | Workbench | Chest => Some(ToolKind::Axe),
        _ => None,
    }
}

/// Dureza nominal H de cada bloque (Q10: 10 = 1.0).
pub fn hardness_q10(block: BlockKind) -> u32 {
    use BlockKind::*;
    match block {
        Air | Water | Lava => 0,
        Torch => 0,
        Ladder | Rail | Beacon => 3,
        Dirt => 5,
        Gravel => 6,
        Log => 20,
        Plank | PropBeam => 20,
        Workbench | Chest => 25,
        Rock => 15,
        CoalOre | Ore => 30,
        Gold => 30,
        Diamond => 30,
        Furnace => 35,
        HardRock => 30,
        Obsidian => 300,
        // Bloques de equipo del modo competitivo.
        TeamBlock | JumpPad | Explosive => 15,
        Electric | BucketPost => 30,
        ForceField | BucketStation => 45,
    }
}

/// Nivel mínimo de herramienta para que el bloque suelte material.
pub fn required_tier(block: BlockKind) -> u8 {
    use BlockKind::*;
    match block {
        Ore => 1,                // pico de piedra: hierro, cobre
        Gold => 2,               // pico de hierro: oro, plata
        Diamond | Obsidian => 3, // acero: tungsteno, diamante, obsidiana
        HardRock => 1,
        _ => 0,
    }
}

/// Ticks totales para romper el bloque con la herramienta empuñada
/// (`None` si no se puede picar). La eficiencia llega en Q8.
pub fn break_ticks(block: BlockKind, held: Option<ItemStackLite>) -> Option<u32> {
    if !block.is_diggable() {
        return None;
    }
    let hardness = hardness_q10(block);
    if hardness == 0 {
        return Some(1);
    }
    // Multiplicador efectivo en Q8: solo aplica si la clase de herramienta
    // coincide con el bloque; si no, velocidad base 1.0×.
    let mult_q8 = held
        .filter(|s| s.tool_kind == tool_class(block))
        .map_or(256, |s| u32::from(s.efficiency_q8));
    // ticks = ceil(H/10 × 30 × 256 / mult): H en Q10 y mult en Q8.
    let work = hardness * TICKS_FACTOR * 256;
    Some(work.div_ceil(10 * mult_q8))
}

/// Vista mínima de la pila empuñada (evita depender del inventario).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemStackLite {
    pub id: ItemId,
    pub tool_kind: Option<ToolKind>,
    pub tier: Option<u8>,
    pub efficiency_q8: u16,
}

impl ItemStackLite {
    pub fn of(id: ItemId) -> Self {
        Self {
            id,
            tool_kind: id.tool_kind(),
            tier: id.tool_tier(),
            efficiency_q8: id.efficiency_q8(),
        }
    }

    pub fn bare_hand() -> Self {
        Self::of(ItemId(0))
    }
}

/// ¿Suelta material al romperse? La herramienta debe alcanzar el nivel.
pub fn drops_item(block: BlockKind, held: Option<ItemStackLite>) -> bool {
    let need = required_tier(block);
    let have = held.and_then(|s| s.tier).unwrap_or(0);
    have >= need
}

/// Ítem que suelta el bloque (algunos menas sueltan material procesado).
pub fn drop_of(block: BlockKind) -> Option<ItemId> {
    use super::items::{COAL, DIAMOND_GEM};
    match block {
        BlockKind::CoalOre => Some(ItemId(COAL)),
        BlockKind::Diamond => Some(ItemId(DIAMOND_GEM)),
        BlockKind::Water | BlockKind::Lava | BlockKind::Air => None,
        other => Some(ItemId::of_block(other)),
    }
}

/// Fase de fractura (0..=10) según el progreso acumulado.
pub fn fracture_stage(progress: u32, total_ticks: u32) -> u8 {
    if total_ticks == 0 {
        return FRACTURE_STAGES;
    }
    let stage = progress * u32::from(FRACTURE_STAGES) / total_ticks;
    stage.min(u32::from(FRACTURE_STAGES)) as u8
}

#[cfg(test)]
mod tests {
    use super::super::items::{ItemId, ToolKind, tool_item};
    use super::*;
    use crate::BlockKind;

    fn lite(id: ItemId) -> Option<ItemStackLite> {
        Some(ItemStackLite::of(id))
    }

    #[test]
    fn hand_breaks_dirt_slowly_and_tools_speed_up() {
        let bare = break_ticks(BlockKind::Dirt, None).unwrap();
        let wood_shovel =
            break_ticks(BlockKind::Dirt, lite(tool_item(0, ToolKind::Shovel))).unwrap();
        let iron_shovel =
            break_ticks(BlockKind::Dirt, lite(tool_item(2, ToolKind::Shovel))).unwrap();
        // H = 0.5 → 15 ticks a mano (0.75 s).
        assert_eq!(bare, 15);
        assert_eq!(wood_shovel, (15u32 * 256).div_ceil(512));
        assert!(iron_shovel < wood_shovel);
        // Un pico no acelera la tierra.
        let pick = break_ticks(BlockKind::Dirt, lite(tool_item(2, ToolKind::Pickaxe))).unwrap();
        assert_eq!(pick, bare);
    }

    #[test]
    fn wrong_tier_breaks_but_yields_nothing() {
        let stone_pick = lite(tool_item(1, ToolKind::Pickaxe));
        let wood_pick = lite(tool_item(0, ToolKind::Pickaxe));
        assert!(drops_item(BlockKind::Ore, stone_pick));
        assert!(!drops_item(BlockKind::Ore, wood_pick));
        assert!(!drops_item(BlockKind::Ore, None));
        // Sigue rompiéndose, solo que más lento y sin botín.
        assert!(break_ticks(BlockKind::Ore, wood_pick).is_some());
        assert_eq!(
            drop_of(BlockKind::Ore),
            Some(ItemId::of_block(BlockKind::Ore))
        );
        assert_eq!(
            drop_of(BlockKind::CoalOre).map(|i| i.0),
            Some(super::super::items::COAL)
        );
    }

    #[test]
    fn obsidian_needs_steel_and_takes_long() {
        let steel = lite(tool_item(3, ToolKind::Pickaxe));
        let iron = lite(tool_item(2, ToolKind::Pickaxe));
        assert!(drops_item(BlockKind::Obsidian, steel));
        assert!(!drops_item(BlockKind::Obsidian, iron));
        // H = 30 → 900 ticks de trabajo base; con acero (8×) son ~113.
        let ticks = break_ticks(BlockKind::Obsidian, steel).unwrap();
        assert!((110..=115).contains(&ticks), "{ticks}");
    }

    #[test]
    fn fracture_progresses_through_ten_stages() {
        assert_eq!(fracture_stage(0, 60), 0);
        assert_eq!(fracture_stage(30, 60), 5);
        assert_eq!(fracture_stage(59, 60), 9);
        assert_eq!(fracture_stage(60, 60), 10);
        assert_eq!(fracture_stage(0, 0), FRACTURE_STAGES);
    }

    #[test]
    fn torch_breaks_instantly_and_fluids_are_not_diggable() {
        assert_eq!(break_ticks(BlockKind::Torch, None), Some(1));
        assert_eq!(break_ticks(BlockKind::Water, None), None);
        assert_eq!(break_ticks(BlockKind::Lava, None), None);
        assert_eq!(drop_of(BlockKind::Lava), None);
    }
}
