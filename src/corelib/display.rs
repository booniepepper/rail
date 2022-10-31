use std::fmt::Display;
use std::io::{self, Stdout, Write};

use crate::rail_machine::{RailDef, RailType, RailVal};
use crate::RAIL_VERSION;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        printer("p", "FIXME", &|a| {
            print_or_die(|out| out.write_fmt(format_args!("{}", a)))
        }),
        printer("pl", "FIXME", &|a| {
            print_or_die(|out| out.write_fmt(format_args!("{}\n", a)))
        }),
        RailDef::contextless("nl", "FIXME", &[], &[], || {
            print_or_die(|out| out.write(b"\n"))
        }),
        RailDef::on_state("status", "FIXME", &[], &[], |state| {
            print_or_die(|out| out.write_fmt(format_args!("{}\n", state.stack)));
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

fn print_or_die<P, T>(p: P)
where
    P: Fn(&mut Stdout) -> io::Result<T>,
{
    let res = p(&mut io::stdout());
    if res.is_err() {
        std::process::exit(1);
    }
}
