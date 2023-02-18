use crate::rail_machine::{RailRunResult, RailState, RunConventions};
use crate::tokens::Token;
use crate::{loading, log};
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct RailPrompt {
    is_tty: bool,
    editor: Editor<()>,
    conventions: &'static RunConventions<'static>,
}

impl RailPrompt {
    pub fn new(conventions: &'static RunConventions) -> RailPrompt {
        let mut editor = Editor::<()>::new().expect("Unable to boot editor");
        let is_tty = editor.dimensions().is_some();

        RailPrompt {
            is_tty,
            editor,
            conventions,
        }
    }

    pub fn run(self, state: RailState) -> RailRunResult {
        log::info(
            state.conventions,
            format!(
                "{} {}",
                self.conventions.exe_name, self.conventions.exe_version
            ),
        );

        self.fold(Ok(state), |state, term| match state {
            Ok(state) => state.run_tokens(term),
            err => err,
        })
    }
}

impl Iterator for RailPrompt {
    type Item = Vec<Token>;

    fn next(&mut self) -> Option<Self::Item> {
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
                log::fatal(self.conventions, "End of input");
                return None;
            } else if let ReadlineError::Interrupted = e {
                log::fatal(self.conventions, "Process interrupt");
                return None;
            }

            log::fatal(self.conventions, e);
            std::process::exit(1);
        }

        let input = input.unwrap();

        self.editor.add_history_entry(&input);

        Some(loading::get_source_as_tokens(input))
    }
}
