use std::cell::LazyCell;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;

use crate::{
    pascal::{choose, factorial, pick},
    polynomial::{x, Polynomial},
};

mod poly_to_usize;
mod stream;

use self::poly_to_usize::poly_to_usize;
use self::stream::Stream;

type Result<T> = std::result::Result<T, String>;

pub(crate) fn parse(input: &str) -> Result<Polynomial> {
    let mut stream = Stream::new(input);
    let poly = parse_expr(&mut stream)?;
    match stream.finish() {
        Ok(()) => Ok(poly),
        Err(remainder) => Err(format!("Unexpected input after parsing: '{}'", remainder)),
    }
}

fn parse_expr(input: &mut Stream) -> Result<Polynomial> {
    let mut poly = parse_term(input)?;
    loop {
        if input.take_char('+') {
            poly += parse_term(input)?;
        } else if input.take_char('-') {
            poly -= parse_term(input)?;
        } else {
            break;
        }
    }
    Ok(poly)
}

fn parse_term(input: &mut Stream) -> Result<Polynomial> {
    let mut poly = parse_factor(input)?;
    loop {
        if input.take_char('*') {
            poly *= parse_factor(input)?;
        } else if input.take_char('/') {
            let rhs = parse_factor(input)?;
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
        } else if let Some(next_char) = input.peek_char() {
            if next_char.is_ascii_alphabetic() || next_char == '(' {
                poly *= parse_factor(input)?;
                continue;
            }
            break;
        } else {
            break;
        }
    }
    Ok(poly)
}

fn parse_factor(input: &mut Stream) -> Result<Polynomial> {
    if input.take_char('-') {
        Ok(-parse_factor(input)?)
    } else {
        parse_power(input)
    }
}

fn parse_power(input: &mut Stream) -> Result<Polynomial> {
    let mut base = parse_postfix(input)?;
    loop {
        if input.take_char('^') {
            let exponent = parse_postfix(input)?;
            let exp_val = poly_to_usize(&exponent, "Exponent")?;
            base = base.pow(exp_val);
        } else {
            break;
        }
    }
    Ok(base)
}

fn parse_postfix(input: &mut Stream) -> Result<Polynomial> {
    let mut poly = parse_primary(input)?;
    loop {
        if input.take_char('!') {
            let n = poly_to_usize(&poly, "Operand for !")?;
            poly = Polynomial::constant(factorial(n).into());
        } else {
            break;
        }
    }
    Ok(poly)
}

fn parse_primary(input: &mut Stream) -> Result<Polynomial> {
    if input.take_char('(') {
        let poly = parse_expr(input)?;
        if !input.take_char(')') {
            return Err("Mismatched parentheses".to_string());
        }
        Ok(poly)
    } else if input.peek_char().is_some_and(|c| c.is_ascii_alphabetic()) {
        let ident = input.parse_all_matching(char::is_ascii_alphabetic);
        if input.take_char('(') {
            let args = parse_args(input)?;
            if !input.take_char(')') {
                return Err("Expected ')' to close function call".to_string());
            }
            match ident {
                "P" => parse_function_call(&args, 'P', pick),
                "C" => parse_function_call(&args, 'C', choose),
                _ => Err(format!("Unknown function '{}'", ident)),
            }
        } else if ident == "x" {
            Ok(x())
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

fn parse_function_call(
    args: &[Polynomial],
    fn_name: char,
    f: impl FnOnce(&Polynomial, usize) -> Polynomial,
) -> Result<Polynomial> {
    if args.len() != 2 {
        return Err(format!(
            "Function {} takes 2 arguments, got {}",
            fn_name,
            args.len()
        ));
    }
    let poly_arg = &args[0];
    let k_arg = &args[1];
    let pos = LazyCell::new(|| format!("Second argument to {}", fn_name));
    let k = poly_to_usize(k_arg, pos)?;
    Ok(f(poly_arg, k))
}

fn parse_args(input: &mut Stream) -> Result<Vec<Polynomial>> {
    let mut args = Vec::new();
    if input.peek_char() == Some(')') {
        return Ok(args);
    }
    loop {
        let arg = parse_expr(input)?;
        args.push(arg);
        if input.peek_char() == Some(')') {
            break;
        }
        if !input.take_char(',') {
            return Err("Expected ',' or ')' in argument list".to_string());
        }
    }
    Ok(args)
}

fn parse_number(input: &mut Stream) -> Result<Polynomial> {
    let num_str = input.parse_all_matching(char::is_ascii_digit);
    match num_str.parse::<BigInt>() {
        Ok(val) => Ok(Polynomial::constant(BigRational::from(val))),
        Err(_) => Err(format!("Invalid number format for '{}'", num_str)),
    }
}
