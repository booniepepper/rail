use directories::ProjectDirs;
pub use loading::SourceConventions;
pub use rail_machine::RunConventions;
use std::path::PathBuf;

pub mod corelib;
pub mod loading;
pub mod log;
pub mod prompt;
pub mod rail_machine;
pub mod tokens;

pub const RAIL_VERSION: &str = std::env!("CARGO_PKG_VERSION");
pub const RAIL_INFO_PREFIX: &str = "";
pub const RAIL_WARN_PREFIX: &str = "[Warn] ";
pub const RAIL_ERROR_PREFIX: &str = "[Error] ";
pub const RAIL_FATAL_PREFIX: &str = "[Derailed] ";

pub fn rail_lib_path(conventions: &RunConventions) -> PathBuf {
    let app = format!("rail-{}", RAIL_VERSION);
    let home = ProjectDirs::from("army", "rail-lang", &app).unwrap_or_else(|| {
        log::error(
            conventions,
            "Unable to access a suitable directory for Rail.",
        );
        std::process::exit(1)
    });
    home.data_dir().to_owned()
}
