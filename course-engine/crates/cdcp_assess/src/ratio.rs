//! Reduced rational. Denominator is always > 0. No floating-point.
use crate::error::AssessError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Reduced `num/den` with `den > 0` and `gcd(|num|, den) == 1` (or num == 0 ⇒ den == 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ratio {
    num: i64,
    den: i64,
}

impl Ratio {
    pub fn new(num: i64, den: i64) -> Result<Self, AssessError> {
        if den == 0 {
            return Err(AssessError::ZeroDenominator);
        }
        reduce(i128::from(num), i128::from(den))
    }

    pub fn from_int(n: i64) -> Self {
        // den = 1; never fails.
        Self { num: n, den: 1 }
    }

    pub fn num(self) -> i64 {
        self.num
    }

    pub fn den(self) -> i64 {
        self.den
    }

    pub fn is_negative(self) -> bool {
        self.num < 0
    }

    pub fn abs(self) -> Self {
        if self.num < 0 {
            // i64::MIN cannot be negated in i64; treat as overflow at construction.
            // A reduced Ratio never holds i64::MIN because `new` rejects it.
            Self {
                num: -self.num,
                den: self.den,
            }
        } else {
            self
        }
    }

    pub fn checked_add(self, other: Self) -> Result<Self, AssessError> {
        // a/b + c/d = (a*d + c*b) / (b*d)
        let a = i128::from(self.num);
        let b = i128::from(self.den);
        let c = i128::from(other.num);
        let d = i128::from(other.den);
        let num = a
            .checked_mul(d)
            .and_then(|ad| c.checked_mul(b).and_then(|cb| ad.checked_add(cb)))
            .ok_or(AssessError::Overflow)?;
        let den = b.checked_mul(d).ok_or(AssessError::Overflow)?;
        reduce(num, den)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, AssessError> {
        self.checked_add(Ratio {
            num: other.num.checked_neg().ok_or(AssessError::Overflow)?,
            den: other.den,
        })
    }

    pub fn checked_mul(self, other: Self) -> Result<Self, AssessError> {
        let num = i128::from(self.num)
            .checked_mul(i128::from(other.num))
            .ok_or(AssessError::Overflow)?;
        let den = i128::from(self.den)
            .checked_mul(i128::from(other.den))
            .ok_or(AssessError::Overflow)?;
        reduce(num, den)
    }

    pub fn cmp_ratio(self, other: Self) -> Result<std::cmp::Ordering, AssessError> {
        // a/b ? c/d  with b>0, d>0  ⇒  a*d ? c*b
        let left = i128::from(self.num)
            .checked_mul(i128::from(other.den))
            .ok_or(AssessError::Overflow)?;
        let right = i128::from(other.num)
            .checked_mul(i128::from(self.den))
            .ok_or(AssessError::Overflow)?;
        Ok(left.cmp(&right))
    }

    pub fn le(self, other: Self) -> Result<bool, AssessError> {
        Ok(self.cmp_ratio(other)? != std::cmp::Ordering::Greater)
    }
}

impl Serialize for Ratio {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Ratio", 2)?;
        s.serialize_field("num", &self.num)?;
        s.serialize_field("den", &self.den)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Ratio {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            num: i64,
            den: i64,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ratio::new(raw.num, raw.den).map_err(serde::de::Error::custom)
    }
}

fn reduce(num: i128, den: i128) -> Result<Ratio, AssessError> {
    if den == 0 {
        return Err(AssessError::ZeroDenominator);
    }
    let neg = (num < 0) ^ (den < 0);
    let n = num.unsigned_abs();
    let d = den.unsigned_abs();
    let g = gcd_u128(n, d);
    let n = n / g;
    let d = d / g;
    let n = i64::try_from(n).map_err(|_| AssessError::Overflow)?;
    let d = i64::try_from(d).map_err(|_| AssessError::Overflow)?;
    // i64::MIN cannot appear: n is unsigned-abs then fits i64, so 0..=i64::MAX.
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
        assert_eq!(Ratio::new(1, 0), Err(AssessError::ZeroDenominator));
    }

    #[test]
    fn add_mul_are_exact() {
        let a = Ratio::new(1, 2).unwrap();
        let b = Ratio::new(1, 3).unwrap();
        let s = a.checked_add(b).unwrap();
        assert_eq!(s, Ratio::new(5, 6).unwrap());
        let p = a.checked_mul(b).unwrap();
        assert_eq!(p, Ratio::new(1, 6).unwrap());
    }

    #[test]
    fn cmp_cross_multiply() {
        let a = Ratio::new(2, 3).unwrap();
        let b = Ratio::new(3, 4).unwrap();
        assert_eq!(a.cmp_ratio(b).unwrap(), std::cmp::Ordering::Less);
        assert!(a.le(b).unwrap());
        assert!(b.le(b).unwrap());
    }
}
