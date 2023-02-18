use clap::Parser;
use rail_lang::{
    loading, log, RunConventions, RAIL_ERROR_PREFIX, RAIL_FATAL_PREFIX, RAIL_INFO_PREFIX,
    RAIL_VERSION, RAIL_WARN_PREFIX,
};

const EXE_NAME: &str = "rail";

const CONV: RunConventions = RunConventions {
    exe_name: EXE_NAME,
    exe_version: RAIL_VERSION,
    info_prefix: RAIL_INFO_PREFIX,
    warn_prefix: RAIL_WARN_PREFIX,
    error_prefix: RAIL_ERROR_PREFIX,
    fatal_prefix: RAIL_FATAL_PREFIX,
};

pub fn main() {
    let args = RailEvaluator::parse();

    let state = match loading::initial_rail_state(args.no_stdlib, args.lib_list, &CONV) {
        Ok(state) => state,
        Err((state, err)) => {
            log::error(&CONV, format!("Error loading initial state: {:?}", err));
            log::error(&CONV, format!("State dump: {}", state.stack));
            std::process::exit(1);
        }
    };

    let tokens = loading::get_source_as_tokens(args.rail_code.join(" "));

    let end_state = match state.run_tokens(tokens) {
        Ok(state) => state,
        Err((state, err)) => {
            log::error(&CONV, format!("Exiting with error: {:?}", err));
            state
        }
    };

    if !end_state.stack.is_empty() {
        log::error(&CONV, format!("State dump: {}", end_state.stack));
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
