use std::fmt::Display;

use colored::Colorize;

use crate::{rail_machine::RailVal, RunConventions};

pub fn info(conv: &RunConventions, thing: impl Display) {
    eprintln!("{}{}", conv.info_prefix, thing.to_string().dimmed().red());
}

pub fn warn(conv: &RunConventions, thing: impl Display) {
    eprintln!("{}{}", conv.warn_prefix, thing.to_string().dimmed().red());
}

pub fn error(conv: &RunConventions, thing: impl Display) {
    eprintln!("{}{}", conv.error_prefix, thing.to_string().dimmed().red());
}

pub fn fatal(conv: &RunConventions, thing: impl Display) {
    eprintln!("{}{}", conv.fatal_prefix, thing.to_string().dimmed().red());
}

// TODO: Anywhere this is used should return Result
pub fn type_panic_msg(context: &str, expected: &str, actual: RailVal) -> String {
    format!(
        "[Context: {}] Wanted {}, but got {}",
        context, expected, actual
    )
}
