use crate::rail_machine::{RailDef, RailType};

use RailType::{A, B, C};

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state("drop", "FIXME", &[A], &[], |quote| quote.pop().1),
        RailDef::on_state("dup", "FIXME", &[A], &[A, A], |quote| {
            let (a, quote) = quote.pop();
            quote.push(a.clone()).push(a)
        }),
        RailDef::on_state("dup2", "FIXME", &[A, B], &[A, B, A, B], |quote| {
            let (b, quote) = quote.pop();
            let (a, quote) = quote.pop();
            quote.push(a.clone()).push(b.clone()).push(a).push(b)
        }),
        RailDef::on_state("swap", "FIXME", &[A, B], &[B, A], |quote| {
            let (a, quote) = quote.pop();
            let (b, quote) = quote.pop();
            quote.push(a).push(b)
        }),
        RailDef::on_state("rot", "FIXME", &[A, B, C], &[C, A, B], |quote| {
            let (a, quote) = quote.pop();
            let (b, quote) = quote.pop();
            let (c, quote) = quote.pop();
            quote.push(a).push(c).push(b)
        }),
    ]
}
