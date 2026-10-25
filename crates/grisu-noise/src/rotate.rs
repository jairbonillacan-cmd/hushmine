//! Rotación entera exacta por ternas pitagóricas.
//!
//! Muestrear el ruido directamente en las coordenadas del mundo delata la
//! retícula alineada con los ejes: las celdas del ruido de gradiente son
//! cubos alineados y el relieve termina notándolo. Rotar las coordenadas
//! antes de muestrear lo disimula (`docs/mundo/02-generacion.md` §1, fila
//! «Rotación entera»), pero una rotación con `f32`/`f64` no vale: pierde
//! precisión lejos del origen y no garantiza el mismo valor bit a bit en
//! todas las plataformas (INV-W03). La solución es una matriz ortogonal
//! **racional exacta**: numeradores enteros y un denominador fijo con
//! división euclídea, sin nada flotante.
//!
//! - [`rotate2_pythagorean`]: terna pitagórica 3-4-5 en el plano XZ.
//! - [`rotate3_orthogonal`]: matriz ortogonal 3×3 de denominador 3.
//! - [`SampleRotated2`] y [`SampleRotated3`]: envoltorios que rotan antes
//!   de muestrear un [`Noise2`]/[`Fbm2`] o un [`Noise3`]/[`Fbm3`].

use crate::fbm::{Fbm2, Fbm3};
use crate::gradient::{Noise2, Noise3};

/// Rotación 2D exacta con la terna pitagórica (3, 4, 5).
///
/// Devuelve `((3x − 4z) / 5, (4x + 3z) / 5)`. La matriz `[[3, −4], [4, 3]]`
/// tiene filas de norma² 25 y ortogonales entre sí, así que la rotación
/// preserva distancias sin deriva de escala. Los intermedios se calculan en
/// `i128` (`3x − 4z` alcanza ±7·2⁶³ y `i128` lo absorbe sin desbordar) y la
/// división con [`i128::div_euclid`] redondea de forma determinista hacia
/// −∞. Cuando `x ≡ 3z (mod 5)` la división es exacta y la norma al
/// cuadrado se conserva **exactamente** (por ejemplo `(3, 1) -> (1, 3)`);
/// en el resto de puntos el error de redondeo queda acotado a unidades.
///
/// Jamás entra en pánico, ni siquiera con `i64::MIN` o `i64::MAX`: la
/// conversión final `as i64` trunca en vez de desbordar, y ese
/// truncamiento solo es alcanzable muy fuera del rango del mundo (en
/// ±7 500 000 la rotación cabe de sobra en `i64`).
#[inline]
#[must_use]
pub const fn rotate2_pythagorean(x: i64, z: i64) -> (i64, i64) {
    let a = 3 * x as i128 - 4 * z as i128;
    let b = 4 * x as i128 + 3 * z as i128;
    (a.div_euclid(5) as i64, b.div_euclid(5) as i64)
}

/// Rotación 3D con matriz ortogonal racional exacta de denominador 3.
///
/// Devuelve `((2x + 2y − z) / 3, (2x − y + 2z) / 3, (−x + 2y + 2z) / 3)`.
/// Las tres filas `(2, 2, −1)`, `(2, −1, 2)` y `(−1, 2, 2)` tienen norma²
/// 9 = 3² y son ortogonales dos a dos (determinante −1): la rotación
/// preserva distancias al cuadrado sin deriva de escala y rompe la
/// alineación de la retícula con los tres ejes a la vez. Cuando
/// `x + y + z ≡ 0 (mod 3)` la división es exacta y la norma se conserva
/// **exactamente** (por ejemplo `(1, 1, 1) -> (1, 1, 1)` o
/// `(1, −1, 0) -> (0, 1, −1)`).
///
/// Mismas garantías que [`rotate2_pythagorean`]: intermedios en `i128`,
/// redondeo determinista hacia −∞ y jamás un pánico; el truncamiento final
/// `as i64` solo es alcanzable muy fuera del rango del mundo.
#[inline]
#[must_use]
pub const fn rotate3_orthogonal(x: i64, y: i64, z: i64) -> (i64, i64, i64) {
    // Ojo con la precedencia: `-x as i128` sería `(-x) as i128` y negaría
    // en `i64` (desborde con `i64::MIN`), así que la tercera fila se escribe
    // como resta en `i128` y no con menos unario.
    let a = 2 * x as i128 + 2 * y as i128 - z as i128;
    let b = 2 * x as i128 - y as i128 + 2 * z as i128;
    let c = 2 * y as i128 + 2 * z as i128 - x as i128;
    (
        a.div_euclid(3) as i64,
        b.div_euclid(3) as i64,
        c.div_euclid(3) as i64,
    )
}

