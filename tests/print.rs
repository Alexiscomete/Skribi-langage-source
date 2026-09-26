use std::process::Command;

use insta_cmd::assert_cmd_snapshot;
use insta_cmd::get_cargo_bin;

macro_rules! cli {
    () => {
        Command::new(get_cargo_bin("skribi"))
    };
}

#[test]
fn test_build_println() {
    assert_eq!(
        cli!()
            .arg("build")
            .arg("resources/test_programs/print.skrb")
            .arg("-o")
            .arg(".skribi/print.out")
            .status()
            .unwrap()
            .code()
            .unwrap(),
        0
    );
    assert_cmd_snapshot!("run print out", Command::new("./.skribi/print.out"))
}
