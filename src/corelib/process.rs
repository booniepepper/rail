use std::env;

use crate::rail_machine::{self, RailDef, RailType, RailVal};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state("exec", "Consume a string as an executable name, and produce a symbol table with \"status\", \"stdout\" and \"stderr\" keys.", &[String], &[Stab], |quote| {
            let (invocation, quote) = quote.pop_string("exec");
            let invocation = invocation.trim();
            let (exe, args) = invocation.split_once(' ').unwrap_or((invocation, ""));
            let args = args.split_ascii_whitespace().collect::<Vec<_>>();

            let res = std::process::Command::new(exe).args(args).output().unwrap();

            let mut result = rail_machine::new_stab();
            result.insert(
                "status".to_string(),
                RailVal::I64(res.status.code().unwrap_or(-1).into()),
            );
            result.insert(
                "stdout".to_string(),
                RailVal::String(
                    std::string::String::from_utf8(res.stdout)
                        .unwrap()
                        .trim_end()
                        .to_string(),
                ),
            );
            result.insert(
                "stderr".to_string(),
                RailVal::String(
                    std::string::String::from_utf8(res.stderr)
                        .unwrap()
                        .trim_end()
                        .to_string(),
                ),
            );

            quote.push_stab(result)
        }),
        RailDef::on_state("env", "Produce a symbol table of all environment variables.", &[], &[Stab], |quote| {
            let vars = env::vars().fold(rail_machine::new_stab(), |mut stab, (k, v)| {
                stab.insert(k, RailVal::String(v));
                stab
            });
            quote.push_stab(vars)
        }),
        RailDef::on_state("envget", "Consume a string as an environment key, and produce its value.", &[String], &[String], |quote| {
            let (key, quote) = quote.pop_string("envget");
            let var = env::var(key).unwrap_or_else(|_| "unset".to_string());
            quote.push_string(var)
        }),
        RailDef::on_state("envset", "Consume a string as an environment key and a string as its value, and set it in the current environment.", &[String, String], &[], |quote| {
            let (var, quote) = quote.pop_string("envset");
            let (key, quote) = quote.pop_string("envset");
            env::set_var(key, var);
            quote
        }),
        RailDef::on_state("stdin", "Read standard input and produce a list of lines.", &[], &[Quote], |quote| {
            let lines = std::io::stdin()
                .lines()
                .filter_map(|line| line.ok())
                .fold(quote.child(), |quote, line| quote.push_string(line));
            quote.push_quote(lines)
        }),
    ]
}
