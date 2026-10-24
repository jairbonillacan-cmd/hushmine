//! Aritmética determinista en punto fijo `Q16.16`.
//!
//! Representa números con signo usando 16 bits para la parte entera y 16 bits
//! para la parte fraccionaria sobre un `i32` (`1.0 == 65_536`).
//!
//! ## Política de redondeo
//! Todas las operaciones con escalado ([`Fixed::from_ratio`], [`Fixed::checked_mul`],
//! [`Fixed::saturating_mul`], [`Fixed::checked_div`] y [`Fixed::saturating_div`])
//! aplican **truncamiento hacia cero (redondeo hacia cero)** mediante división
//! entera en `i64` (`/`). Nunca se utiliza desplazamiento aritmético `>> 16`
//! sobre valores con signo, ya que redondearía hacia $-\infty$ en los negativos
//! y rompería la simetría entre signos.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Error en operaciones de punto fijo Q16.16.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixedError {
    /// Intento de dividir por cero.
    DivisionByZero,
    /// El valor no cabe en el rango representable de Q16.16 (`i32`).
    Overflow,
}

impl fmt::Display for FixedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivisionByZero => f.write_str("división por cero en punto fijo Q16.16"),
            Self::Overflow => f.write_str("desbordamiento en punto fijo Q16.16"),
        }
    }
}

impl std::error::Error for FixedError {}

/// Número en punto fijo determinista con formato `Q16.16` (16 bits enteros con
/// signo y 16 bits fraccionarios sobre un `i32`).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct Fixed(i32);

impl Fixed {
    /// Número de bits fraccionarios en `Q16.16`.
    pub const FRAC_BITS: u32 = 16;

    /// Factor de escala (`1 << 16 = 65_536`), equivalente a `1.0`.
    pub const SCALE: i32 = 1 << Self::FRAC_BITS;

    /// Constante `0.0` en punto fijo.
    pub const ZERO: Self = Self(0);

    /// Constante `1.0` en punto fijo.
    pub const ONE: Self = Self(Self::SCALE);

    /// Valor mínimo representable en `Q16.16` (`-32_768.0`).
    pub const MIN: Self = Self(i32::MIN);

    /// Valor máximo representable en `Q16.16` (`~32_767.9999847`).
    pub const MAX: Self = Self(i32::MAX);

    /// Construye un [`Fixed`] directamente a partir de su representación cruda en `Q16.16`.
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// Devuelve el valor crudo subyacente en `Q16.16`.
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Convierte un entero `i32` a `Q16.16`.
    ///
    /// Devuelve [`FixedError::Overflow`] si el valor está fuera del rango `[-32_768, 32_767]`.
    pub const fn from_int(value: i32) -> Result<Self, FixedError> {
        let scaled = (value as i64) * (Self::SCALE as i64);
        match Self::from_i64_checked(scaled) {
            Some(fixed) => Ok(fixed),
            None => Err(FixedError::Overflow),
        }
    }

    /// Devuelve la parte entera truncada hacia cero (`self.0 / Self::SCALE`).
    pub const fn to_int_trunc(self) -> i32 {
        self.0 / Self::SCALE
    }

    /// Construye un [`Fixed`] a partir de la fracción `num / den` truncando hacia cero.
    ///
    /// Devuelve [`FixedError::DivisionByZero`] si `den == 0` y [`FixedError::Overflow`]
    /// si el cociente escalado no cabe en un `i32`.
    pub const fn from_ratio(num: i32, den: i32) -> Result<Self, FixedError> {
        if den == 0 {
            return Err(FixedError::DivisionByZero);
        }
        let quot = ((num as i64) * (Self::SCALE as i64)) / (den as i64);
        match Self::from_i64_checked(quot) {
            Some(fixed) => Ok(fixed),
            None => Err(FixedError::Overflow),
        }
    }

    /// Convierte a `f32` con posible pérdida de precisión.
    ///
    /// **Solo para presentación; no usar nunca en el estado ni en las reglas de la simulación.**
    pub fn to_f32_lossy(self) -> f32 {
        self.0 as f32 / (Self::SCALE as f32)
    }

