use crate::v1::rail_machine::{RailDef, RailType};

use RailType::{A, B, C};

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state_noerr(
            "drop",
            "Consume one value and discard it.",
            &[A],
            &[],
            |quote| quote.pop().1,
        ),
        RailDef::on_state_noerr(
            "dup",
            "Consume one value and produce it two times.",
            &[A],
            &[A, A],
            |quote| {
                let (a, quote) = quote.pop();
                quote.push(a.clone()).push(a)
            },
        ),
        RailDef::on_state_noerr(
            "dup2",
            "Consume two values, and produce them two times.",
            &[A, B],
            &[A, B, A, B],
            |quote| {
                let (b, quote) = quote.pop();
                let (a, quote) = quote.pop();
                quote.push(a.clone()).push(b.clone()).push(a).push(b)
            },
        ),
        RailDef::on_state_noerr(
            "swap",
            "Consume two values, and produce them in reverse order.",
            &[A, B],
            &[B, A],
            |quote| {
                let (a, quote) = quote.pop();
                let (b, quote) = quote.pop();
                quote.push(a).push(b)
            },
        ),
        RailDef::on_state_noerr(
            "rot",
            "Consume three values, and produce them rotated once.",
            &[A, B, C],
            &[C, A, B],
            |quote| {
                let (a, quote) = quote.pop();
                let (b, quote) = quote.pop();
                let (c, quote) = quote.pop();
                quote.push(a).push(c).push(b)
            },
        ),
    ]
}
