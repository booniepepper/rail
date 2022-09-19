use clap::Parser;
use rail_lang::{loading, RunConventions, RAIL_FATAL_PREFIX, RAIL_VERSION, RAIL_WARNING_PREFIX};

const EXE_NAME: &str = "railc";

const CONVENTIONS: RunConventions = RunConventions {
    exe_name: EXE_NAME,
    exe_version: RAIL_VERSION,
    warn_prefix: RAIL_WARNING_PREFIX,
    fatal_prefix: RAIL_FATAL_PREFIX,
};

pub fn main() {
    let args = RailCompiler::parse();

    let _state = loading::initial_rail_state(args.no_stdlib, args.lib_list, &CONVENTIONS);

    eprintln!("I'm not implemented yet.");
    std::process::exit(1);
}

#[derive(Parser)]
#[clap(name = EXE_NAME, version = RAIL_VERSION)]
/// Rail Compiler. A straightforward programming language
struct RailCompiler {
    #[clap(long)]
    /// Disable loading the Rail standard library.
    no_stdlib: bool,

    #[clap(short = 'l', long)]
    /// A file containing a line-separated list of library paths to preload.
    lib_list: Option<String>,
}
