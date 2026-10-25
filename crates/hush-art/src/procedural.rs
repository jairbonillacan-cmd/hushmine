//! Texturas provisionales de 16×16 generadas por código.
//!
//! Fueron el arte por defecto hasta la llegada de los PNG reales de
//! `assets/textures/blocks`. Se conservan como generador determinista para
//! pruebas y comparaciones; la capa `i` sigue siendo el bloque `i + 1`.

use grisu_render::TextureData;
use hush_sim::BlockKind;

use crate::textured_blocks;

const SIZE: u32 = 16;

/// Texturas procedurales de todos los bloques salvo el aire, en orden de
/// identificador.
pub fn procedural_block_textures() -> Vec<TextureData> {
    textured_blocks()
        .iter()
        .map(|&b| procedural_block_texture(b))
        .collect()
}

/// Textura procedural de un bloque. El aire devuelve un patrón magenta de
/// error.
pub fn procedural_block_texture(block: BlockKind) -> TextureData {
    let seed = u32::from(block.id()) * 0x51_7CC1;
    paint(|x, y| {
        let n = hash(x, y, seed);
        let fine = (n % 32) as i32;
        match block {
            BlockKind::Air => {
                if (x / 4 + y / 4) % 2 == 0 {
                    [255, 0, 255]
                } else {
                    [0, 0, 0]
                }
            }
            BlockKind::Dirt => {
                let base = shade([104, 74, 50], fine - 16);
                if n.is_multiple_of(9) {
                    shade(base, -30)
                } else if n.is_multiple_of(23) {
                    [140, 118, 92]
                } else {
                    base
                }
            }
            BlockKind::Rock => rock(x, y, n, [96, 96, 102]),
            BlockKind::HardRock => {
                let base = rock(x, y, n, [58, 60, 72]);
                if (x * 2 + y * 3 + n % 3).is_multiple_of(13) {
                    [92, 96, 114]
                } else {
                    base
                }
            }
            BlockKind::Ore => speckled(x, y, n, [88, 84, 82], [196, 112, 54], 7),
            BlockKind::Gold => speckled(x, y, n, [84, 80, 74], [250, 204, 60], 6),
            BlockKind::Diamond => speckled(x, y, n, [44, 46, 58], [150, 240, 250], 9),
            BlockKind::Lava => {
                let wave = ((x * 3 + y * 5 + (n % 5)) % 16) as i32;
                let hot = (wave - 8).abs() < 2 || n.is_multiple_of(11);
                if hot {
                    [255, 214, 90]
                } else {
                    shade([226, 82, 26], fine / 2 - 8)
                }
            }
            BlockKind::TeamBlock => structure(x, y, n, [150, 110, 80]),
            BlockKind::ForceField => structure(x, y, n, [120, 200, 230]),
            BlockKind::Ladder => {
                if x == 3 || x == 12 || y % 4 == 1 {
                    shade([150, 112, 64], fine / 2 - 8)
                } else {
                    [40, 30, 24]
                }
            }
            BlockKind::JumpPad => structure(x, y, n, [90, 190, 90]),
            BlockKind::Electric => structure(x, y, n, [230, 220, 80]),
            BlockKind::Rail => structure(x, y, n, [130, 130, 140]),
            BlockKind::Beacon => structure(x, y, n, [240, 240, 220]),
            BlockKind::Explosive => structure(x, y, n, [200, 50, 40]),
            BlockKind::BucketStation => structure(x, y, n, [110, 90, 150]),
            BlockKind::BucketPost => structure(x, y, n, [160, 140, 190]),
        }
    })
}

/// Marcador provisional de construcción: marco oscuro y relleno de color.
fn structure(x: u32, y: u32, n: u32, color: [u8; 3]) -> [u8; 3] {
    let edge = x == 0 || y == 0 || x == SIZE - 1 || y == SIZE - 1;
    let cross = x == y || x + y == SIZE - 1;
    if edge || cross {
        shade(color, -70)
    } else {
        shade(color, (n % 16) as i32 - 8)
    }
}

/// Roca con grietas oscuras en diagonal.
fn rock(x: u32, y: u32, n: u32, base: [u8; 3]) -> [u8; 3] {
    let c = shade(base, (n % 24) as i32 - 12);
    if (x + 2 * y + n % 2).is_multiple_of(11) {
        shade(c, -34)
    } else {
        c
    }
}

/// Roca con pepitas de mineral en grupos de 2×2.
fn speckled(x: u32, y: u32, n: u32, base: [u8; 3], gem: [u8; 3], period: u32) -> [u8; 3] {
    let cell = hash(x / 2, y / 2, period * 977);
    if cell.is_multiple_of(period) {
        shade(gem, (n % 30) as i32 - 15)
    } else {
        rock(x, y, n, base)
    }
}

fn paint(color: impl Fn(u32, u32) -> [u8; 3]) -> TextureData {
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let [r, g, b] = color(x, y);
            rgba.extend_from_slice(&[r, g, b, 255]);
        }
    }
    TextureData {
        width: SIZE,
        height: SIZE,
        rgba,
    }
}

fn shade(c: [u8; 3], d: i32) -> [u8; 3] {
    c.map(|v| (i32::from(v) + d).clamp(0, 255) as u8)
}

/// Hash entero estable para patrones procedurales.
fn hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ seed;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn procedural_covers_every_solid_block_in_order() {
        let textures = procedural_block_textures();
        assert_eq!(textures.len(), textured_blocks().len());
        assert!(
            textures
                .iter()
                .all(|t| t.width == 16 && t.rgba.len() == 1024)
        );
    }

    #[test]
    fn procedural_is_deterministic() {
        assert_eq!(
            procedural_block_texture(BlockKind::Ore).rgba,
            procedural_block_texture(BlockKind::Ore).rgba
        );
    }
}
