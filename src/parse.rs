use std::cell::LazyCell;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;

use crate::{
    pascal::{choose, factorial, pick},
    polynomial::{x, Polynomial},
};

mod poly_to_usize;

use self::poly_to_usize::poly_to_usize;

// --- Recursive Descent Parser ---
type ParseResult<'a, T> = Result<(T, &'a str), String>;

pub(crate) fn parse_expr(input: &str) -> ParseResult<Polynomial> {
    let (mut poly, mut remaining) = parse_term(input)?;
    loop {
        let trimmed = remaining.trim_start();
        if trimmed.starts_with('+') {
            let (rhs, next_remaining) = parse_term(&trimmed[1..])?;
            poly += rhs;
            remaining = next_remaining;
        } else if trimmed.starts_with('-') {
            let (rhs, next_remaining) = parse_term(&trimmed[1..])?;
            poly -= rhs;
            remaining = next_remaining;
        } else {
            break;
        }
    }
    Ok((poly, remaining))
}

fn parse_term(input: &str) -> ParseResult<Polynomial> {
    let (mut poly, mut remaining) = parse_factor(input)?;
    loop {
        let trimmed = remaining.trim_start();
        if trimmed.starts_with('*') {
            let (rhs, next_remaining) = parse_factor(&trimmed[1..])?;
            poly = poly * rhs;
            remaining = next_remaining;
        } else if trimmed.starts_with('/') {
            let (rhs, next_remaining) = parse_factor(&trimmed[1..])?;
            let Some(divisor) = rhs.extract_constant() else {
                return Err(format!(
                    "Division must be by a constant number, not a polynomial \
                    containing 'x'. Problem term: {}",
                    rhs
                ));
            };
            if divisor.is_zero() {
                return Err("Division by zero is not allowed.".to_string());
            }
            poly /= &divisor;
            remaining = next_remaining;
        } else if !trimmed.is_empty() {
            let next_char = trimmed.chars().next().unwrap();
            if next_char.is_ascii_alphabetic() || next_char == '(' {
                let (rhs, next_remaining) = parse_factor(trimmed)?;
                poly = poly * rhs;
                remaining = next_remaining;
                continue;
            }
            break;
        } else {
            break;
        }
    }
    Ok((poly, remaining))
}

fn parse_factor(input: &str) -> ParseResult<Polynomial> {
    let input = input.trim_start();
    if input.starts_with('-') {
        let (poly, remaining) = parse_factor(&input[1..])?;
        let neg_one = Polynomial::constant(BigRational::from_integer(BigInt::from(-1)));
        Ok((neg_one * poly, remaining))
    } else {
        parse_power(input)
    }
}

fn parse_power(input: &str) -> ParseResult<Polynomial> {
    let (mut base, mut remaining) = parse_postfix(input)?;
    loop {
        let trimmed = remaining.trim_start();
        if trimmed.starts_with('^') {
            let (exponent, next_remaining) = parse_postfix(&trimmed[1..])?;
            let exp_val = poly_to_usize(&exponent, "Exponent")?;
            base = base.pow(exp_val);
            remaining = next_remaining;
        } else {
            break;
        }
    }
    Ok((base, remaining))
}

fn parse_postfix(input: &str) -> ParseResult<Polynomial> {
    let (mut poly, mut remaining) = parse_primary(input)?;
    loop {
        let trimmed = remaining.trim_start();
        if trimmed.starts_with('!') {
            let n = poly_to_usize(&poly, "Operand for !")?;
            poly = Polynomial::constant(factorial(n).into());
            remaining = &trimmed[1..];
        } else {
            break;
        }
    }
    Ok((poly, remaining))
}

fn parse_primary(input: &str) -> ParseResult<Polynomial> {
    let input = input.trim_start();
    if input.starts_with('(') {
        let (poly, remaining) = parse_expr(&input[1..])?;
        let remaining = remaining.trim_start();
        if !remaining.starts_with(')') {
            return Err("Mismatched parentheses".to_string());
        }
        Ok((poly, &remaining[1..]))
    } else if input
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_alphabetic())
    {
        let (ident, after_ident) = parse_identifier(input);
        if after_ident.trim_start().starts_with('(') {
            parse_function_call(ident, after_ident.trim_start())
        } else if ident == "x" {
            Ok((x(), after_ident))
        } else {
            Err(format!(
                "Unexpected identifier '{}' without function call",
                ident
            ))
        }
    } else {
        parse_number(input)
    }
}

fn parse_identifier(input: &str) -> (&str, &str) {
    let mut end = 0;
    for (i, c) in input.char_indices() {
        if c.is_ascii_alphabetic() {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    (&input[..end], &input[end..])
}

fn parse_function_call<'a>(ident: &'a str, input: &'a str) -> ParseResult<'a, Polynomial> {
    let after_paren = &input[1..];
    let (args, mut remaining) = parse_args(after_paren)?;
    if !remaining.starts_with(')') {
        return Err("Expected ')' to close function call".to_string());
    }
    remaining = &remaining[1..];

    match ident {
        "P" => parse_pascal_call(&args, remaining, 'P', pick),
        "C" => parse_pascal_call(&args, remaining, 'C', choose),
        _ => Err(format!("Unknown function '{}'", ident)),
    }
}

fn parse_pascal_call<'a>(
    args: &Vec<Polynomial>,
    remaining: &'a str,
    fn_name: char,
    f: impl FnOnce(Polynomial, usize) -> Polynomial,
) -> ParseResult<'a, Polynomial> {
    if args.len() != 2 {
        return Err(format!(
            "Function {} takes 2 arguments, got {}",
            fn_name,
            args.len()
        ));
    }
    let poly_arg = args[0].clone();
    let k_arg = &args[1];
    let pos = LazyCell::new(|| format!("Second argument to {}", fn_name));
    let k = poly_to_usize(k_arg, pos)?;
    Ok((f(poly_arg, k), remaining))
}

fn parse_args(input: &str) -> ParseResult<Vec<Polynomial>> {
    let mut args = Vec::new();
    let mut remaining = input.trim_start();
    if remaining.starts_with(')') {
        return Ok((args, remaining));
    }
    loop {
        let (arg, next_remaining) = parse_expr(remaining)?;
        args.push(arg);
        remaining = next_remaining.trim_start();
        if remaining.starts_with(')') {
            break;
        }
        if !remaining.starts_with(',') {
            return Err("Expected ',' or ')' in argument list".to_string());
        }
        remaining = &remaining[1..];
    }
    Ok((args, remaining))
}

fn parse_number(input: &str) -> ParseResult<Polynomial> {
    let input = input.trim_start();
    let mut end = 0;
    for (i, c) in input.char_indices() {
        if c.is_ascii_digit() {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return Err(format!(
            "Expected a number, variable 'x', or parenthesis but found '{}'",
            &input[..1.min(input.len())]
        ));
    }
    let num_str = &input[..end];
    let remaining = &input[end..];
    match num_str.parse::<BigInt>() {
        Ok(val) => Ok((Polynomial::constant(BigRational::from(val)), remaining)),
        Err(_) => Err(format!("Invalid number format for '{}'", num_str)),
    }
}
