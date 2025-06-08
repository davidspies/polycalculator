use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::pascal::{factorial, pick};
use crate::polynomial::Polynomial;

// --- Basis Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Basis {
    Standard,
    Binomial,
}

impl Polynomial {
    // New: Converts from standard basis to binomial basis coefficients
    fn to_binomial_coeffs(&self) -> Vec<BigRational> {
        let mut residual = self.clone();
        let degree = self.degree();
        let mut binomial_coeffs = vec![BigRational::zero(); degree + 1];

        for n in (0..=degree).rev() {
            let c_n = residual.coeff_at(n);
            if !c_n.is_zero() {
                let n_factorial = factorial(&BigInt::from(n));
                binomial_coeffs[n] = c_n.clone() * n_factorial;

                let mut term_to_subtract =
                    pick(Polynomial::new_term(BigRational::one(), 1), n as u32);
                term_to_subtract *= &c_n;

                residual = residual - term_to_subtract;
            }
        }
        binomial_coeffs
    }
}

pub(crate) fn format_poly(poly: &Polynomial, basis: Basis) -> String {
    if poly.is_zero() {
        return "0".to_string();
    }

    let coeffs = match basis {
        Basis::Standard => poly.coeffs().cloned().collect(),
        Basis::Binomial => poly.to_binomial_coeffs(),
    };

    let mut terms = Vec::new();
    for (i, coeff) in coeffs.iter().enumerate().rev() {
        if coeff.is_zero() {
            continue;
        }

        let sign = if terms.is_empty() {
            if coeff.is_negative() {
                "-"
            } else {
                ""
            }
        } else if coeff.is_negative() {
            " - "
        } else {
            " + "
        };
        let abs_coeff = coeff.abs();
        let coeff_str = if abs_coeff.is_one() {
            "".to_string()
        } else if abs_coeff.is_integer() {
            format!("{}*", abs_coeff)
        } else {
            format!("({})*", abs_coeff)
        };

        let var_str = match (basis, i) {
            (_, 0) => format!("{}", abs_coeff), // Constant term
            (_, 1) => "x".to_string(),
            (Basis::Standard, _) => format!("x^{}", i),
            (Basis::Binomial, _) => format!("C(x,{})", i),
        };

        // Don't show "*C(x,k)" if coefficient is 1
        let term = if i == 0 {
            format!("{}{}", sign, var_str)
        } else if abs_coeff.is_one() {
            format!("{}{}", sign, var_str)
        } else {
            format!("{}{}{}", sign, coeff_str, var_str)
        };

        terms.push(term);
    }

    if terms.is_empty() {
        "0".to_string()
    } else {
        terms.join("")
    }
}
