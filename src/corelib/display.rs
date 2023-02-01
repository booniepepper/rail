use std::fmt::Display;
use std::io::{self, Stdout, Write};

use crate::rail_machine::{RailDef, RailType, RailVal};

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        printer("p", "Consumes one value and prints it.", &|a| {
            print_or_die(|out| out.write_fmt(format_args!("{}", a)))
        }),
        printer(
            "pl",
            "Consumes one value, prints it, and prints a newline.",
            &|a| print_or_die(|out| out.write_fmt(format_args!("{}\n", a))),
        ),
        RailDef::contextless("nl", "Prints a newline.", &[], &[], || {
            print_or_die(|out| out.write(b"\n"))
        }),
        RailDef::on_state(
            "status",
            "Prints the current status of the program.",
            &[],
            &[],
            |state| {
                print_or_die(|out| out.write_fmt(format_args!("{}\n", state.stack)));
                state
            },
        ),
        RailDef::contextless(
            "clear",
            "When invoked in a terminal context, clears the screen.",
            &[],
            &[],
            || clearscreen::clear().expect("Unable to clear screen"),
        ),
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
