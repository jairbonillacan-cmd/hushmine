//! Hash espacial entero, la primitiva de todo el ruido del generador.
//!
//! Es el `hash3d` de `docs/mapgen/geologia.md` (mezcla SplitMix64): solo
//! aritmética `wrapping_*` sobre `u64`, coordenadas `i64` y una **sal** por
//! capa de ruido. Sin `f32` ni funciones trascendentes, el resultado es el
//! mismo byte a byte en cualquier plataforma y a cualquier distancia del
//! origen.

/// Un paso de la mezcla SplitMix64, con las constantes de
/// `docs/mapgen/geologia.md`.
#[inline]
pub const fn splitmix64_step(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hash espacial de una posición de bloque y una sal. La sal distingue las
/// capas de ruido (relieve, menas...) que comparten semilla.
#[inline]
pub const fn hash3d(x: i64, y: i64, z: i64, salt: u64) -> u64 {
    let mut h = salt ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= (z as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    splitmix64_step(h)
}

/// Hash espacial de una columna del terreno.
#[inline]
pub const fn hash2d(x: i64, z: i64, salt: u64) -> u64 {
    hash3d(x, 0, z, salt)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valores calculados con el prototipo de referencia de
    /// `tools/mapgen/rng.py`: si esto falla, el hash dejó de ser el de la
    /// geología documentada.
    #[test]
    fn matches_the_reference_prototype() {
        assert_eq!(splitmix64_step(0), 16_294_208_416_658_607_535);
        assert_eq!(hash3d(1, 2, 3, 42), 17_934_152_632_992_133_557);
        assert_eq!(hash3d(-1, -2, -3, 0), 4_379_842_046_401_702_724);
        assert_eq!(hash3d(0, 0, 0, u64::MAX), 16_490_336_266_968_443_936);
        assert_eq!(
            hash3d(7_499_999, -4096, 7_499_999, 42),
            12_951_730_528_942_364_248
        );
    }

    /// Cambiar cualquier entrada (o la sal) cambia el hash: sin colisiones
    /// triviales entre capas de ruido.
    #[test]
    fn every_input_and_salt_matters() {
        let base = hash3d(10, 20, 30, 1);
        assert_ne!(hash3d(11, 20, 30, 1), base);
        assert_ne!(hash3d(10, 21, 30, 1), base);
        assert_ne!(hash3d(10, 20, 31, 1), base);
        assert_ne!(hash3d(10, 20, 30, 2), base);
        assert_ne!(hash3d(-10, 20, 30, 1), base);
    }

    /// Los bits altos están mezclados: un dominio pequeño no concentra el
    /// resto en unos pocos valores.
    #[test]
    fn spreads_small_domains() {
        let mut seen = std::collections::HashSet::new();
        for x in -32..32 {
            for z in -32..32 {
                seen.insert(hash2d(i64::from(x), i64::from(z), 42) >> 56);
            }
        }
        // 4096 muestras sobre 256 cubos de bits altos: si la mezcla fuera
        // mala, habría decenas de cubos vacíos.
        assert!(seen.len() > 200, "{} cubos de 256", seen.len());
    }
}
