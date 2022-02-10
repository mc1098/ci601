use assert_cmd::prelude::*;
use std::process::Command;

// We check the --help output in order to confirm that the clap cli is setup correctly.
// Any arguments that are incorrectly will cause clap to panic regardless of the arguments or
// options provided.
// Calling help does not require any application logic so if this tests fails then we know it
// is to do with the clap cli setup code.
//
// Note: In Rust 1.57.0 panicking in constant evaluation was stabilised so clap might soon be able
// to perform these checks at compile time which may invalidate the need for this test.
#[test]
fn check_clap_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("seb")?;

    cmd.arg("--help");
    cmd.assert().success();

    Ok(())
}
