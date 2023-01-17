mod rail_runner;
use rail_runner::{railsh_run_file, RailRunResult};

fn run_problem(n: &str) -> RailRunResult {
    let filename = format!("tests/project_euler/problem-{}.rail", n);
    railsh_run_file(&filename)
}

#[test]
pub fn problem_01() {
    let res = run_problem("01");
    assert_eq!("", res.stderr);
    assert_eq!("233168", res.stdout.trim());
}

#[test]
pub fn problem_02a() {
    let res = run_problem("02a");
    assert_eq!("", res.stderr);
    assert_eq!("4613732", res.stdout.trim());
}

#[test]
pub fn problem_02b() {
    let res = run_problem("02b");
    assert_eq!("", res.stderr);
    assert_eq!("4613732", res.stdout.trim());
}

#[test]
pub fn problem_03() {
    let res = run_problem("03");
    assert_eq!("", res.stderr);
    assert_eq!("6857", res.stdout.trim());
}

#[test]
pub fn problem_04() {
    let res = run_problem("04");
    assert_eq!("", res.stderr);
    assert_eq!("906609", res.stdout.trim());
}
