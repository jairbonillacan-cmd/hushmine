//! fBm: suma de octavas con frecuencia doble y amplitud mitad.
//!
//! Las frecuencias son potencias de dos (solo desplazamientos) y las
//! amplitudes viven en Q16. La suma se normaliza con la amplitud total para
//! que la salida quede en `[-ONE, ONE]` sea cual sea el número de octavas.

use crate::fixed::ONE;
use crate::gradient::{Noise2, Noise3};

/// Tope de octavas: a partir de la 17 la amplitud (`ONE >> 16`) es 1.
const MAX_OCTAVES: u32 = 17;

/// fBm 3D.
#[derive(Clone, Debug)]
pub struct Fbm3 {
    layers: Vec<Noise3>,
    /// Suma de amplitudes Q16 de las octavas.
    total: i64,
}

/// fBm 2D.
#[derive(Clone, Debug)]
pub struct Fbm2 {
    layers: Vec<Noise2>,
    total: i64,
}

impl Fbm3 {
    /// Crea el fBm con la semilla del mundo y la sal de la capa.
    ///
    /// `period_log2` es el log2 del período de la octava base en bloques;
    /// cada octava duplica la frecuencia. `octaves` se acota para que la
    /// última octava no tenga celdas menores de un bloque.
    #[must_use]
    pub fn new(seed: u64, salt: u64, period_log2: u32, octaves: u32) -> Self {
        let period_log2 = period_log2.min(46);
        let octaves = octaves.clamp(1, MAX_OCTAVES).min(period_log2 + 1);
        let layers = (0..octaves)
            .map(|o| Noise3::new(seed, salt.wrapping_add(u64::from(o)), period_log2 - o))
            .collect();
        let total = (0..octaves).map(|o| ONE >> o).sum();
        Self { layers, total }
    }

    /// Número de octavas tras acotar.
    #[must_use]
    pub fn octaves(&self) -> u32 {
        self.layers.len() as u32
    }

    /// Muestrea en coordenadas de bloque (enteras).
    #[must_use]
    pub fn sample_block(&self, x: i64, y: i64, z: i64) -> i64 {
        let mut acc = 0;
        let mut amp = ONE;
        for layer in &self.layers {
            acc += (layer.sample_block(x, y, z) * amp) >> 16;
            amp >>= 1;
        }
        (acc * ONE).div_euclid(self.total)
    }

    /// Muestrea en coordenadas Q16 (1 bloque = 65 536).
    #[must_use]
    pub fn sample_q16(&self, x: i64, y: i64, z: i64) -> i64 {
        let mut acc = 0;
        let mut amp = ONE;
        for layer in &self.layers {
            acc += (layer.sample_q16(x, y, z) * amp) >> 16;
            amp >>= 1;
        }
        (acc * ONE).div_euclid(self.total)
    }
}

impl Fbm2 {
    /// Crea el fBm como [`Fbm3::new`].
    #[must_use]
    pub fn new(seed: u64, salt: u64, period_log2: u32, octaves: u32) -> Self {
        let period_log2 = period_log2.min(46);
        let octaves = octaves.clamp(1, MAX_OCTAVES).min(period_log2 + 1);
        let layers = (0..octaves)
            .map(|o| Noise2::new(seed, salt.wrapping_add(u64::from(o)), period_log2 - o))
            .collect();
        let total = (0..octaves).map(|o| ONE >> o).sum();
        Self { layers, total }
    }

    /// Número de octavas tras acotar.
    #[must_use]
    pub fn octaves(&self) -> u32 {
        self.layers.len() as u32
    }

    /// Muestrea en coordenadas de bloque (enteras).
    #[must_use]
    pub fn sample_block(&self, x: i64, y: i64) -> i64 {
        let mut acc = 0;
        let mut amp = ONE;
        for layer in &self.layers {
            acc += (layer.sample_block(x, y) * amp) >> 16;
            amp >>= 1;
        }
        (acc * ONE).div_euclid(self.total)
    }

