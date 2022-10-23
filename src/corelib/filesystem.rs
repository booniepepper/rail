use std::env;
use std::fs;
use std::path::Path;

use crate::rail_machine::{RailDef, RailType};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state("cd", "FIXME", &[String], &[], |quote| {
            let (path, quote) = quote.pop_string("cd");
            let path = Path::new(&path);
            env::set_current_dir(path).unwrap();
            quote
        }),
        RailDef::on_state("ls", "FIXME", &[], &[Quote], |state| {
            let path = env::current_dir().unwrap();

            let files = fs::read_dir(path).unwrap().filter(|dir| dir.is_ok()).fold(
                state.child(),
                |quote, dir| {
                    let dir = dir.unwrap().file_name().to_string_lossy().to_string();
                    quote.push_string(dir)
                },
            );

            state.push_quote(files)
        }),
        RailDef::on_state("pwd", "FIXME", &[], &[String], |quote| {
            let path = env::current_dir().unwrap().to_string_lossy().to_string();
            quote.push_string(path)
        }),
        RailDef::on_state("dir?", "FIXME", &[String], &[Boolean], |quote| {
            let (path, quote) = quote.pop_string("dir?");
            let path = Path::new(&path);
            quote.push_bool(path.is_dir())
        }),
        RailDef::on_state("file?", "FIXME", &[String], &[Boolean], |quote| {
            let (path, quote) = quote.pop_string("file?");
            let path = Path::new(&path);
            quote.push_bool(path.is_file())
        }),
        RailDef::on_state("mkdir", "FIXME", &[String], &[], |quote| {
            let (path, quote) = quote.pop_string("mkdir");
            let path = Path::new(&path);
            fs::create_dir(path).unwrap();
            quote
        }),
        RailDef::on_state("readf", "FIXME", &[String], &[String], |quote| {
            let (path, quote) = quote.pop_string("readf");
            let path = Path::new(&path);
            let contents = fs::read_to_string(path).unwrap();
            quote.push_string(contents)
        }),
        RailDef::on_state("writef", "FIXME", &[String, String], &[], |quote| {
            let (path, quote) = quote.pop_string("writef");
            let (contents, quote) = quote.pop_string("writef");
            let path = Path::new(&path);
            fs::write(path, contents).unwrap();
            quote
        }),
    ]
}
