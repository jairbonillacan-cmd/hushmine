//! Ruido de gradiente (Perlin) 2D y 3D en punto fijo.
//!
//! La retícula tiene un período que es potencia de dos, así que pasar a la
//! celda y a la fracción solo cuesta desplazamientos. En cada esquina de la
//! celda un hash elige una gradiente entera y se hace el producto escalar
//! con el desplazamiento hasta esa esquina; el resultado se interpola con la
//! curva quíntica. La salida está normalizada a `[-ONE, ONE]`.

use crate::fixed::{ONE, lerp, quintic};
use crate::hash::{hash2d, hash3d, layer_seed};

/// Límite de `period_log2` para que los desplazamientos internos no se
/// salgan de `i64` (la retícula en Q16 usa `16 + period_log2` bits).
const MAX_PERIOD_LOG2: u32 = 46;

/// Doce gradientes enteras 3D: los puntos medios de las aristas del cubo.
const GRADIENTS_3D: [(i8, i8, i8); 12] = [
    (1, 1, 0),
    (-1, 1, 0),
    (1, -1, 0),
    (-1, -1, 0),
    (1, 0, 1),
    (-1, 0, 1),
    (1, 0, -1),
    (-1, 0, -1),
    (0, 1, 1),
    (0, -1, 1),
    (0, 1, -1),
    (0, -1, -1),
];

/// Ocho gradientes enteras 2D: diagonales y ejes.
const GRADIENTS_2D: [(i8, i8); 8] = [
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
];

/// Celda de la retícula y fracción dentro de ella, en Q16 `[0, ONE)`.
///
/// Con `shift` los bits del período de la retícula en Q16. El
/// desplazamiento aritmético y la máscara hacen bien los negativos.
#[inline]
const fn cell_and_fraction(v: i64, shift: u32) -> (i64, i64) {
    (v >> shift, (v & ((1i64 << shift) - 1)) >> (shift - 16))
}

/// Gradiente 3D de una esquina elegida por hash.
#[inline]
fn dot3(seed: u64, cx: i64, cy: i64, cz: i64, ox: i64, oy: i64, oz: i64) -> i64 {
    let (gx, gy, gz) = GRADIENTS_3D[(hash3d(cx, cy, cz, seed) % 12) as usize];
    i64::from(gx) * ox + i64::from(gy) * oy + i64::from(gz) * oz
}

/// Gradiente 2D de una esquina elegida por hash.
#[inline]
fn dot2(seed: u64, cx: i64, cy: i64, ox: i64, oy: i64) -> i64 {
    let (gx, gy) = GRADIENTS_2D[(hash2d(cx, cy, seed) % 8) as usize];
    i64::from(gx) * ox + i64::from(gy) * oy
}

/// Ruido de gradiente 3D.
#[derive(Clone, Copy, Debug)]
pub struct Noise3 {
    seed: u64,
    /// Bits del período de la retícula en Q16: `16 + period_log2`.
    shift: u32,
}

/// Ruido de gradiente 2D.
#[derive(Clone, Copy, Debug)]
pub struct Noise2 {
    seed: u64,
    shift: u32,
}

impl Noise3 {
    /// Crea el ruido con la semilla del mundo y la sal de la capa.
    ///
    /// `period_log2` es el log2 del período en bloques: 6 son celdas de
    /// 64 bloques. Se acota a 46.
    #[must_use]
    pub fn new(seed: u64, salt: u64, period_log2: u32) -> Self {
        Self {
            seed: layer_seed(seed, salt),
            shift: 16 + period_log2.min(MAX_PERIOD_LOG2),
        }
    }

    /// Muestrea en coordenadas de bloque (enteras).
    ///
    /// Válido para |coordenada| ≤ 2⁴⁷ bloques; más lejos el mundo no llega.
    #[must_use]
    pub fn sample_block(&self, x: i64, y: i64, z: i64) -> i64 {
        self.sample_q16(x << 16, y << 16, z << 16)
    }

    /// Muestrea en coordenadas Q16 (1 bloque = 65 536). Acepta cualquier
    /// `i64` sin desbordar: los intermedios quedan por debajo de 2³⁵.
    #[must_use]
    pub fn sample_q16(&self, x: i64, y: i64, z: i64) -> i64 {
        let (cx, tx) = cell_and_fraction(x, self.shift);
        let (cy, ty) = cell_and_fraction(y, self.shift);
        let (cz, tz) = cell_and_fraction(z, self.shift);
        let (sx, sy, sz) = (quintic(tx), quintic(ty), quintic(tz));
        let seed = self.seed;
        let n000 = dot3(seed, cx, cy, cz, tx, ty, tz);
        let n100 = dot3(seed, cx + 1, cy, cz, tx - ONE, ty, tz);
        let n010 = dot3(seed, cx, cy + 1, cz, tx, ty - ONE, tz);
        let n110 = dot3(seed, cx + 1, cy + 1, cz, tx - ONE, ty - ONE, tz);
        let n001 = dot3(seed, cx, cy, cz + 1, tx, ty, tz - ONE);
        let n101 = dot3(seed, cx + 1, cy, cz + 1, tx - ONE, ty, tz - ONE);
        let n011 = dot3(seed, cx, cy + 1, cz + 1, tx, ty - ONE, tz - ONE);
        let n111 = dot3(seed, cx + 1, cy + 1, cz + 1, tx - ONE, ty - ONE, tz - ONE);
        let x00 = lerp(n000, n100, sx);
        let x10 = lerp(n010, n110, sx);
        let x01 = lerp(n001, n101, sx);
        let x11 = lerp(n011, n111, sx);
        let y0 = lerp(x00, x10, sy);
        let y1 = lerp(x01, x11, sy);
        // El producto escalar llega a ±2·ONE (gradientes con dos componentes
        // no nulas); se divide entre 2 para quedar en [-ONE, ONE].
        lerp(y0, y1, sz) >> 1
    }
}

