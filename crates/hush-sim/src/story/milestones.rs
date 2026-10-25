//! Progresión narrativa: árbol de hitos y clímax de expedición.
//!
//! Los nueve hitos del documento en cadena lineal. El clímax del Pozo
//! Ancestral exige sellar 4 válvulas de gas y reparar 4 engranajes antes de
//! activar el motor; la activación detona un asedio de 5 minutos y la
//! victoria llega al volver a la superficie.

use serde::{Deserialize, Serialize};

use super::inventory::Inventory;
use super::items::{ItemId, ToolKind, tool_item};
use super::strata::Bands;
use crate::BlockKind;

/// Asedio del clímax: 5 minutos a 20 Hz.
pub const SIEGE_TICKS: u32 = 6_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Milestone {
    /// El Despertar: madera fósil y herramientas primordiales.
    Awakening = 1,
    /// Edad de Piedra: pico de caliza y recolección de carbón.
    StoneAge = 2,
    /// Luz y Hogar: antorchas estables y campamento base.
    LightHome = 3,
    /// Metalurgia: horno de fundición y lingotes de hierro.
    Metallurgy = 4,
    /// Grieta Oscura: descenso bajo la cota del estrato abisal.
    DarkRift = 5,
    /// Acero Templado: herramientas de acero.
    SteelTempering = 6,
    /// Manto de Fuego: lecho basáltico y diamantes.
    FireMantle = 7,
    /// Pozo Ancestral: válvulas selladas y ascensor reparado.
    AncestralWell = 8,
    /// Ascenso Final: defensa de la maquinaria y escape.
    FinalAscent = 9,
}

impl Milestone {
    pub const ALL: [Self; 9] = [
        Self::Awakening,
        Self::StoneAge,
        Self::LightHome,
        Self::Metallurgy,
        Self::DarkRift,
        Self::SteelTempering,
        Self::FireMantle,
        Self::AncestralWell,
        Self::FinalAscent,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Awakening => "EL DESPERTAR",
            Self::StoneAge => "EDAD DE PIEDRA",
            Self::LightHome => "LUZ Y HOGAR",
            Self::Metallurgy => "METALURGIA",
            Self::DarkRift => "GRIETA OSCURA",
            Self::SteelTempering => "ACERO TEMPLADO",
            Self::FireMantle => "MANTO DE FUEGO",
            Self::AncestralWell => "POZO ANCESTRAL",
            Self::FinalAscent => "ASCENSO FINAL",
        }
    }

    pub fn goal(self) -> &'static str {
        match self {
            Self::Awakening => "FABRICA HERRAMIENTAS DE MADERA",
            Self::StoneAge => "FABRICA UN PICO DE PIEDRA Y MINA CARBON",
            Self::LightHome => "COLOCA 8 ANTORCHAS DE CAMPAMENTO",
            Self::Metallurgy => "FUNDE UN LINGOTE DE HIERRO",
            Self::DarkRift => "DESCIENDE AL ESTRATO ABISAL",
            Self::SteelTempering => "FORJA UNA HERRAMIENTA DE ACERO",
            Self::FireMantle => "OBTEN UN DIAMANTE DEL MANTO",
            Self::AncestralWell => "SELLA EL POZO Y ACTIVA EL ASCENSOR",
            Self::FinalAscent => "SOBREVIVE EL ASEDIO Y ASCIENDE",
        }
    }
}

/// Progreso del jugador por el árbol de hitos y estado del clímax.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Milestones {
    /// Máximo hito alcanzado (0 = ninguno).
    pub reached: u8,
    /// Antorchas colocadas (hito 3).
    pub torches_placed: u32,
    /// Válvulas de gas selladas (hito 8).
    pub valves_sealed: u8,
    /// Engranajes reparados (hito 8).
    pub gears_repaired: u8,
    /// Motor del ascensor activado: empieza el asedio.
    pub siege_started: bool,
    /// Ticks restantes de asedio.
    pub siege_ticks_left: u32,
    /// Victoria: asedio superado y jugador en la superficie.
    pub victory: bool,
}

/// Señales que la expedición reporta cada tick para evaluar hitos.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProgressSignals {
    pub has_iron_ingot: bool,
    pub has_steel_tool: bool,
    pub has_diamond: bool,
    pub player_y: i32,
    pub on_surface: bool,
}

impl Milestones {
    /// Evalúa el siguiente hito de la cadena con las señales actuales.
    /// Devuelve el hito recién alcanzado.
    pub fn check(
        &mut self,
        signals: ProgressSignals,
        inventory: &Inventory,
        bands: &Bands,
    ) -> Option<Milestone> {
        let next = Milestone::ALL.get(usize::from(self.reached)).copied()?;
        let done = match next {
            Milestone::Awakening => {
                inventory
                    .hotbar
                    .iter()
                    .chain(&inventory.backpack)
                    .flatten()
                    .any(|s| s.id.is_tool() && s.id.tool_tier() == Some(0))
                    || inventory.count_of(ItemId::of_block(BlockKind::Plank)) > 0
            }
            Milestone::StoneAge => {
                let stone_pick = tool_item(1, ToolKind::Pickaxe);
                inventory.count_of(stone_pick) > 0
                    && inventory.count_of(ItemId(super::items::COAL)) > 0
            }
            Milestone::LightHome => self.torches_placed >= 8,
            Milestone::Metallurgy => signals.has_iron_ingot,
            Milestone::DarkRift => signals.player_y < bands.abyssal,
            Milestone::SteelTempering => signals.has_steel_tool,
            Milestone::FireMantle => signals.has_diamond && signals.player_y < bands.basaltic,
            // El 8 se alcanza al activar el ascensor (lo marca la
            // expedición al completar válvulas y engranajes).
            Milestone::AncestralWell => self.siege_started,
            // La victoria la marca la expedición al volver a la superficie.
            Milestone::FinalAscent => false,
        };
        if done {
            self.reached = next as u8;
            Some(next)
        } else {
            None
        }
    }