    /// Suma dos valores en punto fijo, devolviendo `None` si desborda.
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.0.checked_add(rhs.0) {
            Some(res) => Some(Self(res)),
            None => None,
        }
    }

    /// Resta dos valores en punto fijo, devolviendo `None` si desborda.
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.0.checked_sub(rhs.0) {
            Some(res) => Some(Self(res)),
            None => None,
        }
    }

    /// Multiplica dos valores en punto fijo truncando hacia cero; devuelve `None` si desborda.
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        let prod = ((self.0 as i64) * (rhs.0 as i64)) / (Self::SCALE as i64);
        Self::from_i64_checked(prod)
    }

    /// Divide dos valores en punto fijo truncando hacia cero; devuelve `None` si `rhs == 0` o si desborda.
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        let quot = ((self.0 as i64) * (Self::SCALE as i64)) / (rhs.0 as i64);
        Self::from_i64_checked(quot)
    }

    /// Suma dos valores en punto fijo saturando en [`Fixed::MIN`] o [`Fixed::MAX`].
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Resta dos valores en punto fijo saturando en [`Fixed::MIN`] o [`Fixed::MAX`].
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Multiplica dos valores en punto fijo truncando hacia cero y saturando al rango de `i32`.
    pub const fn saturating_mul(self, rhs: Self) -> Self {
        let prod = ((self.0 as i64) * (rhs.0 as i64)) / (Self::SCALE as i64);
        Self::from_i64_saturating(prod)
    }

    /// Divide dos valores en punto fijo truncando hacia cero y saturando al rango de `i32`.
    ///
    /// Devuelve `None` si `rhs == 0` (nunca hace `panic!`).
    pub const fn saturating_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        let quot = ((self.0 as i64) * (Self::SCALE as i64)) / (rhs.0 as i64);
        Some(Self::from_i64_saturating(quot))
    }

    const fn from_i64_checked(value: i64) -> Option<Self> {
        if value < (i32::MIN as i64) || value > (i32::MAX as i64) {
            None
        } else {
            Some(Self(value as i32))
        }
    }

    const fn from_i64_saturating(value: i64) -> Self {
        if value < (i32::MIN as i64) {
            Self::MIN
        } else if value > (i32::MAX as i64) {
            Self::MAX
        } else {
            Self(value as i32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_one_third_times_three_is_within_one_ulp_of_one() {
        let third = Fixed::from_ratio(1, 3).unwrap();
        let three = Fixed::from_int(3).unwrap();
        let res = third.checked_mul(three).unwrap();
        assert_eq!((res.raw() - Fixed::ONE.raw()).abs(), 1);

        let neg_third = Fixed::from_ratio(-1, 3).unwrap();
        let neg_one = Fixed::from_int(-1).unwrap();
        let neg_res = neg_third.checked_mul(three).unwrap();
        assert_eq!((neg_res.raw() - neg_one.raw()).abs(), 1);
        assert_eq!(neg_third.raw(), -third.raw());
    }

    #[test]
    fn checked_mul_returns_none_on_overflow() {
        let a = Fixed::from_int(300).unwrap();
        let b = Fixed::from_int(300).unwrap();
        assert_eq!(a.checked_mul(b), None);

        let neg_a = Fixed::from_int(-300).unwrap();
        assert_eq!(neg_a.checked_mul(b), None);
    }

    #[test]
    fn saturating_mul_clamps_to_bounds_on_overflow() {
        let a = Fixed::from_int(300).unwrap();
        let b = Fixed::from_int(300).unwrap();
        let neg_a = Fixed::from_int(-300).unwrap();

        assert_eq!(a.saturating_mul(b), Fixed::MAX);
        assert_eq!(neg_a.saturating_mul(b), Fixed::MIN);
    }

    #[test]
    fn division_by_zero_never_panics() {
        let x = Fixed::from_int(42).unwrap();
        assert_eq!(Fixed::from_ratio(42, 0), Err(FixedError::DivisionByZero));
        assert_eq!(x.checked_div(Fixed::ZERO), None);
        assert_eq!(x.saturating_div(Fixed::ZERO), None);
    }

    #[test]
    fn add_sub_conversions_and_display_work_as_expected() {
        let a = Fixed::from_ratio(7, 2).unwrap();
        let b = Fixed::from_ratio(-5, 2).unwrap();

        assert_eq!(a.to_int_trunc(), 3);
        assert_eq!(b.to_int_trunc(), -2);
        assert_eq!(a.checked_add(b), Some(Fixed::ONE));
        assert_eq!(a.checked_sub(b), Some(Fixed::from_int(6).unwrap()));

        assert_eq!(Fixed::MAX.checked_add(Fixed::ONE), None);
        assert_eq!(Fixed::MIN.checked_sub(Fixed::ONE), None);
        assert_eq!(Fixed::MAX.saturating_add(Fixed::ONE), Fixed::MAX);
        assert_eq!(Fixed::MIN.saturating_sub(Fixed::ONE), Fixed::MIN);

        let half = Fixed::from_ratio(1, 2).unwrap();
        assert_eq!(Fixed::MAX.checked_div(half), None);
        assert_eq!(Fixed::MAX.saturating_div(half), Some(Fixed::MAX));
        assert_eq!(Fixed::from_int(40_000), Err(FixedError::Overflow));
        assert_eq!(Fixed::from_raw(Fixed::SCALE), Fixed::ONE);
        assert!((half.to_f32_lossy() - 0.5).abs() < 1e-6);
        assert!(!FixedError::DivisionByZero.to_string().is_empty());
        assert!(!FixedError::Overflow.to_string().is_empty());
    }
}
