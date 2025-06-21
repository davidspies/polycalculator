use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::One;

use crate::polynomial::Polynomial;

// --- Math Helper Functions ---
pub(crate) fn factorial(n: usize) -> BigInt {
    (1..=n).map(BigInt::from).product()
}

pub(crate) fn pick(poly: Polynomial, k: usize) -> Polynomial {
    let mut result = Polynomial::constant(BigRational::one());
    for i in 0..k {
        let big_i = BigRational::from_integer(i.into());
        result *= poly.clone() - big_i;
    }
    result
}

pub(crate) fn choose(poly: Polynomial, k: usize) -> Polynomial {
    pick(poly, k) / &BigRational::from(factorial(k))
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
