//! Hash espacial determinista compartido con `docs/mapgen/geologia.md`.

/// Constante áurea de 64 bits de SplitMix64.
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// Un paso de la mezcla SplitMix64.
///
/// Es el finalizador de `docs/mapgen/geologia.md`: aritmética `wrapping`
/// sobre `u64`, idéntica en nativo y en wasm.
#[inline]
#[must_use]
pub const fn splitmix64_step(x: u64) -> u64 {
    let mut z = x.wrapping_add(GOLDEN);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hash espacial entero `hash3d(x, y, z, seed)` de `docs/mapgen/geologia.md`.
///
/// Las coordenadas son `i64` (valores extremos incluidos: todo es
/// `wrapping`) y la semilla ya lleva la sal de la capa mezclada (ver
/// [`layer_seed`).
#[inline]
#[must_use]
pub const fn hash3d(x: i64, y: i64, z: i64, seed: u64) -> u64 {
    let mut h = seed ^ (x as u64).wrapping_mul(GOLDEN);
    h ^= (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= (z as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    splitmix64_step(h)
}

/// Hash espacial 2D: la tercera coordenada es 0.
#[inline]
#[must_use]
pub const fn hash2d(x: i64, y: i64, seed: u64) -> u64 {
    hash3d(x, y, 0, seed)
}

/// Semilla de una capa de ruido.
///
/// Mezcla la semilla del mundo con la sal de la capa para que capas con
/// sales consecutivas (las octavas de un fBm, por ejemplo) no estén
/// correlacionadas.
#[inline]
#[must_use]
pub const fn layer_seed(seed: u64, salt: u64) -> u64 {
    splitmix64_step(seed ^ splitmix64_step(salt).wrapping_mul(GOLDEN))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valores de referencia calculados con la aritmética exacta de la
    /// especificación (módulo 2⁶⁴). Detectan cambios accidentales del hash,
    /// que rompería mundos guardados.
    #[test]
    fn hash3d_valores_de_referencia() {
        let cases = [
            ((0, 0, 0, 0), 16_294_208_416_658_607_535),
            ((0, 0, 0, 42), 13_679_457_532_755_275_413),
            ((1, 2, 3, 42), 17_934_152_632_992_133_557),
            ((-1, 0, 0, 42), 8_407_032_967_045_885_863),
            ((-468_750, -10, 468_749, 42), 4_051_584_609_894_277_639),
            (
                (7_499_999, -7_499_999, 7_499_999, 1_234_567_890_123_456_789),
                3_059_947_590_057_991_020,
            ),
            (
                (i64::MAX, i64::MIN, i64::MAX, 0xDEAD_BEEF),
                6_321_349_790_120_653_172,
            ),
        ];
        for ((x, y, z, seed), expected) in cases {
            assert_eq!(
                hash3d(x, y, z, seed),
                expected,
                "hash3d({x}, {y}, {z}, {seed})"
            );
        }
    }

    #[test]
    fn layer_seed_descorrelaciona_sales_consecutivas() {
        // Sales consecutivas dan semillas distintas y con la distancia de
        // Hamming de dos valores al azar (~32 bits); el XOR directo con la
        // sal daría semillas casi iguales.
        let seeds: Vec<u64> = (0..8u64).map(|salt| layer_seed(42, salt)).collect();
        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                let distance = (seeds[i] ^ seeds[j]).count_ones();
                assert!(
                    distance >= 20,
                    "sal {i} y {j} casi iguales: {distance} bits"
                );
            }
        }
        // Determinista y dependiente de la semilla del mundo.
        assert_eq!(seeds[1], layer_seed(42, 1));
        assert_ne!(seeds[1], layer_seed(43, 1));
    }

    #[test]
    fn hash2d_es_hash3d_con_z_cero() {
        assert_eq!(hash2d(17, -3, 99), hash3d(17, -3, 0, 99));
    }
}
