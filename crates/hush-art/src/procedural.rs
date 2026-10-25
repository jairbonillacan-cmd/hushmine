//! Texturas provisionales de 16×16 generadas por código.
//!
//! Fueron el arte por defecto hasta la llegada de los PNG reales de
//! `assets/textures/blocks`; hoy todos los bloques tienen PNG y este módulo
//! queda como generador determinista para pruebas y comparaciones. La capa
//! `i` sigue siendo el bloque `i + 1`.

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
            // Bloques del modo historia.
            BlockKind::Plank => {
                // Tablones horizontales con vetas.
                let grain = (y % 4 == 0 || (x + n % 3).is_multiple_of(9)) as i32;
                shade([168, 126, 74], fine / 3 - 6 - grain * 34)
            }
            BlockKind::PropBeam => {
                // Puntal: travesaño con marco reforzado.
                let frame = !(2..SIZE - 2).contains(&x) || !(2..SIZE - 2).contains(&y);
                let diagonal = (x + y).is_multiple_of(7);
                if frame || diagonal {
                    shade([120, 84, 48], -18)
                } else {
                    shade([160, 118, 70], fine / 3 - 6)
                }
            }
            BlockKind::Torch => {
                // Palo vertical con llama en la punta.
                if (6..=9).contains(&x) && (1..=4).contains(&y) {
                    [255, 196, 70]
                } else if (7..=8).contains(&x) && y >= 5 {
                    shade([110, 78, 44], -10)
                } else {
                    [24, 18, 12]
                }
            }
            BlockKind::Workbench => {
                // Tablero superior claro y frente con herramientas.
                let top = y < 5;
                if top {
                    shade([196, 156, 100], fine / 4 - 5)
                } else if (x + 2 * y).is_multiple_of(13) {
                    [90, 90, 96]
                } else {
                    shade([140, 102, 62], fine / 3 - 6)
                }
            }
            BlockKind::Furnace => {
                // Muro de piedra con boca encendida.
                let mouth = (4..=11).contains(&x) && (8..=13).contains(&y);
                if mouth {
                    if n.is_multiple_of(5) {
                        [255, 170, 40]
                    } else {
                        [170, 60, 18]
                    }
                } else {
                    rock(x, y, n, [110, 104, 100])
                }
            }
            BlockKind::Chest => {
                // Arcón: madera con tapa y cerradura de latón.
                let lid = y == 6;
                let lock = (7..=8).contains(&x) && (6..=9).contains(&y);
                if lock {
                    [220, 180, 70]
                } else if lid {
                    shade([150, 108, 58], -30)
                } else {
                    shade([158, 116, 64], fine / 3 - 6)
                }
            }
            BlockKind::Gravel => {
                // Cantos rodados sin orden.
                if hash(x / 2, y / 2, 51).is_multiple_of(3) {
                    shade([132, 124, 118], fine / 2 - 8)
                } else {
                    shade([104, 98, 94], fine / 2 - 8)
                }
            }
            BlockKind::Water => {
                // Agua: azul con oleaje suave.
                let wave = ((x + 2 * y + n % 4) % 8) as i32;
                shade([52, 96, 200], wave - 4)
            }
            BlockKind::Obsidian => {
                // Negro veteado con destellos violáceos.
                if (x * 3 + y * 5 + n % 3).is_multiple_of(17) {
                    [96, 60, 140]
                } else {
                    shade([24, 18, 34], fine / 4)
                }
            }
            BlockKind::CoalOre => speckled(x, y, n, [88, 84, 82], [38, 34, 34], 6),
            BlockKind::Log => {
                // Corteza con vetas verticales.
                let bark = !(2..SIZE - 2).contains(&x) || (x + n % 2).is_multiple_of(5);
                if bark {
                    shade([96, 66, 38], -14)
                } else {
                    shade([148, 104, 58], fine / 3 - 6)
                }
            }
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