    /// ¿Están las cuatro válvulas selladas y los cuatro engranajes listos?
    pub fn repairs_complete(&self) -> bool {
        self.valves_sealed >= 4 && self.gears_repaired >= 4
    }

    /// Activa el motor: arranca el asedio.
    pub fn activate_elevator(&mut self) -> bool {
        if !self.repairs_complete() || self.siege_started {
            return false;
        }
        self.siege_started = true;
        self.siege_ticks_left = SIEGE_TICKS;
        true
    }

    /// Avanza el asedio un tick. Devuelve `true` al terminarlo con vida.
    pub fn step_siege(&mut self, alive: bool, on_surface: bool) -> Option<bool> {
        if !self.siege_started || self.victory {
            return None;
        }
        if !alive {
            return Some(false);
        }
        if self.siege_ticks_left > 0 {
            self.siege_ticks_left -= 1;
            return None;
        }
        // Asedio superado: queda volver a la superficie.
        if on_surface {
            self.victory = true;
            self.reached = Milestone::FinalAscent as u8;
            return Some(true);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::items::{COAL, PLANK_ITEM};
    use super::*;
    use crate::story::inventory::ItemStack;

    #[test]
    fn chain_requires_each_milestone_in_order() {
        let mut m = Milestones::default();
        let mut inv = Inventory::new();
        let bands = Bands::new(96, 20);
        let signals = ProgressSignals::default();
        assert_eq!(m.check(signals, &inv, &bands), None);

        inv.add(ItemStack::new(ItemId(PLANK_ITEM), 4));
        assert_eq!(m.check(signals, &inv, &bands), Some(Milestone::Awakening));
        // Con madera ya no vuelve a dar el hito 1.
        assert_eq!(m.check(signals, &inv, &bands), None);
    }

    #[test]
    fn stone_age_needs_pick_and_coal() {
        let mut m = Milestones {
            reached: 1,
            ..Milestones::default()
        };
        let mut inv = Inventory::new();
        let bands = Bands::new(96, 20);
        inv.add(ItemStack::tool(tool_item(1, ToolKind::Pickaxe), 59));
        assert_eq!(m.check(ProgressSignals::default(), &inv, &bands), None);
        inv.add(ItemStack::new(ItemId(COAL), 3));
        assert_eq!(
            m.check(ProgressSignals::default(), &inv, &bands),
            Some(Milestone::StoneAge)
        );
    }

    #[test]
    fn climax_needs_full_repairs_then_siege_then_surface() {
        let mut m = Milestones {
            reached: 7,
            ..Milestones::default()
        };
        assert!(!m.activate_elevator());
        m.valves_sealed = 4;
        assert!(!m.activate_elevator());
        m.gears_repaired = 4;
        assert!(m.activate_elevator());
        assert!(m.siege_started);
        // Durante el asedio no hay victoria.
        assert_eq!(m.step_siege(true, true), None);
        // Muerte durante el asedio: derrota.
        m.siege_ticks_left = SIEGE_TICKS;
        assert_eq!(m.step_siege(false, false), Some(false));
        // Supervivencia pero bajo tierra: nada todavía.
        m.siege_ticks_left = 0;
        assert_eq!(m.step_siege(true, false), None);
        // Al pisar superficie: victoria y hito 9.
        assert_eq!(m.step_siege(true, true), Some(true));
        assert!(m.victory);
        assert_eq!(m.reached, 9);
    }

    #[test]
    fn labels_and_goals_are_set_for_every_milestone() {
        for m in Milestone::ALL {
            assert!(!m.label().is_empty());
            assert!(!m.goal().is_empty());
        }
    }

    #[test]
    fn torch_and_ingot_signals_advance_the_tree() {
        let mut m = Milestones {
            reached: 2,
            torches_placed: 8,
            ..Milestones::default()
        };
        let bands = Bands::new(96, 20);
        assert_eq!(
            m.check(ProgressSignals::default(), &Inventory::new(), &bands),
            Some(Milestone::LightHome)
        );
        m.reached = 3;
        let signals = ProgressSignals {
            has_iron_ingot: true,
            ..ProgressSignals::default()
        };
        assert_eq!(
            m.check(signals, &Inventory::new(), &bands),
            Some(Milestone::Metallurgy)
        );
        m.reached = 4;
        let signals = ProgressSignals {
            player_y: bands.abyssal - 1,
            ..ProgressSignals::default()
        };
        assert_eq!(
            m.check(signals, &Inventory::new(), &bands),
            Some(Milestone::DarkRift)
        );
    }

    #[test]
    fn steel_and_diamond_milestones() {
        let mut m = Milestones {
            reached: 5,
            ..Milestones::default()
        };
        let bands = Bands::new(96, 20);
        let steel = ProgressSignals {
            has_steel_tool: true,
            ..ProgressSignals::default()
        };
        assert_eq!(
            m.check(steel, &Inventory::new(), &bands),
            Some(Milestone::SteelTempering)
        );
        m.reached = 6;
        let mantle = ProgressSignals {
            has_diamond: true,
            player_y: bands.basaltic - 1,
            ..ProgressSignals::default()
        };
        assert_eq!(
            m.check(mantle, &Inventory::new(), &bands),
            Some(Milestone::FireMantle)
        );
    }
}
