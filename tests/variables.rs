use std::process::Command;

use insta_cmd::assert_cmd_snapshot;
use insta_cmd::get_cargo_bin;

macro_rules! cli {
    () => {
        Command::new(get_cargo_bin("skribi"))
    };
}

#[test]
fn test_build_variables() {
    assert_eq!(
        cli!()
            .arg("build")
            .arg("resources/test_programs/variables.skrb")
            .arg("-o")
            .arg(".skribi/variables.out")
            .status()
            .unwrap()
            .code()
            .unwrap(),
        0
    );
    assert_cmd_snapshot!("run variables out", Command::new("./.skribi/variables.out"))
}

#[test]
fn test_pretty_variables() {
    assert_eq!(
        cli!()
            .arg("pretty")
            .arg("resources/test_programs/variables.skrb")
            .status()
            .unwrap()
            .code()
            .unwrap(),
        0
    );
}
