/// Píxeles RGBA8 (sRGB) de una textura.
#[derive(Clone, Debug)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl TextureData {
    /// Patrón de prueba de 16×16: roca gris con vetas de mineral naranja.
    /// Determinista (sin aleatoriedad del sistema), para que las capturas
    /// sean comparables entre ejecuciones.
    pub fn test_ore() -> Self {
        const SIZE: u32 = 16;
        let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let n = hash(x, y);
                let rock = 70 + (n % 28) as u8;
                let is_ore = (x * 3 + y * 5).is_multiple_of(11) || n.is_multiple_of(37);
                let is_edge = x == 0 || y == 0 || x == SIZE - 1 || y == SIZE - 1;
                let (r, g, b) = if is_ore {
                    (226, 128 + (n % 30) as u8, 48)
                } else if is_edge {
                    (rock / 2, rock / 2, rock / 2 + 4)
                } else {
                    (rock, rock, rock + 6)
                };
                rgba.extend_from_slice(&[r, g, b, 255]);
            }
        }
        Self {
            width: SIZE,
            height: SIZE,
            rgba,
        }
    }
}

/// Hash entero simple y estable para patrones procedurales.
fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ore_is_16x16_and_has_ore() {
        let t = TextureData::test_ore();
        assert_eq!((t.width, t.height), (16, 16));
        assert_eq!(t.rgba.len(), 16 * 16 * 4);
        let orange = t.rgba.chunks(4).filter(|p| p[0] > 200 && p[2] < 80).count();
        assert!(
            orange > 10,
            "debería haber píxeles de mineral, hay {orange}"
        );
    }

    #[test]
    fn test_ore_is_deterministic() {
        assert_eq!(TextureData::test_ore().rgba, TextureData::test_ore().rgba);
    }
}
