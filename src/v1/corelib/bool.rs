use crate::v1::log;
use crate::v1::rail_machine::{RailDef, RailType, RailVal};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state("not", "Consumes one boolean value and produces its inverse.", &[Boolean], &[Boolean], |state| {
            let (b, state) = state.pop_bool("not");
            Ok(state.push_bool(!b))
        }),
        RailDef::on_state("or", "Consumes two boolean values. If either are true, produces true. Otherwise produces false.", &[Boolean, Boolean], &[Boolean], |state| {
            let (b2, state) = state.pop_bool("or");
            let (b1, state) = state.pop_bool("or");
            Ok(state.push_bool(b1 || b2))
        }),
        RailDef::on_state("and", "Consumes two boolean values. If both are true, produces true. Otherwise produces false.", &[Boolean, Boolean], &[Boolean], |state| {
            let (b2, state) = state.pop_bool("and");
            let (b1, state) = state.pop_bool("and");
            Ok(state.push_bool(b1 && b2))
        }),
        equality("eq?", "Consumes two values. If they're equal, produces true. Otherwise produces false.", Equality::Equal),
        equality("neq?", "Consumes two values. If they're not equal, produces true. Otherwise produces false.", Equality::NotEqual),
        binary_numeric_pred("gt?", "Consumes two numbers. If the top value is greater, produces true. Otherwise produces false.", |a, b| b > a, |a, b| b > a),
        binary_numeric_pred("lt?", "Consumes two numbers. If the top value is lesser, produces true. Otherwise produces false.", |a, b| b < a, |a, b| b < a),
        binary_numeric_pred("gte?", "Consumes two numbers. If the top value is greater or equal, produces true. Otherwise produces false.", |a, b| b >= a, |a, b| b >= a),
        binary_numeric_pred("lte?", "Consumes two numbers. If the top value is lesser or equal, produces true. Otherwise produces false.", |a, b| b <= a, |a, b| b <= a),
        RailDef::on_state_noerr("any", "Consumes a sequence and a predicate. If the predicate applied to any value in the sequence is true, produces true. Otherwise produces false.", &[Quote, Quote], &[Quote], |state| {
            let (predicate, state) = state.pop_quote("any");
            let (sequence, state) = state.pop_quote("any");

            for term in sequence.stack.values {
                let substate = state.child().push(term);
                let substate = predicate.clone().run_in_state(substate);
                if let Ok(substate) = substate {
                    let (pass, _) = substate.stack.pop_bool("any");
                    if pass {
                        return state.push_bool(true);
                    }
                }
            }

            state.push_bool(false)
        }),
    ]
}

enum Equality {
    Equal,
    NotEqual,
}

fn equality<'a>(name: &'a str, description: &'a str, eq: Equality) -> RailDef<'a> {
    RailDef::on_state_noerr(name, description, &[A, A], &[Boolean], move |quote| {
        let (b, quote) = quote.pop();
        let (a, quote) = quote.pop();

        let res = a == b;

        let res = match eq {
            Equality::Equal => res,
            Equality::NotEqual => !res,
        };

        quote.push_bool(res)
    })
}

fn binary_numeric_pred<'a, F, G>(
    name: &'a str,
    description: &'a str,
    f64_op: F,
    i64_op: G,
) -> RailDef<'a>
where
    F: Fn(f64, f64) -> bool + Sized + 'a,
    G: Fn(i64, i64) -> bool + Sized + 'a,
{
    RailDef::on_state_noerr(
        name,
        description,
        &[Number, Number],
        &[Boolean],
        move |quote| {
            let (b, quote) = quote.pop();
            let (a, quote) = quote.pop();

            use RailVal::*;
            match (a, b) {
                (I64(a), I64(b)) => quote.push_bool(i64_op(a, b)),
                (I64(a), F64(b)) => quote.push_bool(f64_op(a as f64, b)),
                (F64(a), I64(b)) => quote.push_bool(f64_op(a, b as f64)),
                (F64(a), F64(b)) => quote.push_bool(f64_op(a, b)),
                (a, b) => {
                    log::warn(
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