    /// Muestrea en coordenadas Q16 (1 bloque = 65 536).
    #[must_use]
    pub fn sample_q16(&self, x: i64, y: i64) -> i64 {
        let mut acc = 0;
        let mut amp = ONE;
        for layer in &self.layers {
            acc += (layer.sample_q16(x, y) * amp) >> 16;
            amp >>= 1;
        }
        (acc * ONE).div_euclid(self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn una_octava_es_el_ruido_base() {
        // Con una sola octava, amplitud ONE y total ONE: la suma es el
        // ruido de gradiente tal cual (misma sal en la capa 0).
        let fbm = Fbm3::new(42, 9, 6, 1);
        let noise = Noise3::new(42, 9, 6);
        for i in 0..16i64 {
            assert_eq!(
                fbm.sample_block(i * 7, -i, 3),
                noise.sample_block(i * 7, -i, 3)
            );
        }
        let fbm = Fbm2::new(42, 9, 5, 1);
        let noise = Noise2::new(42, 9, 5);
        for i in 0..16i64 {
            assert_eq!(fbm.sample_block(i * 7, -i), noise.sample_block(i * 7, -i));
        }
    }

    #[test]
    fn mas_octavas_no_salen_del_rango() {
        let fbm = Fbm3::new(1, 2, 6, 6);
        assert_eq!(fbm.octaves(), 6);
        let (mut min, mut max) = (i64::MAX, i64::MIN);
        for i in 0..48i64 {
            for j in 0..48i64 {
                let v = fbm.sample_block(i * 3 - 70, j, i * j);
                assert!((-ONE..=ONE).contains(&v), "fuera de rango: {v}");
                min = min.min(v);
                max = max.max(v);
            }
        }
        assert!(
            min < -ONE / 8 && max > ONE / 8,
            "rango comprimido: {min}..{max}"
        );
    }

    #[test]
    fn las_octavas_se_acotan_al_periodo() {
        // Período 1 bloque (period_log2 = 0): solo cabe una octava.
        assert_eq!(Fbm3::new(1, 1, 0, 4).octaves(), 1);
        // Período 4 bloques: hasta 3 octavas (la última, de 1 bloque).
        assert_eq!(Fbm3::new(1, 1, 2, 9).octaves(), 3);
        // Pedir cero octavas da al menos una.
        assert_eq!(Fbm3::new(1, 1, 6, 0).octaves(), 1);
    }

    #[test]
    fn es_continuo_entre_bloques_vecinos() {
        let fbm = Fbm3::new(7, 1, 6, 4);
        let mut max_delta = 0;
        for i in 0..512i64 {
            let a = fbm.sample_block(i, 17, -23);
            let b = fbm.sample_block(i + 1, 17, -23);
            max_delta = max_delta.max((a - b).abs());
        }
        assert!(
            max_delta <= 6000,
            "salto de {max_delta} entre bloques vecinos"
        );
    }

    #[test]
    fn determinista_y_sensible_a_semilla_y_sal() {
        let a = Fbm3::new(42, 1, 6, 4);
        let b = Fbm3::new(42, 1, 6, 4);
        let other_seed = Fbm3::new(43, 1, 6, 4);
        let other_salt = Fbm3::new(42, 2, 6, 4);
        let mut same = 0;
        let mut seed_diff = 0;
        let mut salt_diff = 0;
        for i in 0..64i64 {
            for j in 0..64i64 {
                let (x, y) = (i * 5, j * 3 - 90);
                let v = a.sample_block(x, y, x - y);
                if v == b.sample_block(x, y, x - y) {
                    same += 1;
                }
                if v != other_seed.sample_block(x, y, x - y) {
                    seed_diff += 1;
                }
                if v != other_salt.sample_block(x, y, x - y) {
                    salt_diff += 1;
                }
            }
        }
        assert_eq!(same, 64 * 64, "no es determinista");
        assert!(
            seed_diff > 64 * 64 * 9 / 10,
            "la semilla casi no cambia nada"
        );
        assert!(salt_diff > 64 * 64 * 9 / 10, "la sal casi no cambia nada");
    }

    /// Media, varianza y histograma de 16 cubos de una muestra.
    fn stats(values: &[i64]) -> (f64, f64, [u32; 16]) {
        let n = f64::from(u32::try_from(values.len()).unwrap());
        let mean = values.iter().map(|&v| v as f64).sum::<f64>() / n;
        let variance = values
            .iter()
            .map(|&v| (v as f64 - mean) * (v as f64 - mean))
            .sum::<f64>()
            / n;
        let mut histogram = [0u32; 16];
        for &v in values {
            let bucket = (u32::try_from(v + ONE).unwrap() * 16 / (2 * ONE as u32 + 1)) as usize;
            histogram[bucket.min(15)] += 1;
        }
        (mean, variance, histogram)
    }

    /// INV-W03: en ±7 499 999 la media, la varianza y el histograma del
    /// ruido son estadísticamente iguales a los del origen.
    ///
    /// Las regiones cubren bastantes celdas del período base (12 × 12 en
    /// 2D; 6 × 6 × 6 en 3D con paso 2) para que la comparación no dependa
    /// de las gradientes de unas pocas celdas. Los umbrales dejan margen
    /// para el ruido de muestreo pero detectan cualquier pérdida de
    /// precisión lejos del origen.
    #[test]
    fn inv_w03_estadisticas_en_7_499_999() {
        let edge = 7_499_999i64;
        let fbm2 = Fbm2::new(42, 0x5A17, 4, 5);
        let mut origin = Vec::with_capacity(192 * 192);
        let mut far = Vec::with_capacity(192 * 192);
        for y in 0..192i64 {
            for x in 0..192i64 {
                origin.push(fbm2.sample_block(x, y));
                far.push(fbm2.sample_block(edge + x, -edge + y));
            }
        }
        let (mean_a, var_a, hist_a) = stats(&origin);
        let (mean_b, var_b, hist_b) = stats(&far);
        assert!(
            (mean_a - mean_b).abs() < 512.0,
            "medias distintas: origen {mean_a}, lejos {mean_b}"
        );
        assert!(
            (var_a - var_b).abs() / var_a < 0.15,
            "varianzas distintas: origen {var_a}, lejos {var_b}"
        );
        for (bucket, (a, b)) in hist_a.iter().zip(&hist_b).enumerate() {
            let tolerance = 512.max(a.max(b) / 3);
            assert!(
                a.abs_diff(*b) <= tolerance,
                "histograma {bucket}: origen {a}, lejos {b}"
            );
        }

        let fbm3 = Fbm3::new(42, 0x5A17, 4, 4);
        let mut origin = Vec::new();
        let mut far = Vec::new();
        for z in (0..96i64).step_by(2) {
            for y in (0..96i64).step_by(2) {
                for x in (0..96i64).step_by(2) {
                    origin.push(fbm3.sample_block(x, y, z));
                    far.push(fbm3.sample_block(-edge + x, edge + y, edge + z));
                }
            }
        }
        let (mean_a, var_a, hist_a) = stats(&origin);
        let (mean_b, var_b, hist_b) = stats(&far);
        assert!(
            (mean_a - mean_b).abs() < 1024.0,
            "medias 3D distintas: origen {mean_a}, lejos {mean_b}"
        );
        assert!(
            (var_a - var_b).abs() / var_a < 0.15,
            "varianzas 3D distintas: origen {var_a}, lejos {var_b}"
        );
        for (bucket, (a, b)) in hist_a.iter().zip(&hist_b).enumerate() {
            let tolerance = 2048.max(a.max(b) / 3);
            assert!(
                a.abs_diff(*b) <= tolerance,
                "histograma 3D {bucket}: origen {a}, lejos {b}"
            );
        }
    }

    #[test]
    fn valores_de_referencia() {
        let fbm3 = Fbm3::new(42, 1, 6, 4);
        let fbm2 = Fbm2::new(42, 1, 6, 4);
        // Rellenados con la salida real; cualquier cambio accidental de las
        // octavas o de la normalización los rompe.
        assert_eq!(fbm3.sample_block(100, -35, 7), 3139);
        assert_eq!(fbm2.sample_block(100, -35), -572);
    }
}
