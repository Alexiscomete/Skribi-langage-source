use std::process::Command;

use insta_cmd::assert_cmd_snapshot;
use insta_cmd::get_cargo_bin;

macro_rules! cli {
    () => {
        Command::new(get_cargo_bin("skribi"))
    };
}

#[test]
fn test_build_add() {
    assert_cmd_snapshot!(
        cli!()
            .arg("build")
            .arg("resources/test_programs/exit_add.skrb")
            .arg("-o")
            .arg(".skribi/add.out")
    );
    assert_cmd_snapshot!("run add out", Command::new("./.skribi/add.out"))
}

#[test]
fn test_pretty_add() {
    assert_cmd_snapshot!(
        cli!()
            .arg("pretty")
            .arg("resources/test_programs/exit_add.skrb")
    )
}
