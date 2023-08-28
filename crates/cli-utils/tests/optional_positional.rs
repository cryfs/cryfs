use assert_cmd::Command;
use lazy_static::lazy_static;
use predicates::boolean::PredicateBooleanExt;
use std::ffi::OsString;
use test_binary::build_test_binary;

const VERSION_MESSAGE: &str = "my-cli-name 1.2.3";
const MAIN_MESSAGE: &str = "cryfs-cli-utils-testbins-optional-positional:main";

lazy_static! {
    static ref CMD_PATH_CURRENT: OsString =
        build_test_binary("cryfs-cli-utils-testbins-optional-positional", "testbins").unwrap();
}

fn cmd() -> Command {
    Command::new(&*CMD_PATH_CURRENT)
}

#[test]
fn with_positional_arg() {
    cmd()
        .arg("some_value")
        .assert()
        .success()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stdout(predicates::str::contains(format!(
            "{}:Some(\"some_value\")",
            MAIN_MESSAGE,
        )));
}

#[test]
fn missing_positional_arg() {
    cmd()
        .assert()
        .success()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stdout(predicates::str::contains(format!("{}:None", MAIN_MESSAGE,)));
}

#[test]
fn version_flag_long() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stdout(predicates::str::contains(MAIN_MESSAGE).not());
}

#[test]
fn version_flag_short() {
    cmd()
        .arg("-V")
        .assert()
        .success()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stdout(predicates::str::contains(MAIN_MESSAGE).not());
}

#[test]
fn version_flag_bad() {
    cmd()
        .arg("--version=bad")
        .assert()
        .failure()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stderr(predicates::str::contains(
            "error: unexpected value 'bad' for '--version' found; no more were expected",
        ));
}

#[test]
fn help_flag_long() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stdout(predicates::str::contains(
            "Usage: cryfs-cli-utils-testbins-optional-positional [OPTIONAL_POSITIONAL]",
        ))
        .stdout(predicates::str::contains(
            "Arguments:\n  [OPTIONAL_POSITIONAL]",
        ))
        .stdout(predicates::str::contains("-V, --version"))
        .stdout(predicates::str::contains("-h, --help"))
        .stdout(predicates::str::contains(MAIN_MESSAGE).not());
}

#[test]
fn help_flag_short() {
    cmd()
        .arg("-h")
        .assert()
        .success()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stdout(predicates::str::contains(
            "Usage: cryfs-cli-utils-testbins-optional-positional [OPTIONAL_POSITIONAL]",
        ))
        .stdout(predicates::str::contains(
            "Arguments:\n  [OPTIONAL_POSITIONAL]",
        ))
        .stdout(predicates::str::contains("-V, --version"))
        .stdout(predicates::str::contains("-h, --help"))
        .stdout(predicates::str::contains(MAIN_MESSAGE).not());
}

#[test]
fn help_flag_bad() {
    cmd()
        .arg("--help=bad")
        .assert()
        .failure()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stderr(predicates::str::contains(
            "error: unexpected value 'bad' for '--help' found; no more were expected",
        ));
}

// TODO Test `--version` and `--help` combination
// TODO Test combination of the positional argument and `--version` or `--help`
