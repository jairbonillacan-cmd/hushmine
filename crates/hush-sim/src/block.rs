//! Tipos y propiedades estáticas de los bloques del mundo (Convención A).

use std::fmt;

use serde::{Deserialize, Serialize};

/// Error al intentar convertir un identificador numérico desconocido en [`BlockKind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidBlockId(pub u8);

impl fmt::Display for InvalidBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "identificador de bloque no válido: {}", self.0)
    }
}

impl std::error::Error for InvalidBlockId {}

/// Tipo de bloque del mundo de vóxeles (Convención A).
///
/// El equipo propietario (`Copper`/`Cobalt`) no forma parte del identificador
/// del bloque, sino de los metadatos asociados a su celda.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockKind {
    Air = 0,
    Dirt = 1,
    Rock = 2,
    HardRock = 3,
    Ore = 4,
    Gold = 5,
    Diamond = 6,
    Lava = 7,
    TeamBlock = 8,
    ForceField = 9,
    Ladder = 10,
    JumpPad = 11,
    Electric = 12,
    Rail = 13,
    Beacon = 14,
    Explosive = 15,
    BucketStation = 16,
    BucketPost = 17,
}

impl BlockKind {
    /// Número total de tipos de bloque definidos en la Convención A.
    pub const COUNT: usize = 18;

    /// Todos los tipos de bloque en orden de identificador (`0..=17`).
    pub const ALL: [Self; Self::COUNT] = [
        Self::Air,
        Self::Dirt,
        Self::Rock,
        Self::HardRock,
        Self::Ore,
        Self::Gold,
        Self::Diamond,
        Self::Lava,
        Self::TeamBlock,
        Self::ForceField,
        Self::Ladder,
        Self::JumpPad,
        Self::Electric,
        Self::Rail,
        Self::Beacon,
        Self::Explosive,
        Self::BucketStation,
        Self::BucketPost,
    ];

    /// Devuelve el identificador numérico estable (`0..=17`) del bloque.
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// Convierte un identificador numérico en su [`BlockKind`] correspondiente.
    pub const fn from_id(id: u8) -> Result<Self, InvalidBlockId> {
        match id {
            0 => Ok(Self::Air),
            1 => Ok(Self::Dirt),
            2 => Ok(Self::Rock),
            3 => Ok(Self::HardRock),
            4 => Ok(Self::Ore),
            5 => Ok(Self::Gold),
            6 => Ok(Self::Diamond),
            7 => Ok(Self::Lava),
            8 => Ok(Self::TeamBlock),
            9 => Ok(Self::ForceField),
            10 => Ok(Self::Ladder),
            11 => Ok(Self::JumpPad),
            12 => Ok(Self::Electric),
            13 => Ok(Self::Rail),
            14 => Ok(Self::Beacon),
            15 => Ok(Self::Explosive),
            16 => Ok(Self::BucketStation),
            17 => Ok(Self::BucketPost),
            _ => Err(InvalidBlockId(id)),
        }
    }

    /// Devuelve el identificador en `snake_case` usado en datos, texturas y código.
    // Se moverá a data/*.ron
    pub const fn name(self) -> &'static str {
        match self {
            Self::Air => "air",
            Self::Dirt => "dirt",
            Self::Rock => "rock",
            Self::HardRock => "hard_rock",
            Self::Ore => "ore",
            Self::Gold => "gold",
            Self::Diamond => "diamond",
            Self::Lava => "lava",
            Self::TeamBlock => "team_block",
            Self::ForceField => "force_field",
            Self::Ladder => "ladder",
            Self::JumpPad => "jump_pad",
            Self::Electric => "electric",
            Self::Rail => "rail",
            Self::Beacon => "beacon",
            Self::Explosive => "explosive",
            Self::BucketStation => "bucket_station",
            Self::BucketPost => "bucket_post",
        }
    }

    /// Indica si el bloque presenta colisión sólida (todos salvo `Air` y `Lava`).
    // Se moverá a data/*.ron
    pub const fn is_solid(self) -> bool {
        !matches!(self, Self::Air | Self::Lava)
    }

    /// Indica si el bloque forma parte natural del terreno generado (`1..=7`).
    // Se moverá a data/*.ron
    pub const fn is_natural(self) -> bool {
        matches!(
            self,
            Self::Dirt
                | Self::Rock
                | Self::HardRock
                | Self::Ore
                | Self::Gold
                | Self::Diamond
                | Self::Lava
        )
    }

    /// Indica si el bloque es una estructura construible por un equipo (`8..=17`).
    // Se moverá a data/*.ron
    pub const fn is_team_block(self) -> bool {
        matches!(
            self,
            Self::TeamBlock
                | Self::ForceField
                | Self::Ladder
                | Self::JumpPad
                | Self::Electric
                | Self::Rail
                | Self::Beacon
                | Self::Explosive
                | Self::BucketStation
                | Self::BucketPost
        )
    }

    /// Indica si el bloque es un recurso mineral de valor (`Ore`, `Gold` o `Diamond`).
    // Se moverá a data/*.ron
    pub const fn is_valuable(self) -> bool {
        matches!(self, Self::Ore | Self::Gold | Self::Diamond)
    }
}

impl From<BlockKind> for u8 {
    fn from(kind: BlockKind) -> Self {
        kind.id()
    }
}

impl TryFrom<u8> for BlockKind {
    type Error = InvalidBlockId;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        Self::from_id(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ids_round_trip() {
        assert_eq!(BlockKind::ALL.len(), BlockKind::COUNT);
        for (index, &expected_kind) in BlockKind::ALL.iter().enumerate() {
            let id = u8::try_from(index).unwrap();
            assert_eq!(expected_kind.id(), id);
            assert_eq!(u8::from(expected_kind), id);
            assert_eq!(BlockKind::from_id(id), Ok(expected_kind));
            assert_eq!(BlockKind::try_from(id), Ok(expected_kind));
        }
    }

    #[test]
    fn invalid_ids_return_error() {
        for id in 18..=u8::MAX {
            let err = BlockKind::try_from(id).unwrap_err();
            assert_eq!(err, InvalidBlockId(id));
            assert_eq!(BlockKind::from_id(id), Err(InvalidBlockId(id)));
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn names_match_convention_a() {
        let expected = [
            "air",
            "dirt",
            "rock",
            "hard_rock",
            "ore",
            "gold",
            "diamond",
            "lava",
            "team_block",
            "force_field",
            "ladder",
            "jump_pad",
            "electric",
            "rail",
            "beacon",
            "explosive",
            "bucket_station",
            "bucket_post",
        ];
        assert_eq!(BlockKind::ALL.len(), expected.len());
        for (kind, expected_name) in BlockKind::ALL.into_iter().zip(expected) {
            assert_eq!(kind.name(), expected_name);
        }
    }

    #[test]
    fn block_properties_are_consistent() {
        for kind in BlockKind::ALL {
            let id = kind.id();
            assert_eq!(kind.is_solid(), id != 0 && id != 7);
            assert_eq!(kind.is_natural(), (1..=7).contains(&id));
            assert_eq!(kind.is_team_block(), (8..=17).contains(&id));
            assert_eq!(kind.is_valuable(), (4..=6).contains(&id));

            // Partición estricta entre aire, bloques naturales y bloques de equipo.
            let categories = usize::from(kind == BlockKind::Air)
                + usize::from(kind.is_natural())
                + usize::from(kind.is_team_block());
            assert_eq!(categories, 1);

            if kind.is_valuable() {
                assert!(kind.is_natural());
                assert!(kind.is_solid());
            }
        }
    }
}
