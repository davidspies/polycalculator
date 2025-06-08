use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive};

use crate::polynomial::Polynomial;

// --- Math Helper Functions ---
pub(crate) fn factorial(n: &BigInt) -> BigInt {
    (1..=n.to_usize().unwrap()).map(BigInt::from).product()
}

pub(crate) fn pick(poly: Polynomial, k: u32) -> Polynomial {
    if k == 0 {
        return Polynomial::constant(BigRational::one());
    }
    let mut result = Polynomial::constant(BigRational::one());
    for i in 0..k {
        let i_poly = Polynomial::constant(BigRational::from_integer(i.into()));
        result = result * (poly.clone() - i_poly);
    }
    result
}

pub(crate) fn choose(poly: Polynomial, k: u32) -> Polynomial {
    if k == 0 {
        return Polynomial::constant(BigRational::one());
    }
    let mut num = pick(poly, k);
    let den = factorial(&BigInt::from(k));
    let den_rational = BigRational::from(den);
    num /= &den_rational;
    num
}

pub(crate) fn generate_pascal_triangle(rows: usize) -> Vec<Vec<BigInt>> {
    if rows == 0 {
        return vec![];
    }
    let mut triangle = vec![vec![BigInt::one()]];
    for i in 1..rows {
        let prev_row = &triangle[i - 1];
        let mut new_row = vec![BigInt::one()];
        for j in 0..prev_row.len() - 1 {
            new_row.push(&prev_row[j] + &prev_row[j + 1]);
        }
        new_row.push(BigInt::one());
        triangle.push(new_row);
    }
    triangle
}