impl Noise2 {
    /// Crea el ruido como [`Noise3::new`].
    #[must_use]
    pub fn new(seed: u64, salt: u64, period_log2: u32) -> Self {
        Self {
            seed: layer_seed(seed, salt),
            shift: 16 + period_log2.min(MAX_PERIOD_LOG2),
        }
    }

    /// Muestrea en coordenadas de bloque (enteras).
    #[must_use]
    pub fn sample_block(&self, x: i64, y: i64) -> i64 {
        self.sample_q16(x << 16, y << 16)
    }

    /// Muestrea en coordenadas Q16 (1 bloque = 65 536) sin desbordar.
    #[must_use]
    pub fn sample_q16(&self, x: i64, y: i64) -> i64 {
        let (cx, tx) = cell_and_fraction(x, self.shift);
        let (cy, ty) = cell_and_fraction(y, self.shift);
        let (sx, sy) = (quintic(tx), quintic(ty));
        let seed = self.seed;
        let n00 = dot2(seed, cx, cy, tx, ty);
        let n10 = dot2(seed, cx + 1, cy, tx - ONE, ty);
        let n01 = dot2(seed, cx, cy + 1, tx, ty - ONE);
        let n11 = dot2(seed, cx + 1, cy + 1, tx - ONE, ty - ONE);
        // Misma normalización que en 3D.
        lerp(lerp(n00, n10, sx), lerp(n01, n11, sx), sy) >> 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_salida_queda_en_rango() {
        let n3 = Noise3::new(42, 7, 6);
        let n2 = Noise2::new(42, 7, 5);
        for i in 0..64i64 {
            for j in 0..64i64 {
                let v = n3.sample_block(i * 17 - 500, j * 13, i * j - 200);
                assert!((-ONE..=ONE).contains(&v), "3D fuera de rango: {v}");
                let v = n2.sample_block(i * 17 - 500, j * 13);
                assert!((-ONE..=ONE).contains(&v), "2D fuera de rango: {v}");
            }
        }
    }

    #[test]
    fn es_continuo_entre_bloques_vecinos() {
        // El ruido de gradiente con curva quíntica es C²: un salto grande
        // entre bloques contiguos delata un error de esquina o de celda.
        let n3 = Noise3::new(7, 1, 5);
        let n2 = Noise2::new(7, 2, 5);
        let mut max3 = 0;
        let mut max2 = 0;
        for i in 0..512i64 {
            // El recorrido cruza varias fronteras de celda (período 32).
            max3 = max3.max((n3.sample_block(i, 17, -23) - n3.sample_block(i + 1, 17, -23)).abs());
            max2 = max2.max((n2.sample_block(-31, i) - n2.sample_block(-31, i + 1)).abs());
        }
        assert!(max3 <= 8192, "salto 3D de {max3} entre bloques vecinos");
        assert!(max2 <= 8192, "salto 2D de {max2} entre bloques vecinos");
    }

    #[test]
    fn es_cero_en_las_esquinas_de_la_reticula() {
        // En una esquina la fracción es 0 y el producto escalar usa la
        // gradiente de esa esquina: el valor depende solo de ella.
        let n = Noise3::new(3, 3, 6);
        assert_eq!(n.sample_block(0, 0, 0), 0);
        assert_eq!(n.sample_block(64, 64, 64), 0);
        let n = Noise2::new(3, 3, 6);
        assert_eq!(n.sample_block(0, 0), 0);
        assert_eq!(n.sample_block(-64, 128), 0);
    }

    #[test]
    fn la_salida_cubre_bien_el_rango() {
        // Si el producto escalar o la normalización estuvieran mal
        // comprimidos, la salida no saldría de una franja estrecha.
        let n3 = Noise3::new(42, 0, 6);
        let (mut min, mut max) = (i64::MAX, i64::MIN);
        for i in 0..64i64 {
            for j in 0..64i64 {
                let v = n3.sample_block(i * 3, j * 5, i + j);
                min = min.min(v);
                max = max.max(v);
            }
        }
        assert!(min < -ONE / 4, "mínimo sospechosamente alto: {min}");
        assert!(max > ONE / 4, "máximo sospechosamente bajo: {max}");
    }

    #[test]
    fn coordenadas_extremas_no_desbordan() {
        let n3 = Noise3::new(1, 1, 6);
        let n2 = Noise2::new(1, 1, 6);
        let extremes = [i64::MIN, i64::MIN + 1, -1, 0, i64::MAX - 1, i64::MAX];
        for &e in &extremes {
            assert!((-ONE..=ONE).contains(&n3.sample_q16(e, e, e)));
            assert!((-ONE..=ONE).contains(&n2.sample_q16(e, e)));
        }
        // Bloques grandes pero muy lejos del límite de 2⁴⁷.
        let far = (1i64 << 40) + 123;
        assert!((-ONE..=ONE).contains(&n3.sample_block(far, -far, far)));
        assert!((-ONE..=ONE).contains(&n2.sample_block(-far, far)));
    }

    #[test]
    fn valores_de_referencia() {
        let n3 = Noise3::new(42, 7, 6);
        let n2 = Noise2::new(42, 7, 5);
        assert_eq!(n3.sample_block(0, 0, 0), 0);
        // Rellenados con la salida real; cualquier cambio accidental del
        // muestreo los rompe.
        assert_eq!(n3.sample_block(100, -35, 7), 760);
        assert_eq!(n2.sample_block(100, -35), -3195);
    }
}
