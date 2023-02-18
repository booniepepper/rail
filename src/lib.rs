use colored::Colorize;
use directories::ProjectDirs;
pub use loading::SourceConventions;
pub use rail_machine::RunConventions;
use std::path::PathBuf;

pub mod corelib;
pub mod loading;
pub mod prompt;
pub mod rail_machine;
pub mod tokens;

pub const RAIL_VERSION: &str = std::env!("CARGO_PKG_VERSION");
pub const RAIL_WARNING_PREFIX: &str = "WARN";
pub const RAIL_FATAL_PREFIX: &str = "Derailed";

pub fn rail_lib_path() -> PathBuf {
    let app = format!("rail-{}", RAIL_VERSION);
    let home = ProjectDirs::from("army", "rail-lang", &app).unwrap_or_else(|| {
        let msg = "Unable to access a suitable directory for Rail.".to_string();
        eprintln!("{}", msg.dimmed().red());
        std::process::exit(1)
    });
    home.data_dir().to_owned()
}
