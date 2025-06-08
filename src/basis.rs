use std::borrow::Cow;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;

use crate::format::format_from_coeffs;
use crate::pascal::{factorial, pick};
use crate::polynomial::{x, Polynomial};

// --- Basis Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Basis {
    Standard,
    Binomial,
}

impl Polynomial {
    fn to_binomial_coeffs(&self) -> Vec<BigRational> {
        let mut residual = self.clone();
        let degree = self.degree();
        let mut binomial_coeffs = vec![BigRational::zero(); degree + 1];

        for n in (0..=degree).rev() {
            let c_n = residual.coeff_at(n);
            if !c_n.is_zero() {
                let n_factorial = factorial(&BigInt::from(n));
                binomial_coeffs[n] = c_n.clone() * n_factorial;

                let mut term_to_subtract = pick(x(), n as u32);
                term_to_subtract *= &c_n;

                residual = residual - term_to_subtract;
            }
        }
        binomial_coeffs
    }
}

impl Basis {
    pub(crate) fn format(&self, poly: &Polynomial) -> String {
        let coeffs = match self {
            Basis::Standard => Cow::Borrowed(poly.coeffs()),
            Basis::Binomial => Cow::Owned(poly.to_binomial_coeffs()),
        };
        let format_term = |degree: usize| -> String {
            match self {
                Basis::Standard => format!("x^{}", degree),
                Basis::Binomial => format!("C(x,{})", degree),
            }
        };
        format_from_coeffs(&coeffs, format_term)
    }
}
