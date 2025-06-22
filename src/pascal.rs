use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::One;

use crate::polynomial::Polynomial;

// --- Math Helper Functions ---
pub(crate) fn factorial(n: usize) -> BigInt {
    (1..=n).map(BigInt::from).product()
}

pub(crate) fn pick(poly: &Polynomial, k: usize) -> Polynomial {
    let mut result = Polynomial::constant(BigRational::one());
    for i in 0..k {
        result *= poly.clone() - BigRational::from_integer(i.into());
    }
    result
}

pub(crate) fn choose(poly: &Polynomial, k: usize) -> Polynomial {
    pick(poly, k) / &BigRational::from(factorial(k))
}
