use std::borrow::Cow;
use std::fmt;
use std::mem;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use num_rational::BigRational;
use num_traits::{One, Zero};

use crate::format::format_from_coeffs;

// --- Polynomial Struct and Operations ---
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Polynomial(Vec<BigRational>);

impl Polynomial {
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

    pub(crate) fn pow(self, mut n: usize) -> Self {
        let mut base = self;
        let mut acc = Polynomial::constant(BigRational::one());
        while n >= 1 {
            if n % 2 == 1 {
                acc *= base.clone();
            }
            base *= base.clone();
            n /= 2;
        }
        acc
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.coeffs().iter().all(|c| c.is_zero())
    }

    pub(crate) fn degree(&self) -> usize {
        self.0.len().saturating_sub(1)
    }

    pub(crate) fn coeff_at(&self, n: usize) -> BigRational {
        self.0.get(n).cloned().unwrap_or_else(BigRational::zero)
    }

    pub(crate) fn coeffs(&self) -> &[BigRational] {
        &self.0
    }

    pub(crate) fn extract_constant(&self) -> Option<Cow<BigRational>> {
        match self.0.len() {
            0 => Some(Cow::Owned(BigRational::zero())),
            1 => Some(Cow::Borrowed(&self.0[0])),
            _ => None,
        }
    }
}

pub(crate) fn x() -> Polynomial {
    Polynomial(vec![BigRational::zero(), BigRational::one()])
}

// --- Operator Overloading ---
impl Neg for Polynomial {
    type Output = Self;
    fn neg(mut self) -> Self {
        for coeff in &mut self.0 {
            *coeff = -mem::take(coeff);
        }
        self
    }
}

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

impl AddAssign for Polynomial {
    fn add_assign(&mut self, rhs: Self) {
        *self = mem::take(self) + rhs;
    }
}

impl AddAssign<BigRational> for Polynomial {
    fn add_assign(&mut self, rhs: BigRational) {
        if self.0.is_empty() {
            self.0.push(rhs);
        } else {
            self.0[0] += rhs;
        }
        self.trim();
    }
}

impl Add<BigRational> for Polynomial {
    type Output = Self;
    fn add(mut self, rhs: BigRational) -> Self {
        self += rhs;
        self
    }
}

impl Sub for Polynomial {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl SubAssign for Polynomial {
    fn sub_assign(&mut self, rhs: Self) {
        *self = mem::take(self) - rhs;
    }
}

impl SubAssign<BigRational> for Polynomial {
    fn sub_assign(&mut self, rhs: BigRational) {
        if self.0.is_empty() {
            self.0.push(-rhs);
        } else {
            self.0[0] -= rhs;
        }
        self.trim();
    }
}

impl Sub<BigRational> for Polynomial {
    type Output = Self;
    fn sub(mut self, rhs: BigRational) -> Self {
        self -= rhs;
        self
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

impl<'a> MulAssign<&'a BigRational> for Polynomial {
    fn mul_assign(&mut self, rhs: &'a BigRational) {
        for coeff in &mut self.0 {
            *coeff *= rhs;
        }
    }
}

impl<'a> Mul<&'a BigRational> for Polynomial {
    type Output = Self;
    fn mul(mut self, rhs: &'a BigRational) -> Self {
        self *= rhs;
        self
    }
}

impl<'a> DivAssign<&'a BigRational> for Polynomial {
    fn div_assign(&mut self, rhs: &'a BigRational) {
        for coeff in &mut self.0 {
            *coeff /= rhs;
        }
    }
}

impl Div<&BigRational> for Polynomial {
    type Output = Self;
    fn div(mut self, rhs: &BigRational) -> Self {
        self /= rhs;
        self
    }
}

impl fmt::Display for Polynomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_from_coeffs(self.coeffs(), |degree| format!("x^{}", degree)).fmt(f)
    }
}
