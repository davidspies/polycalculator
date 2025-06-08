use std::borrow::Cow;
use std::ops::{Add, DivAssign, Mul, MulAssign, Sub};
use std::{fmt, slice};

use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

// --- Polynomial Struct and Operations ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Polynomial(Vec<BigRational>);

impl Polynomial {
    pub(crate) fn new_term(coeff: BigRational, power: usize) -> Self {
        let mut coeffs = vec![BigRational::zero(); power + 1];
        coeffs[power] = coeff;
        Polynomial(coeffs)
    }

    pub(crate) fn constant(val: BigRational) -> Self {
        Polynomial(vec![val])
    }

    fn trim(&mut self) {
        while self.degree() > 0 && self.0.last().unwrap().is_zero() {
            self.0.pop();
        }
    }

    pub(crate) fn eval(&self, x: &BigRational) -> BigRational {
        let mut result = BigRational::zero();
        for c in self.0.iter().rev() {
            result *= x;
            result += c;
        }
        result
    }

    pub(crate) fn pow(self, exp: u32) -> Self {
        if exp == 0 {
            return Polynomial::constant(BigRational::one());
        }
        let mut base = self;
        let mut acc = Polynomial::constant(BigRational::one());
        let mut n = exp;
        while n > 1 {
            if n % 2 == 1 {
                acc *= base.clone();
            }
            base *= base.clone();
            n /= 2;
        }
        acc * base
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.coeffs().all(|c| c.is_zero())
    }

    pub(crate) fn degree(&self) -> usize {
        self.0.len().saturating_sub(1)
    }

    pub(crate) fn coeff_at(&self, n: usize) -> BigRational {
        self.0.get(n).cloned().unwrap_or_else(BigRational::zero)
    }

    pub(crate) fn coeffs(&self) -> slice::Iter<BigRational> {
        self.0.iter()
    }

    pub(crate) fn extract_constant(&self) -> Option<Cow<BigRational>> {
        match self.0.len() {
            0 => Some(Cow::Owned(BigRational::zero())),
            1 => Some(Cow::Borrowed(&self.0[0])),
            _ => None,
        }
    }
}

// --- Operator Overloading ---
impl Add for Polynomial {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let max_len = self.0.len().max(rhs.0.len());
        let mut result_coeffs = vec![BigRational::zero(); max_len];
        for (i, c) in self.0.iter().enumerate() {
            result_coeffs[i] += c;
        }
        for (i, c) in rhs.0.iter().enumerate() {
            result_coeffs[i] += c;
        }
        let mut result = Polynomial(result_coeffs);
        result.trim();
        result
    }
}

impl Sub for Polynomial {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let max_len = self.0.len().max(rhs.0.len());
        let mut result_coeffs = vec![BigRational::zero(); max_len];
        for (i, c) in self.0.iter().enumerate() {
            result_coeffs[i] += c;
        }
        for (i, c) in rhs.0.iter().enumerate() {
            result_coeffs[i] -= c;
        }
        let mut result = Polynomial(result_coeffs);
        result.trim();
        result
    }
}

impl Mul for Polynomial {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Polynomial(vec![]);
        }
        let result_deg = self.degree() + rhs.degree();
        let mut result_coeffs = vec![BigRational::zero(); result_deg + 1];
        for (i, c1) in self.0.iter().enumerate() {
            for (j, c2) in rhs.0.iter().enumerate() {
                result_coeffs[i + j] += c1 * c2;
            }
        }
        let mut result = Polynomial(result_coeffs);
        result.trim();
        result
    }
}

impl MulAssign for Polynomial {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs
    }
}

impl MulAssign<&BigRational> for Polynomial {
    fn mul_assign(&mut self, rhs: &BigRational) {
        for coeff in &mut self.0 {
            *coeff *= rhs;
        }
    }
}

impl DivAssign<&BigRational> for Polynomial {
    fn div_assign(&mut self, rhs: &BigRational) {
        for coeff in &mut self.0 {
            *coeff /= rhs;
        }
    }
}

// --- Display Formatting ---
impl fmt::Display for Polynomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        let mut terms = Vec::new();
        for (i, coeff) in self.coeffs().enumerate().rev() {
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
            let coeff_str = if abs_coeff.is_one() && i > 0 {
                "".to_string()
            } else if abs_coeff.is_integer() {
                format!("{}", abs_coeff)
            } else {
                format!("({})", abs_coeff)
            };
            let var_str = match i {
                0 => "",
                1 => "x",
                _ => &format!("x^{}", i),
            };
            let term = if i > 0 && coeff_str.is_empty() {
                format!("{}{}", sign, var_str)
            } else if i > 0 && !coeff_str.is_empty() {
                format!("{}{}{}", sign, coeff_str, var_str)
            } else {
                format!("{}{}", sign, coeff_str)
            };
            terms.push(term);
        }
        write!(f, "{}", terms.join(""))
    }
}
