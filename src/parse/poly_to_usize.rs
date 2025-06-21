use std::cell::LazyCell;

use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

use crate::polynomial::Polynomial;

pub(super) fn poly_to_usize(poly: &Polynomial, pos: impl ResolvesToStr) -> Result<usize, String> {
    let Some(n_rational) = poly.extract_constant() else {
        return Err(format!("{} must be a constant, got {}", pos.to_str(), poly));
    };
    if !n_rational.is_integer() {
        return Err(format!(
            "{} must be an integer, got {}",
            pos.to_str(),
            n_rational
        ));
    }
    let n_bigint = n_rational.to_integer();
    let Some(n) = n_bigint.to_usize() else {
        let errmsg = if n_bigint < BigInt::zero() {
            "must be non-negative"
        } else {
            "is too large"
        };
        return Err(format!("{} {}, got {}", pos.to_str(), errmsg, n_bigint));
    };
    Ok(n)
}

pub(super) trait ResolvesToStr {
    fn to_str(&self) -> &str;
}
impl ResolvesToStr for str {
    fn to_str(&self) -> &str {
        self
    }
}
impl<T: ResolvesToStr + ?Sized> ResolvesToStr for &'_ T {
    fn to_str(&self) -> &str {
        T::to_str(self)
    }
}
impl ResolvesToStr for String {
    fn to_str(&self) -> &str {
        self
    }
}
impl<T: ResolvesToStr, F: FnOnce() -> T> ResolvesToStr for LazyCell<T, F> {
    fn to_str(&self) -> &str {
        T::to_str(self)
    }
}
