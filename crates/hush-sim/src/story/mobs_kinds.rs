//! Las cuatro clases de enemigo y sus números: vida, velocidad y daño.

use super::*;

/// Tipología de enemigo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobKind {
    /// Humanoide ciego y nudoso que ataca cuerpo a cuerpo.
    TunnelBrawler,
    /// Criatura silenciosa que deflagra junto al jugador.
    DustStalker,
    /// Restos esqueléticos que disparan pedernal a distancia.
    FossilGunner,
    /// Insecto blindado que trepa muros.
    CreviceVermin,
}

impl MobKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::TunnelBrawler => "BREGADOR DEL TUNEL",
            Self::DustStalker => "ACECHADOR DEL POLVO",
            Self::FossilGunner => "ARTILLERO FOSIL",
            Self::CreviceVermin => "ALIMANA DE LA GRIETA",
        }
    }

    /// Puntos de vida.
    pub fn health(self) -> u8 {
        match self {
            Self::TunnelBrawler => 14,
            Self::DustStalker => 8,
            Self::FossilGunner => 10,
            Self::CreviceVermin => 4,
        }
    }

    /// Velocidad en bloques/tick (punto fijo).
    pub fn speed(self) -> i32 {
        let blocks_per_s = match self {
            Self::TunnelBrawler => 3,
            Self::DustStalker => 4,
            Self::FossilGunner => 2,
            Self::CreviceVermin => 3,
        };
        blocks_per_s * FIXED_ONE as i32 / STORY_TICK_RATE as i32
    }

    /// Daño de contacto por golpe.
    pub fn melee_damage(self) -> u8 {
        match self {
            Self::TunnelBrawler => 5,
            Self::DustStalker => 2,
            Self::FossilGunner => 2,
            Self::CreviceVermin => 1,
        }
    }
}
