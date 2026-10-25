//! Punto fijo Q16.16 compartido por todo el ruido.

/// La unidad Q16.16 (1.0). Todas las salidas de ruido están en `[-ONE, ONE]`.
pub const ONE: i64 = 65_536;

/// Interpolación lineal `a + (b − a) · t` con `t` en Q16.
///
/// Con `t` en `[0, ONE]` el resultado queda entre `a` y `b`, y con `t`
/// igual a `0` o `ONE` es exactamente `a` o `b`.
#[inline]
#[must_use]
pub const fn lerp(a: i64, b: i64, t: i64) -> i64 {
    a + (((b - a) * t) >> 16)
}

/// Curva de suavizado quíntica `6t⁵ − 15t⁴ + 10t³` en Q16.
///
/// Anula la primera y la segunda derivada en los extremos, así el ruido es
/// C² en las fronteras de celda y no se ven pliegues en el relieve. Requiere
/// `t` en `[0, ONE]`; los intermedios caben en `i64`.
#[inline]
#[must_use]
pub const fn quintic(t: i64) -> i64 {
    let t2 = (t * t) >> 16;
    let t3 = (t2 * t) >> 16;
    let t4 = (t2 * t2) >> 16;
    let t5 = (t3 * t2) >> 16;
    6 * t5 - 15 * t4 + 10 * t3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_es_exacta_en_los_extremos() {
        assert_eq!(lerp(-10, 20, 0), -10);
        assert_eq!(lerp(-10, 20, ONE), 20);
        assert_eq!(lerp(5, 9, ONE >> 1), 7);
    }

    #[test]
    fn quintic_pasa_por_cero_y_uno() {
        assert_eq!(quintic(0), 0);
        assert_eq!(quintic(ONE), ONE);
        // Monótona y por debajo de la diagonal en la primera mitad.
        let mut prev = 0;
        for i in 0..=64 {
            let t = i * 1024;
            let s = quintic(t);
            assert!(s >= prev, "quintica no monótona en {t}");
            if t < ONE / 2 {
                assert!(
                    s <= t,
                    "quintica ({s}) por encima de t ({t}) en la primera mitad"
                );
            }
            prev = s;
        }
    }
}
