//! Luz de bloque en color RGB: 4 bits por canal, más 4 de cielo.
//!
//! Según `docs/graficos/04-shaders.md` §3, cada vóxel guarda luz de bloque
//! RGB (4 bits por canal) + luz de cielo (4 bits): 16 bits por bloque. Esta
//! tabla de emisión en color es la que usará el BFS de luz en color del hito
//! G0; **todavía no está conectada**: `grisu_voxel::LightMap` solo propaga un
//! nivel escalar y el cliente sigue usando [`crate::emission_table`], así que
//! en el juego la luz aún no tiene color.

use hush_sim::BlockKind;

/// Luz que emite un bloque: un canal de 4 bits por color (0-15).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct RgbLight {
    /// Canal rojo (0-15).
    pub r: u8,
    /// Canal verde (0-15).
    pub g: u8,
    /// Canal azul (0-15).
    pub b: u8,
}

impl RgbLight {
    /// Empaqueta el RGB y la luz de cielo en los 16 bits de un vóxel:
    /// cielo en el nibble alto y r, g, b en los tres nibbles bajos.
    pub const fn pack_u16(self, sky: u8) -> u16 {
        ((sky & 0xF) as u16) << 12
            | ((self.r & 0xF) as u16) << 8
            | ((self.g & 0xF) as u16) << 4
            | (self.b & 0xF) as u16
    }

    /// Inversa exacta de [`Self::pack_u16`]: devuelve la luz y la luz de cielo.
    pub const fn unpack_u16(packed: u16) -> (Self, u8) {
        (
            Self {
                r: (packed >> 8) as u8 & 0xF,
                g: (packed >> 4) as u8 & 0xF,
                b: packed as u8 & 0xF,
            },
            (packed >> 12) as u8,
        )
    }

    /// El canal más brillante; en la lava coincide con el valor escalar de
    /// [`crate::emission_table`].
    pub const fn max_channel(self) -> u8 {
        let rg = if self.r > self.g { self.r } else { self.g };
        if rg > self.b { rg } else { self.b }
    }
}

/// Color que emite cada bloque (4 bits por canal), indexado por su
/// identificador: la lava da naranja cálido, la baliza ámbar de lámpara de
/// sodio, el eléctrico una chispa azulada tenue y el campo de fuerza un
/// resplandor tenue. El resto no emite.
pub fn emission_rgb_table() -> [RgbLight; 256] {
    let mut table = [RgbLight::default(); 256];
    table[usize::from(BlockKind::Lava.id())] = RgbLight { r: 13, g: 6, b: 1 };
    table[usize::from(BlockKind::ForceField.id())] = RgbLight { r: 4, g: 7, b: 10 };
    table[usize::from(BlockKind::Electric.id())] = RgbLight { r: 3, g: 6, b: 9 };
    table[usize::from(BlockKind::Beacon.id())] = RgbLight { r: 14, g: 11, b: 4 };
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empaquetar y desempaquetar son inversas exactas en las 65536
    /// combinaciones de 4 bits.
    #[test]
    fn pack_and_unpack_are_exact_inverses() {
        for sky in 0..=15u8 {
            for r in 0..=15u8 {
                for g in 0..=15u8 {
                    for b in 0..=15u8 {
                        let light = RgbLight { r, g, b };
                        let (unpacked, unpacked_sky) = RgbLight::unpack_u16(light.pack_u16(sky));
                        assert_eq!(unpacked, light, "r={r} g={g} b={b}");
                        assert_eq!(unpacked_sky, sky);
                    }
                }
            }
        }
    }

    /// Los canales se recortan a 4 bits al empaquetar y no contaminan a los
    /// nibbles vecinos.
    #[test]
    fn packing_masks_each_channel_to_four_bits() {
        let light = RgbLight {
            r: 255,
            g: 16,
            b: 240,
        };
        let (unpacked, sky) = RgbLight::unpack_u16(light.pack_u16(255));
        assert_eq!((unpacked.r, unpacked.g, unpacked.b), (15, 0, 0));
        assert_eq!(sky, 15);
    }

    /// La lava coincide con la tabla escalar en su canal rojo (13).
    #[test]
    fn lava_red_channel_matches_scalar_table() {
        let id = usize::from(BlockKind::Lava.id());
        let lava = emission_rgb_table()[id];
        assert_eq!(lava.r, crate::emission_table()[id]);
        assert_eq!(lava, RgbLight { r: 13, g: 6, b: 1 });
    }

    /// Solo los cuatro bloques emisores tienen luz; el resto es negro.
    #[test]
    fn only_the_four_emissive_blocks_glow() {
        let table = emission_rgb_table();
        for block in BlockKind::ALL {
            let expected = match block {
                BlockKind::Lava => RgbLight { r: 13, g: 6, b: 1 },
                BlockKind::ForceField => RgbLight { r: 4, g: 7, b: 10 },
                BlockKind::Electric => RgbLight { r: 3, g: 6, b: 9 },
                BlockKind::Beacon => RgbLight { r: 14, g: 11, b: 4 },
                _ => RgbLight::default(),
            };
            assert_eq!(
                table[usize::from(block.id())],
                expected,
                "emisión inesperada para {block:?}"
            );
        }
    }

    /// `max_channel` devuelve el canal más brillante.
    #[test]
    fn max_channel_tracks_the_brightest_channel() {
        assert_eq!(RgbLight { r: 13, g: 6, b: 1 }.max_channel(), 13);
        assert_eq!(RgbLight { r: 14, g: 11, b: 4 }.max_channel(), 14);
        assert_eq!(RgbLight { r: 1, g: 9, b: 9 }.max_channel(), 9);
        assert_eq!(RgbLight::default().max_channel(), 0);
    }
}
