use crate::loading;
use crate::rail_machine::{self, RailState, RunConventions};
use crate::tokens::Token;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct RailPrompt {
    is_tty: bool,
    editor: Editor<()>,
    terms: Vec<Token>,
    conventions: &'static RunConventions<'static>,
}

impl RailPrompt {
    pub fn new(conventions: &'static RunConventions) -> RailPrompt {
        let mut editor = Editor::<()>::new().expect("Unable to boot editor");
        let is_tty = editor.dimensions().is_some();
        let terms = vec![];
        RailPrompt {
            is_tty,
            editor,
            terms,
            conventions,
        }
    }

    pub fn run(self, state: RailState) {
        let name_and_version = format!(
            "{} {}",
            self.conventions.exe_name, self.conventions.exe_version
        );
        eprintln!("{}", name_and_version.dimmed().red());

        let end_state = self.fold(state, |state, term| state.run_token(term));

        if !end_state.stack.is_empty() {
            let end_state_msg = format!("State dump: {}", end_state.stack);
            eprintln!("{}", end_state_msg.dimmed().red());
        }
    }
}

impl Iterator for RailPrompt {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        while self.terms.is_empty() {
            // If we're interactive with a human (at a TTY and not piped stdin),
            // we pad with a newline in case the user uses print without newline.
            // (Otherwise, the prompt will rewrite the line with output.)
            if self.is_tty {
                println!();
            }

            let input = self.editor.readline("> ");

            if let Err(e) = input {
                // ^D and ^C are not error cases.
                if let ReadlineError::Eof = e {
                    rail_machine::log_fatal(self.conventions, "End of input");
                    return None;
                } else if let ReadlineError::Interrupted = e {
                    rail_machine::log_fatal(self.conventions, "Process interrupt");
                    return None;
                }

                rail_machine::log_fatal(self.conventions, e);
                std::process::exit(1);
            }

            let input = input.unwrap();

            self.editor.add_history_entry(&input);

            self.terms = loading::get_source_as_tokens(input);
            self.terms.reverse();
        }

        self.terms.pop()
    }
}
