use crate::rail_machine::{self, Dictionary};

mod bool;
mod choice;
mod command;
mod display;
mod filesystem;
mod math;
mod meta;
mod process;
mod repeat;
mod sequence;
mod shuffle;
mod stab;
mod string;
mod test;

pub fn rail_builtin_dictionary() -> Dictionary {
    rail_machine::dictionary_of(
        [
            bool::builtins(),
            choice::builtins(),
            command::builtins(),
            display::builtins(),
            filesystem::builtins(),
            math::builtins(),
            meta::builtins(),
            process::builtins(),
            repeat::builtins(),
            shuffle::builtins(),
            sequence::builtins(),
            stab::builtins(),
            string::builtins(),
            test::builtins(),
        ]
        .concat(),
    )
}
