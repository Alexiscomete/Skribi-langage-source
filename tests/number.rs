use std::process::Command;

use insta_cmd::assert_cmd_snapshot;
use insta_cmd::get_cargo_bin;

macro_rules! cli {
    () => {
        Command::new(get_cargo_bin("skribi"))
    };
}

#[test]
fn test_build_number() {
    assert_cmd_snapshot!(
        cli!()
            .arg("build")
            .arg("resources/test_programs/number.skrb")
    )
}

#[test]
fn test_pretty_number() {
    assert_cmd_snapshot!(
        cli!()
            .arg("pretty")
            .arg("resources/test_programs/number.skrb")
    )
}
