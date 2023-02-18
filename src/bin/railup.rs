use std::{env, fs, io, process};

use clap::{ArgMatches, Command};
use rail_lang::{rail_lib_path, RAIL_VERSION};

pub fn main() {
    let args = parse_args();

    match args.subcommand() {
        Some(("bootstrap", _)) => {
            let path = rail_lib_path();
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
                    "https://github.com/hiljusti/rail",
                    &path.to_string_lossy(),
                ])
                .output()
                .expect("Error running git clone");
            if !clone_result.status.success() {
                eprintln!("{}", String::from_utf8(clone_result.stderr).unwrap())
            }
        }
        Some(("zap", _)) => {
            let path = rail_lib_path();
            if let Err(e) = fs::remove_dir_all(path.clone()) {
                match e.kind() {
                    io::ErrorKind::NotFound => (),
                    kind => panic!("Unable to zap directory {:?} : {}", path, kind),
                }
            }
        }
        Some((unknown_cmd, _)) => {
            panic!("Unknown command: {}", unknown_cmd);
        }
        None => (),
    }
}

fn parse_args() -> ArgMatches {
    // TODO: How can I do these in the derive style?
    let bootstrap_help = format!("Fetch the Rail {} std library and extras", RAIL_VERSION);
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
