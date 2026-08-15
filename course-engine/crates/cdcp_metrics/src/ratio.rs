//! Reduced rational. Denominator is always > 0. No floating-point.

use crate::error::MetricsError;

/// Reduced `num/den` with `den > 0` and `gcd(|num|, den) == 1`
/// (or `num == 0` ⇒ `den == 1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ratio {
    num: i64,
    den: i64,
}

impl Ratio {
    /// Construct and reduce. `den == 0` is [`MetricsError::ZeroDenominator`].
    pub fn new(num: i64, den: i64) -> Result<Self, MetricsError> {
        if den == 0 {
            return Err(MetricsError::ZeroDenominator);
        }
        reduce(i128::from(num), i128::from(den))
    }

    /// Integer `n/1`.
    #[must_use]
    pub fn from_int(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    /// Numerator of the reduced form.
    #[must_use]
    pub fn num(self) -> i64 {
        self.num
    }

    /// Denominator of the reduced form (always > 0).
    #[must_use]
    pub fn den(self) -> i64 {
        self.den
    }

    /// True when `num < 0`.
    #[must_use]
    pub fn is_negative(self) -> bool {
        self.num < 0
    }

    /// True when `num == 0`.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.num == 0
    }

    /// `a/b + c/d` with overflow checks.
    pub fn checked_add(self, other: Self) -> Result<Self, MetricsError> {
        let a = i128::from(self.num);
        let b = i128::from(self.den);
        let c = i128::from(other.num);
        let d = i128::from(other.den);
        let num = a
            .checked_mul(d)
            .and_then(|ad| c.checked_mul(b).and_then(|cb| ad.checked_add(cb)))
            .ok_or(MetricsError::Overflow)?;
        let den = b.checked_mul(d).ok_or(MetricsError::Overflow)?;
        reduce(num, den)
    }

    /// `a/b − c/d`.
    pub fn checked_sub(self, other: Self) -> Result<Self, MetricsError> {
        self.checked_add(Ratio {
            num: other.num.checked_neg().ok_or(MetricsError::Overflow)?,
            den: other.den,
        })
    }

    /// `a/b × c/d`.
    pub fn checked_mul(self, other: Self) -> Result<Self, MetricsError> {
        let num = i128::from(self.num)
            .checked_mul(i128::from(other.num))
            .ok_or(MetricsError::Overflow)?;
        let den = i128::from(self.den)
            .checked_mul(i128::from(other.den))
            .ok_or(MetricsError::Overflow)?;
        reduce(num, den)
    }

    /// `a/b ÷ c/d`.
    pub fn checked_div(self, other: Self) -> Result<Self, MetricsError> {
        if other.num == 0 {
            return Err(MetricsError::ZeroDenominator);
        }
        self.checked_mul(Ratio::new(other.den, other.num)?)
    }

    /// Cross-multiply compare. No floating-point.
    pub fn cmp_ratio(self, other: Self) -> Result<std::cmp::Ordering, MetricsError> {
        let left = i128::from(self.num)
            .checked_mul(i128::from(other.den))
            .ok_or(MetricsError::Overflow)?;
        let right = i128::from(other.num)
            .checked_mul(i128::from(self.den))
            .ok_or(MetricsError::Overflow)?;
        Ok(left.cmp(&right))
    }
}

impl std::fmt::Display for Ratio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

fn reduce(num: i128, den: i128) -> Result<Ratio, MetricsError> {
    if den == 0 {
        return Err(MetricsError::ZeroDenominator);
    }
    let neg = (num < 0) ^ (den < 0);
    let n = num.unsigned_abs();
    let d = den.unsigned_abs();
    let g = gcd_u128(n, d);
    let n = n / g;
    let d = d / g;
    let n = i64::try_from(n).map_err(|_| MetricsError::Overflow)?;
    let d = i64::try_from(d).map_err(|_| MetricsError::Overflow)?;
    Ok(Ratio {
        num: if neg { -n } else { n },
        den: d,
    })
}

fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces_and_normalizes_sign() {
        let r = Ratio::new(4, -2).unwrap();
        assert_eq!(r.num(), -2);
        assert_eq!(r.den(), 1);
        let z = Ratio::new(0, 8).unwrap();
        assert_eq!(z.num(), 0);
        assert_eq!(z.den(), 1);
    }

    #[test]
    fn zero_den_is_error() {
        assert_eq!(Ratio::new(1, 0), Err(MetricsError::ZeroDenominator));
    }

    #[test]
    fn add_mul_div_are_exact() {
        let a = Ratio::new(1, 2).unwrap();
        let b = Ratio::new(1, 3).unwrap();
        assert_eq!(a.checked_add(b).unwrap(), Ratio::new(5, 6).unwrap());
        assert_eq!(a.checked_mul(b).unwrap(), Ratio::new(1, 6).unwrap());
        assert_eq!(a.checked_div(b).unwrap(), Ratio::new(3, 2).unwrap());
    }

    #[test]
    fn cmp_cross_multiply() {
        let a = Ratio::new(6, 5).unwrap();
        let b = Ratio::new(5, 4).unwrap();
        assert_eq!(a.cmp_ratio(b).unwrap(), std::cmp::Ordering::Less);
    }
}
