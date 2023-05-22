use clap::{Parser, Subcommand};
use rail_lang::v1::prompt::RailPrompt;
use rail_lang::v1::{
    loading, log, RunConventions, RAIL_ERROR_PREFIX, RAIL_FATAL_PREFIX, RAIL_INFO_PREFIX,
    RAIL_WARN_PREFIX,
};
use rail_lang::RAIL_VERSION;

const EXE_NAME: &str = "railsh";

const CONV: RunConventions = RunConventions {
    exe_name: EXE_NAME,
    exe_version: RAIL_VERSION,
    info_prefix: RAIL_INFO_PREFIX,
    warn_prefix: RAIL_WARN_PREFIX,
    error_prefix: RAIL_ERROR_PREFIX,
    fatal_prefix: RAIL_FATAL_PREFIX,
};

pub fn main() {
    let args = RailShell::parse();

    let state = match loading::initial_rail_state(args.no_stdlib, args.lib_list, &CONV) {
        Ok(state) => state,
        Err((state, err)) => {
            log::error(&CONV, format!("Error loading initial state: {:?}", err));
            log::error(&CONV, format!("State dump: {}", state.stack));
            std::process::exit(1);
        }
    };

    let end_state = match args.mode {
        Some(Mode::Interactive) | None => RailPrompt::new(&CONV).run(state),
        Some(Mode::Run { file }) => {
            let tokens = loading::get_source_file_as_tokens(file);
            state.run_tokens(tokens)
        }
        Some(Mode::RunStdin) => {
            log::error(&CONV, "I don't know how to run stdin yet");
            std::process::exit(1);
        }
    };

    let end_state = log::error_coerce(end_state);

    if !end_state.stack.is_empty() {
        log::error(&CONV, format!("State dump: {}", end_state.stack));
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
