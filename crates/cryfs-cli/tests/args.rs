use assert_cmd::Command;
use lazy_static::lazy_static;
use predicates::boolean::PredicateBooleanExt;
use std::path::PathBuf;

// TODO Use indoc! for multiline strings

lazy_static! {
    // Don't use escargot for getting the path of the executable built with same settings as the
    // test was built, because that one is already built by cargo and we don't need to re-build it.
    static ref CRYFS_CMD_PATH_CURRENT: PathBuf = assert_cmd::cargo::cargo_bin("cryfs");
    static ref CRYFS_CMD_PATH_DEBUG: PathBuf = escargot::CargoBuild::new()
        .current_target()
        .bin("cryfs")
        .run()
        .unwrap()
        .path()
        .to_owned();
    static ref CRYFS_CMD_PATH_RELEASE: PathBuf = escargot::CargoBuild::new()
        .current_target()
        .release()
        .bin("cryfs")
        .run()
        .unwrap()
        .path()
        .to_owned();
}

fn cryfs_cmd() -> Command {
    Command::new(&*CRYFS_CMD_PATH_CURRENT)
}

fn cryfs_cmd_debug() -> Command {
    Command::new(&*CRYFS_CMD_PATH_DEBUG)
}

fn cryfs_cmd_release() -> Command {
    Command::new(&*CRYFS_CMD_PATH_RELEASE)
}

mod help {
    use super::*;

    #[test]
    fn show_help() {
        cryfs_cmd()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicates::str::contains("Usage"));
    }
}

mod version {
    use super::*;

    #[test]
    fn show_version_long() {
        cryfs_cmd()
            .arg("--version")
            .assert()
            .success()
            .stderr(predicates::str::contains(format!(
                "CryFS Version {}",
                cryfs_cryfs::CRYFS_VERSION,
            )));
    }

    #[test]
    fn show_version_short() {
        cryfs_cmd()
            .arg("-V")
            .assert()
            .success()
            .stderr(predicates::str::contains(format!(
                "CryFS Version {}",
                cryfs_cryfs::CRYFS_VERSION,
            )));
    }

    #[test]
    fn show_version_long_and_basedir_mountdir() {
        cryfs_cmd()
            .args(["--version", "basedir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"
error: the argument '--version' cannot be used with:
  <BASEDIR>
  <MOUNTDIR>
"#,
            ));
    }

    #[test]
    fn show_version_short_and_basedir_mountdir() {
        cryfs_cmd()
            .args(["-V", "basedir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"
error: the argument '--version' cannot be used with:
  <BASEDIR>
  <MOUNTDIR>
"#,
            ));
    }

    #[test]
    fn show_version_short_and_ciphers() {
        cryfs_cmd()
            .args(["-V", "--show-ciphers"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"error: the argument '--version' cannot be used with '--show-ciphers'"#,
            ));
    }

    #[test]
    fn show_version_long_and_ciphers() {
        cryfs_cmd()
            .args(["--version", "--show-ciphers"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"error: the argument '--version' cannot be used with '--show-ciphers'"#,
            ));
    }
}

mod show_ciphers {
    use super::*;

    #[test]
    fn show_ciphers() {
        cryfs_cmd()
            .arg("--show-ciphers")
            .assert()
            .success()
            .stdout(cryfs_cryfs::config::ALL_CIPHERS.join("\n") + "\n");
    }

    #[test]
    fn show_ciphers_and_basedir_mountdir() {
        cryfs_cmd()
            .args(["--show-ciphers", "basedir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"
error: the argument '--show-ciphers' cannot be used with:
  <BASEDIR>
  <MOUNTDIR>
"#,
            ));
    }

    #[test]
    fn show_ciphers_and_version_short() {
        cryfs_cmd()
            .args(["--show-ciphers", "-V"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"error: the argument '--show-ciphers' cannot be used with '--version'"#,
            ));
    }

    #[test]
    fn show_ciphers_and_version_long() {
        cryfs_cmd()
            .args(["--show-ciphers", "--version"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"error: the argument '--show-ciphers' cannot be used with '--version'"#,
            ));
    }
}

mod debug_build_warning {
    use super::*;

    #[test]
    fn debug_build() {
        cryfs_cmd_debug()
            // TODO Test this by actually mounting a test file system (probably with test scrypt parameters for performance), not with "--version"
            .arg("--version")
            .assert()
            .success()
            .stderr(predicates::str::contains("WARNING! This is a debug build."));
    }

    #[test]
    fn release_build() {
        cryfs_cmd_release()
            // TODO Test this by actually mounting a test file system (probably with test scrypt parameters for performance), not with "--version"
            .arg("--version")
            .assert()
            .success()
            .stderr(predicates::str::contains("WARNING! This is a debug build.").not());
    }
}

// TODO Test that invalid arguments show the usage info (but with an error exit code)
//    - missing basedir/mountdir
//    - ...
// TODO Test that help shows environment var info
// TODO Test cli shows version info when mounting a file system
// TODO Test update checks
//      and outputs:
//      - Automatic checking for security vulnerabilities and updates is disabled.
//      - Automatic checking for security vulnerabilities and updates is disabled in noninteractive mode.
// TODO Test gitinfo warnings
//  - WARNING! This is a development version based on git commit {}. Please don't use in production.
//  - WARNING! There were uncommitted changes in the repository when building this version.
//  - WARNING! This is a prerelease version. Please backup your data frequently!
