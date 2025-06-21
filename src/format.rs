use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

pub(crate) fn format_from_coeffs(
    coeffs: &[BigRational],
    format_term: impl Fn(usize) -> String,
) -> String {
    if coeffs.is_empty() {
        return "0".to_string();
    }

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

        let var_str = match i {
            0 => format!("{}", abs_coeff), // Constant term
            1 => "x".to_string(),
            _ => format_term(i),
        };

        let term = if i == 0 || abs_coeff.is_one() {
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
