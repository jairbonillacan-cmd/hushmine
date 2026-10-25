//! Ruido en cresta.

use crate::fixed::ONE;

/// Ruido en cresta: `65 536 − |ruido|`.
///
/// Convierte un ruido en `[-ONE, ONE]` en un campo de crestas: 0 donde el
/// ruido cruza por cero y `ONE` donde roza ±1. Con un umbral alto sobre la
/// cresta salen crestas de montaña y túneles largos y sinuosos.
#[inline]
#[must_use]
pub const fn ridge(n: i64) -> i64 {
    ONE - n.saturating_abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_cresta_invierte_el_valor_absoluto() {
        assert_eq!(ridge(0), ONE);
        assert_eq!(ridge(ONE), 0);
        assert_eq!(ridge(-ONE), 0);
        assert_eq!(ridge(12345), ONE - 12345);
        assert_eq!(ridge(-12345), ONE - 12345);
    }

    #[test]
    fn no_entra_en_panico_con_i64_minimo() {
        // Fuera del rango documentado no revienta: satura.
        assert_eq!(ridge(i64::MIN), ONE - i64::MAX);
    }
}
