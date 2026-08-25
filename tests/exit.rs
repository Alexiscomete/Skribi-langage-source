use std::process::Command;

use insta_cmd::assert_cmd_snapshot;
use insta_cmd::get_cargo_bin;

macro_rules! cli {
    () => {
        Command::new(get_cargo_bin("skribi"))
    };
}

#[test]
fn test_fixed_exit_build() {
    assert_cmd_snapshot!(
        cli!()
            .arg("build")
            .arg("resources/test_programs/fixed_exit.skrb")
    )
}

#[test]
fn test_fixed_exit_pretty() {
    assert_cmd_snapshot!(
        cli!()
            .arg("pretty")
            .arg("resources/test_programs/fixed_exit.skrb")
    )
}

#[test]
fn test_exit_build() {
    assert_cmd_snapshot!(cli!().arg("build").arg("resources/test_programs/exit.skrb"))
}

#[test]
fn test_exit_pretty() {
    assert_cmd_snapshot!(
        cli!()
            .arg("pretty")
            .arg("resources/test_programs/exit.skrb")
    )
}
