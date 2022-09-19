use clap::{Parser, Subcommand};
use rail_lang::prompt::RailPrompt;
use rail_lang::rail_machine::RunConventions;
use rail_lang::{loading, RAIL_FATAL_PREFIX, RAIL_VERSION, RAIL_WARNING_PREFIX};

const EXE_NAME: &str = "railsh";

const CONVENTIONS: RunConventions = RunConventions {
    exe_name: EXE_NAME,
    exe_version: RAIL_VERSION,
    warn_prefix: RAIL_WARNING_PREFIX,
    fatal_prefix: RAIL_FATAL_PREFIX,
};

pub fn main() {
    let args = RailShell::parse();

    let state = loading::initial_rail_state(args.no_stdlib, args.lib_list, &CONVENTIONS);

    match args.mode {
        Some(Mode::Interactive) | None => RailPrompt::new(&CONVENTIONS).run(state),
        Some(Mode::Run { file }) => {
            let tokens = loading::get_source_file_as_tokens(file);
            state.run_tokens(tokens);
        }
        Some(Mode::RunStdin) => unimplemented!("I don't know how to run stdin yet"),
    }
}

#[derive(Parser)]
#[clap(name = EXE_NAME, version = RAIL_VERSION)]
/// Rail Shell. A straightforward programming language
struct RailShell {
    #[clap(subcommand)]
    mode: Option<Mode>,

    #[clap(long)]
    /// Disable loading the Rail standard library.
    no_stdlib: bool,

    #[clap(short = 'l', long)]
    /// A file containing a line-separated list of library paths to preload.
    lib_list: Option<String>,
}

#[derive(Subcommand)]
enum Mode {
    #[clap(visible_alias = "i")]
    /// Start an interactive session. (Default when no subcommand specified)
    Interactive,

    #[clap(visible_alias = "r")]
    /// Execute a file.
    Run { file: String },

    #[clap(name = "-")]
    /// Read from standard input.
    RunStdin,
}