/// Muestreo en Q16 de un ruido 2D (para poder envolverlo).
pub trait NoiseSample2 {
    /// Muestrea en coordenadas Q16 (1 bloque = 65 536).
    fn sample_q16(&self, x: i64, z: i64) -> i64;
}

/// Muestreo en Q16 de un ruido 3D.
pub trait NoiseSample3 {
    /// Muestrea en coordenadas Q16 (1 bloque = 65 536).
    fn sample_q16(&self, x: i64, y: i64, z: i64) -> i64;
}

impl NoiseSample2 for Noise2 {
    fn sample_q16(&self, x: i64, z: i64) -> i64 {
        Noise2::sample_q16(self, x, z)
    }
}

impl NoiseSample2 for Fbm2 {
    fn sample_q16(&self, x: i64, z: i64) -> i64 {
        Fbm2::sample_q16(self, x, z)
    }
}

impl NoiseSample3 for Noise3 {
    fn sample_q16(&self, x: i64, y: i64, z: i64) -> i64 {
        Noise3::sample_q16(self, x, y, z)
    }
}

impl NoiseSample3 for Fbm3 {
    fn sample_q16(&self, x: i64, y: i64, z: i64) -> i64 {
        Fbm3::sample_q16(self, x, y, z)
    }
}

/// Envoltorio que rota las coordenadas 2D antes de muestrear.
///
/// Disimula la retícula del ruido alineada con los ejes: el llamador pasa
/// las coordenadas del mundo y el envoltorio las gira con la terna 3-4-5
/// ([`rotate2_pythagorean`]) antes de pedir el valor al ruido interno.
#[derive(Clone, Debug)]
pub struct SampleRotated2<N> {
    inner: N,
}

impl<N: NoiseSample2> SampleRotated2<N> {
    /// Envuelve `inner`; a partir de ahora todo muestreo va rotado.
    #[must_use]
    pub const fn new(inner: N) -> Self {
        Self { inner }
    }

    /// El ruido envuelto, por si el llamador necesita consultar o reutilizarlo.
    #[must_use]
    pub const fn inner(&self) -> &N {
        &self.inner
    }

    /// Muestrea en coordenadas Q16 tras rotar con [`rotate2_pythagorean`].
    #[must_use]
    pub fn sample_q16(&self, x: i64, z: i64) -> i64 {
        let (rx, rz) = rotate2_pythagorean(x, z);
        self.inner.sample_q16(rx, rz)
    }

    /// Muestrea en coordenadas de bloque (enteras).
    #[must_use]
    pub fn sample_block(&self, x: i64, z: i64) -> i64 {
        self.sample_q16(x << 16, z << 16)
    }
}

/// Envoltorio que rota las coordenadas 3D antes de muestrear.
///
/// Igual que [`SampleRotated2`] pero con la matriz ortogonal de
/// denominador 3 ([`rotate3_orthogonal`]), que rompe la alineación de la
/// retícula en los tres ejes a la vez.
#[derive(Clone, Debug)]
pub struct SampleRotated3<N> {
    inner: N,
}

impl<N: NoiseSample3> SampleRotated3<N> {
    /// Envuelve `inner`; a partir de ahora todo muestreo va rotado.
    #[must_use]
    pub const fn new(inner: N) -> Self {
        Self { inner }
    }

    /// El ruido envuelto, por si el llamador necesita consultar o reutilizarlo.
    #[must_use]
    pub const fn inner(&self) -> &N {
        &self.inner
    }

    /// Muestrea en coordenadas Q16 tras rotar con [`rotate3_orthogonal`].
    #[must_use]
    pub fn sample_q16(&self, x: i64, y: i64, z: i64) -> i64 {
        let (rx, ry, rz) = rotate3_orthogonal(x, y, z);
        self.inner.sample_q16(rx, ry, rz)
    }

    /// Muestrea en coordenadas de bloque (enteras).
    #[must_use]
    pub fn sample_block(&self, x: i64, y: i64, z: i64) -> i64 {
        self.sample_q16(x << 16, y << 16, z << 16)
    }
}

#[cfg(test)]
#[path = "rotate_tests.rs"]
mod tests;
