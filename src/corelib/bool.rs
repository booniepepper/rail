use crate::rail_machine::{self, RailDef, RailType, RailVal};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        push_bool("true", true),
        push_bool("false", false),
        RailDef::on_state("not", &[Boolean], &[Boolean], |quote| {
            let (b, quote) = quote.pop_bool("not");
            quote.push_bool(!b)
        }),
        RailDef::on_state("or", &[Boolean, Boolean], &[Boolean], |quote| {
            let (b2, quote) = quote.pop_bool("or");
            let (b1, quote) = quote.pop_bool("or");
            quote.push_bool(b1 || b2)
        }),
        RailDef::on_state("and", &[Boolean, Boolean], &[Boolean], |quote| {
            let (b2, quote) = quote.pop_bool("and");
            let (b1, quote) = quote.pop_bool("and");
            quote.push_bool(b1 && b2)
        }),
        equality("eq?", Equality::Equal),
        equality("neq?", Equality::NotEqual),
        binary_numeric_pred("gt?", |a, b| b > a, |a, b| b > a),
        binary_numeric_pred("lt?", |a, b| b < a, |a, b| b < a),
        binary_numeric_pred("gte?", |a, b| b >= a, |a, b| b >= a),
        binary_numeric_pred("lte?", |a, b| b <= a, |a, b| b <= a),
        RailDef::on_state("any", &[Quote, Quote], &[Quote], |state| {
            let (predicate, state) = state.pop_quote("any");
            let (sequence, state) = state.pop_quote("any");

            for term in sequence.stack.values {
                let substate = state.child().push(term);
                let substate = predicate.clone().run_in_state(substate);
                let (pass, _) = substate.stack.pop_bool("any");
                if pass {
                    return state.push_bool(true);
                }
            }

            state.push_bool(false)
        }),
    ]
}

fn push_bool(name: &str, b: bool) -> RailDef<'_> {
    RailDef::on_state(name, &[], &[Boolean], move |quote| quote.push_bool(b))
}

enum Equality {
    Equal,
    NotEqual,
}

fn equality(name: &str, eq: Equality) -> RailDef<'_> {
    RailDef::on_state(name, &[A, A], &[Boolean], move |quote| {
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

fn binary_numeric_pred<'a, F, G>(name: &'a str, f64_op: F, i64_op: G) -> RailDef<'a>
where
    F: Fn(f64, f64) -> bool + Sized + 'a,
    G: Fn(i64, i64) -> bool + Sized + 'a,
{
    RailDef::on_state(name, &[Number, Number], &[Boolean], move |quote| {
        let (b, quote) = quote.pop();
        let (a, quote) = quote.pop();

        use RailVal::*;
        match (a, b) {
            (I64(a), I64(b)) => quote.push_bool(i64_op(a, b)),
            (I64(a), F64(b)) => quote.push_bool(f64_op(a as f64, b)),
            (F64(a), I64(b)) => quote.push_bool(f64_op(a, b as f64)),
            (F64(a), F64(b)) => quote.push_bool(f64_op(a, b)),
            (a, b) => {
                rail_machine::log_warn(format!(
                    "Can only perform {} on numeric values but got {} and {}",
                    name, a, b
                ));
                quote.push(a).push(b)
            }
        }
    })
}
