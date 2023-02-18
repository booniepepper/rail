use clap::Parser;
use colored::Colorize;
use rail_lang::{loading, RunConventions, RAIL_FATAL_PREFIX, RAIL_VERSION, RAIL_WARNING_PREFIX};

const EXE_NAME: &str = "rail";

const CONVENTIONS: RunConventions = RunConventions {
    exe_name: EXE_NAME,
    exe_version: RAIL_VERSION,
    warn_prefix: RAIL_WARNING_PREFIX,
    fatal_prefix: RAIL_FATAL_PREFIX,
};

pub fn main() {
    let args = RailEvaluator::parse();

    let state = match loading::initial_rail_state(args.no_stdlib, args.lib_list, &CONVENTIONS) {
        Ok(state) => state,
        Err((state, err)) => {
            eprintln!("Error loading initial state: {:?}", err);
            eprintln!("State dump: {}", state.stack);
            std::process::exit(1);
        }
    };

    let tokens = loading::get_source_as_tokens(args.rail_code.join(" "));

    let end_state = match state.run_tokens(tokens) {
        Ok(state) => state,
        Err((state, err)) => {
            let msg = format!("Exiting with error: {:?}", err);
            eprintln!("{}", msg.dimmed().red());
            state
        }
    };

    if !end_state.stack.is_empty() {
        let msg = format!("State dump: {}", end_state.stack);
        eprintln!("{}", msg.dimmed().red());
    }
}

#[derive(Parser)]
#[clap(name = EXE_NAME, version = RAIL_VERSION)]
/// Rail Evaluator. A straightforward programming language
struct RailEvaluator {
    #[clap(long)]
    /// Disable loading the Rail standard library.
    no_stdlib: bool,

    #[clap(short = 'l', long)]
    /// A file containing a line-separated list of library paths to preload.
    lib_list: Option<String>,

    /// Code to evaluate
    rail_code: Vec<String>,
}
