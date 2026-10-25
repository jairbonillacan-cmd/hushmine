//! Identificadores de bloque del terreno generado.
//!
//! Son los ids de la Convención A (`hush_sim::BlockKind`, reservados
//! `0..=17` según el ADR 0002) duplicados como constantes: el generador no
//! debe depender de las reglas del juego, y para el motor un bloque es un
//! `BlockId`. Si la Convención A los renumbra, estas constantes siguen el
//! cambio (lo vigila la prueba de `tests/inv_w01_determinismo.rs` sobre los
//! valores esperados del terreno).

use grisu_voxel::BlockId;

/// Tierra, la capa superficial (`hush_sim::BlockKind::Dirt`).
pub const DIRT: BlockId = 1;
/// Roca, el grueso del subsuelo (`BlockKind::Rock`).
pub const ROCK: BlockId = 2;
/// Roca dura, la profundidad (`BlockKind::HardRock`).
pub const HARD_ROCK: BlockId = 3;
/// Mena (`BlockKind::Ore`).
pub const ORE: BlockId = 4;
/// Oro (`BlockKind::Gold`).
pub const GOLD: BlockId = 5;
/// Diamante (`BlockKind::Diamond`).
pub const DIAMOND: BlockId = 6;
/// Lava de las bolsas del manto (`id` 7 de la Convención A). Siempre se
/// genera rodeada de roca sólida (`INV-W05`).
pub const LAVA: BlockId = 7;
