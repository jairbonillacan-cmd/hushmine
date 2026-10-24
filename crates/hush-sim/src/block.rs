use grisu_voxel::BlockId;
use serde::{Deserialize, Serialize};

/// Bloques de HUSHMINE. El valor numérico es el que guarda el terreno y
/// también el índice de su textura (`valor - 1`) en el cliente.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Block {
    Air = 0,
    Dirt = 1,
    Rock = 2,
    HardRock = 3,
    Ore = 4,
    Gold = 5,
    Diamond = 6,
    Lava = 7,
}

impl Block {
    /// Todos los bloques sólidos, en orden de valor.
    pub const SOLID: [Self; 7] = [
        Self::Dirt,
        Self::Rock,
        Self::HardRock,
        Self::Ore,
        Self::Gold,
        Self::Diamond,
        Self::Lava,
    ];

    pub fn id(self) -> BlockId {
        self as BlockId
    }

    pub fn from_id(id: BlockId) -> Option<Self> {
        match id {
            0 => Some(Self::Air),
            _ => Self::SOLID.get(usize::from(id) - 1).copied(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Air => "air",
            Self::Dirt => "dirt",
            Self::Rock => "rock",
            Self::HardRock => "hard_rock",
            Self::Ore => "ore",
            Self::Gold => "gold",
            Self::Diamond => "diamond",
            Self::Lava => "lava",
        }
    }

    /// ¿Se puede picar? La lava no.
    pub fn is_diggable(self) -> bool {
        !matches!(self, Self::Air | Self::Lava)
    }

    /// Ticks de pico necesarios para romperlo (a 30 Hz). Se usará cuando
    /// lleguen las clases; de momento solo informa.
    pub fn dig_ticks(self) -> u32 {
        match self {
            Self::Air | Self::Lava => 0,
            Self::Dirt => 6,
            Self::Rock | Self::Ore => 12,
            Self::Gold | Self::Diamond => 15,
            Self::HardRock => 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        assert_eq!(Block::from_id(0), Some(Block::Air));
        for block in Block::SOLID {
            assert_eq!(Block::from_id(block.id()), Some(block));
        }
        assert_eq!(Block::from_id(200), None);
    }

    #[test]
    fn solid_list_is_dense_from_one() {
        for (i, block) in Block::SOLID.iter().enumerate() {
            assert_eq!(usize::from(block.id()), i + 1);
        }
    }
}
