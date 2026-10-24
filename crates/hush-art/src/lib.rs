//! Texturas provisionales de 16×16 para los bloques de HUSHMINE.
//!
//! Se generan por código (deterministas, sin archivos) hasta que llegue el
//! arte definitivo en el hito 6. La capa `i` corresponde al bloque `i + 1`.

use grisu_render::TextureData;
use hush_sim::Block;

const SIZE: u32 = 16;

/// Texturas de todos los bloques sólidos, en el orden de [`Block::SOLID`].
pub fn block_textures() -> Vec<TextureData> {
    Block::SOLID.iter().map(|&b| block_texture(b)).collect()
}

/// Luz que emite cada bloque (0-15), indexada por su valor.
pub fn emission_table() -> [u8; 256] {
    let mut table = [0; 256];
    table[usize::from(Block::Lava.id())] = 13;
    table
}

/// Textura de un bloque. El aire devuelve un patrón magenta de error.
pub fn block_texture(block: Block) -> TextureData {
    let seed = u32::from(block.id()) * 0x51_7CC1;
    paint(|x, y| {
        let n = hash(x, y, seed);
        let fine = (n % 32) as i32;
        match block {
            Block::Air => {
                if (x / 4 + y / 4) % 2 == 0 {
                    [255, 0, 255]
                } else {
                    [0, 0, 0]
                }
            }
            Block::Dirt => {
                let base = shade([104, 74, 50], fine - 16);
                if n.is_multiple_of(9) {
                    shade(base, -30)
                } else if n.is_multiple_of(23) {
                    [140, 118, 92]
                } else {
                    base
                }
            }
            Block::Rock => rock(x, y, n, [96, 96, 102]),
            Block::HardRock => {
                let base = rock(x, y, n, [58, 60, 72]);
                if (x * 2 + y * 3 + n % 3).is_multiple_of(13) {
                    [92, 96, 114]
                } else {
                    base
                }
            }
            Block::Ore => speckled(x, y, n, [88, 84, 82], [196, 112, 54], 7),
            Block::Gold => speckled(x, y, n, [84, 80, 74], [250, 204, 60], 6),
            Block::Diamond => speckled(x, y, n, [44, 46, 58], [150, 240, 250], 9),
            Block::Lava => {
                let wave = ((x * 3 + y * 5 + (n % 5)) % 16) as i32;
                let hot = (wave - 8).abs() < 2 || n.is_multiple_of(11);
                if hot {
                    [255, 214, 90]
                } else {
                    shade([226, 82, 26], fine / 2 - 8)
                }
            }
        }
    })
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

    fn mean(t: &TextureData) -> [u32; 3] {
        let mut sum = [0u32; 3];
        for p in t.rgba.chunks(4) {
            for i in 0..3 {
                sum[i] += u32::from(p[i]);
            }
        }
        sum.map(|s| s / (SIZE * SIZE))
    }

    #[test]
    fn one_texture_per_solid_block() {
        let textures = block_textures();
        assert_eq!(textures.len(), Block::SOLID.len());
        assert!(
            textures
                .iter()
                .all(|t| t.width == 16 && t.rgba.len() == 1024)
        );
    }

    #[test]
    fn blocks_are_distinguishable() {
        let means: Vec<_> = block_textures().iter().map(mean).collect();
        for i in 0..means.len() {
            for j in i + 1..means.len() {
                let d: u32 = (0..3).map(|k| means[i][k].abs_diff(means[j][k])).sum();
                assert!(
                    d > 12,
                    "{:?} y {:?} se parecen demasiado",
                    Block::SOLID[i],
                    Block::SOLID[j]
                );
            }
        }
        let gold = mean(&block_texture(Block::Gold));
        assert!(
            gold[0] > gold[2] + 20,
            "el oro debería verse amarillo: {gold:?}"
        );
    }

    #[test]
    fn textures_are_deterministic() {
        assert_eq!(
            block_texture(Block::Ore).rgba,
            block_texture(Block::Ore).rgba
        );
    }
}
