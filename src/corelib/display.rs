use std::fmt::Display;

use crate::rail_machine::{RailDef, RailType, RailVal};
use crate::RAIL_VERSION;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        printer("p", "FIXME", &|a| print!("{}", a)),
        printer("pl", "FIXME", &|a| println!("{}", a)),
        RailDef::contextless("nl", "FIXME", &[], &[], || print!("\n")),
        RailDef::on_state("status", "FIXME", &[], &[], |state| {
            println!("{}", state.stack);
            state
        }),
        RailDef::contextless("clear", "FIXME", &[], &[], || {
            clearscreen::clear().expect("Unable to clear screen")
        }),
        RailDef::on_state("version", "FIXME", &[], &[RailType::String], |quote| {
            quote.push_str(RAIL_VERSION)
        }),
    ]
}

fn printer<'a, P>(name: &str, description: &str, p: &'a P) -> RailDef<'a>
where
    P: Fn(&dyn Display) + 'a,
{
    RailDef::on_state(name, description, &[RailType::A], &[], move |quote| {
        let (a, quote) = quote.pop();
        match a {
            RailVal::String(a) => p(&a),
            _ => p(&a),
        }
        quote
    })
}
