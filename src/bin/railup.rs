use std::{env, fs, io, process};

use clap::{ArgMatches, Command};
use rail_lang::v1::{
    log, rail_lib_path, RunConventions, RAIL_ERROR_PREFIX, RAIL_FATAL_PREFIX, RAIL_INFO_PREFIX,
    RAIL_WARN_PREFIX,
};
use rail_lang::RAIL_VERSION;

const CONV: RunConventions = RunConventions {
    exe_name: "railup",
    exe_version: RAIL_VERSION,
    info_prefix: RAIL_INFO_PREFIX,
    warn_prefix: RAIL_WARN_PREFIX,
    error_prefix: RAIL_ERROR_PREFIX,
    fatal_prefix: RAIL_FATAL_PREFIX,
};

pub fn main() {
    let args = parse_args();

    match args.subcommand() {
        Some(("bootstrap", _)) | None => {
            let path = rail_lib_path(&CONV);
            fs::create_dir_all(path.clone())
                .unwrap_or_else(|e| panic!("Couldn't create {:?} : {}", path, e));
            env::set_current_dir(path.clone())
                .unwrap_or_else(|e| panic!("Couldn't access {:?} : {}", path, e));
            let version_tag = format!("v{}", RAIL_VERSION);
            let clone_result = process::Command::new("git")
                .args([
                    "clone",
                    "--single-branch",
                    "--branch",
                    &version_tag,
                    "https://github.com/booniepepper/rail",
                    &path.to_string_lossy(),
                ])
                .output()
                .expect("Error running git clone");
            if !clone_result.status.success() {
                log::error(&CONV, String::from_utf8(clone_result.stderr).unwrap())
            }
        }
        Some(("zap", _)) => {
            let path = rail_lib_path(&CONV);
            if let Err(e) = fs::remove_dir_all(path.clone()) {
                match e.kind() {
                    io::ErrorKind::NotFound => (),
                    kind => {
                        log::error(
                            &CONV,
                            format!("Unable to zap directory {:?} : {}", path, kind),
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
        Some((unknown_cmd, _)) => {
            log::error(&CONV, format!("Unknown command: {}", unknown_cmd));
            std::process::exit(1);
        }
    }
}

fn parse_args() -> ArgMatches {
    // TODO: How can I do these in the derive style?
    let bootstrap_help = format!("Fetch the Rail {} std library and extras. This is the default behavior when no subcommand is provided", RAIL_VERSION);
    let zap_help = format!(
        "Destroy any cached Rail {} std library and extras",
        RAIL_VERSION
    );

    Command::new("railup")
        .about("Rail Updater. Provides updates for the Rail programming language")
        .subcommand(
            Command::new("bootstrap")
                .short_flag('b')
                .about(bootstrap_help),
        )
        .subcommand(Command::new("zap").about(zap_help))
        .get_matches()
}
