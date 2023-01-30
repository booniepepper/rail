use crate::rail_machine::{self, RailDef, RailType, RailVal};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        unary_numeric_op("abs", "Consume a number and produce its absolute value.", |a| a.abs(), |a| a.abs()),
        unary_numeric_op("negate", "Consume a number and produce its negation.", |a| -a, |a| -a),
        unary_to_f64_op("sqrt", "Consume a number and produce its square root.", |a| a.sqrt()),
        unary_to_i64_op("floor", "Consume a number and produce its floor.", |a| a),
        binary_numeric_op("+", "Consume two numbers and produce their sum.", |a, b| a + b, |a, b| a + b),
        binary_numeric_op("-", "Consume two numbers and produce their difference.", |a, b| a - b, |a, b| a - b),
        binary_numeric_op("*", "Consume two numbers and produce their product.", |a, b| a * b, |a, b| a * b),
        binary_numeric_op("/", "Consume two numbers and produce their ratio.", |a, b| a / b, |a, b| a / b),
        binary_numeric_op("mod", "Consume two numbers and produce their remainder.", |a, b| a % b, |a, b| a % b),
        RailDef::on_state("int-max", "Produce the maximum integer value.", &[], &[I64], |quote| {
            quote.push_i64(i64::MAX)
        }),
        RailDef::on_state("int-min", "Produce the minimum integer value.", &[], &[I64], |quote| {
            quote.push_i64(i64::MIN)
        }),
        RailDef::on_state("float-max", "Produce the maximum floating-point value.", &[], &[F64], |quote| {
            quote.push_f64(f64::MAX)
        }),
        RailDef::on_state("float-min", "Produce the minimum floating-point value.", &[], &[F64], |quote| {
            quote.push_f64(f64::MIN)
        }),
        RailDef::on_state("digits", "Consume a number and produce a list of its decimal digits.", &[I64], &[Quote], |quote| {
            let (n, quote) = quote.pop_i64("digits");
            let ns = quote.child();
            if n == 0 {
                return quote.push_quote(ns.push_i64(0));
            }
            let mut ns = ns;
            let mut n = n;
            while n != 0 {
                ns = ns.push_i64(n % 10);
                n /= 10;
            }
            quote.push_quote(ns.reverse())
        }),
    ]
}

fn unary_numeric_op<'a, F, G>(
    name: &'a str,
    description: &'a str,
    f64_op: F,
    i64_op: G,
) -> RailDef<'a>
where
    F: Fn(f64) -> f64 + Sized + 'a,
    G: Fn(i64) -> i64 + Sized + 'a,
{
    RailDef::on_state(name, description, &[Number], &[Number], move |quote| {
        let (n, quote) = quote.pop();
        match n {
            RailVal::I64(n) => quote.push_i64(i64_op(n)),
            RailVal::F64(n) => quote.push_f64(f64_op(n)),
            _ => {
                rail_machine::log_warn(
                    quote.conventions,
                    format!("Can only perform {} on numeric values, but got {}", name, n),
                );
                quote.push(n)
            }
        }
    })
}

fn unary_to_f64_op<'a, F>(name: &'a str, description: &'a str, f64_op: F) -> RailDef<'a>
where
    F: Fn(f64) -> f64 + Sized + 'a,
{
    RailDef::on_state(name, description, &[Number], &[F64], move |quote| {
        let (n, quote) = quote.pop();
        match n {
            RailVal::I64(n) => quote.push_f64(f64_op(n as f64)),
            RailVal::F64(n) => quote.push_f64(f64_op(n)),
            _ => {
                rail_machine::log_warn(
                    quote.conventions,
                    format!("Can only perform {} on numeric values, but got {}", name, n),
                );
                quote.push(n)
            }
        }
    })
}

fn unary_to_i64_op<'a, F>(name: &'a str, description: &'a str, i64_op: F) -> RailDef<'a>
where
    F: Fn(i64) -> i64 + Sized + 'a,
{
    RailDef::on_state(name, description, &[Number], &[I64], move |quote| {
        let (n, quote) = quote.pop();
        match n {
            RailVal::I64(n) => quote.push_i64(i64_op(n)),
            RailVal::F64(n) => quote.push_i64(i64_op(n as i64)),
            _ => {
                rail_machine::log_warn(
                    quote.conventions,
                    format!("Can only perform {} on numeric values, but got {}", name, n),
                );
                quote.push(n)
            }
        }
    })
}

fn binary_numeric_op<'a, F, G>(
    name: &'a str,
    description: &'a str,
    f64_op: F,
    i64_op: G,
) -> RailDef<'a>
where
    F: Fn(f64, f64) -> f64 + Sized + 'a,
    G: Fn(i64, i64) -> i64 + Sized + 'a,
{
    RailDef::on_state(
        name,
        description,
        &[Number, Number],
        &[Number],
        move |quote| {
            let (b, quote) = quote.pop();
            let (a, quote) = quote.pop();

            use RailVal::*;
            match (a, b) {
                (I64(a), I64(b)) => quote.push_i64(i64_op(a, b)),
                (I64(a), F64(b)) => quote.push_f64(f64_op(a as f64, b)),
                (F64(a), I64(b)) => quote.push_f64(f64_op(a, b as f64)),
                (F64(a), F64(b)) => quote.push_f64(f64_op(a, b)),
                (a, b) => {
                    rail_machine::log_warn(
                        quote.conventions,
                        format!(
                            "Can only perform {} on numeric values but got {} and {}",
                            name, a, b
                        ),
                    );
                    quote.push(a).push(b)
                }
            }
        },
    )
}
