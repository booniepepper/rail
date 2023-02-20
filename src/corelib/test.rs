use crate::log;
use crate::rail_machine::{RailDef, RailType};

use RailType::*;

// TODO: More forms, optional messages, etc. Input as stab? Output as stab or quote of failures?
pub fn builtins() -> Vec<RailDef<'static>> {
    vec![RailDef::on_state(
        "assert-true",
        "FIXME",
        &[Boolean, String],
        &[],
        |quote| {
            let (msg, quote) = quote.pop_string("assert-true");
            let (b, quote) = quote.pop_bool("assert-true");

            if !b {
                log::error(quote.conventions, format!("Assertion failed: {}", msg));
                std::process::exit(1);
            }

            Ok(quote)
        },
    )]
}
