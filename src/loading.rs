use std::{fmt::Debug, fs, path::Path};

use crate::rail_machine::{RailState, RunConventions};
use crate::tokens;
use crate::{rail_lib_path, rail_machine};

pub struct SourceConventions<'a> {
    pub lib_exts: &'a [&'a str],
    pub lib_list_exts: &'a [&'a str],
}

pub const RAIL_SOURCE_CONVENTIONS: SourceConventions = SourceConventions {
    lib_exts: &[".rail"],
    lib_list_exts: &[".txt"],
};

impl SourceConventions<'_> {
    pub fn is_lib(&self, filename: &str) -> bool {
        self.has_ext(filename, self.lib_exts)
    }

    pub fn is_lib_list(&self, filename: &str) -> bool {
        self.has_ext(filename, self.lib_list_exts)
    }

    fn has_ext(&self, filename: &str, exts: &[&str]) -> bool {
        exts.iter().any(|ext| filename.ends_with(ext))
    }
}

pub fn initial_rail_state(
    skip_stdlib: bool,
    lib_list: Option<String>,
    rc: &'static RunConventions,
) -> RailState {
    let state = RailState::new_main(rc);

    let state = if skip_stdlib {
        state
    } else {
        let tokens = from_rail_stdlib(rc);
        state.run_tokens(tokens)
    };

    if let Some(lib_list) = lib_list {
        let tokens = from_lib_list(&lib_list, &RAIL_SOURCE_CONVENTIONS);
        state.run_tokens(tokens)
    } else {
        state
    }
}

pub fn get_source_as_tokens(source: String) -> Vec<String> {
    source.split('\n').flat_map(tokens::tokenize).collect()
}

pub fn get_source_file_as_tokens<P>(path: P) -> Vec<String>
where
    P: AsRef<Path> + Debug,
{
    let error_msg = format!("Error reading file {:?}", path);
    let source = fs::read_to_string(path).expect(&error_msg);

    get_source_as_tokens(source)
}

pub fn from_rail_stdlib(rc: &RunConventions) -> Vec<String> {
    let path = rail_lib_path().join("rail-src/stdlib/all.txt");

    if path.is_file() {
        return from_lib_list(path, &RAIL_SOURCE_CONVENTIONS);
    }

    let message = format!(
        "Unable to load stdlib. Wanted to find it at {:?}\nDo you need to run 'railup bootstrap'?",
        path
    );
    rail_machine::log_warn(rc, message);

    vec![]
}

pub fn from_lib_list<P>(path: P, conventions: &SourceConventions) -> Vec<String>
where
    P: AsRef<Path> + Debug,
{
    let path: &Path = path.as_ref();

    let base_dir = path.parent().unwrap();

    fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Unable to load library list file {:?}", path))
        .split('\n')
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|filepath| base_dir.join(filepath).to_string_lossy().to_string())
        .map(|file| {
            if conventions.is_lib(&file) {
                Some(get_source_file_as_tokens(file))
            } else if conventions.is_lib_list(&file) {
                Some(from_lib_list(file, conventions))
            } else {
                None
            }
        })
        .filter(|list| list.is_some())
        .flat_map(|list| list.unwrap())
        .collect::<Vec<_>>()
}
