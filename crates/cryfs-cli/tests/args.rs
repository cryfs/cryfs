use assert_cmd::Command;
use lazy_static::lazy_static;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::ContainsPredicate;
use std::path::{Path, PathBuf};

// TODO Use indoc! for multiline strings

lazy_static! {
    // Don't use escargot for getting the path of the executable built with same settings as the
    // test was built, because that one is already built by cargo and we don't need to re-build it.
    static ref CRYFS_CMD_PATH_CURRENT: &'static Path = assert_cmd::cargo::cargo_bin!("cryfs");
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

mod no_args {
    use super::*;

    #[test]
    fn no_args() {
        cryfs_cmd()
            .assert()
            .failure()
            .stderr(predicates::str::contains("Usage:"));
    }
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
                "cryfs {}",
                cryfs_config::CRYFS_VERSION,
            )));
    }

    #[test]
    fn show_version_short() {
        cryfs_cmd()
            .arg("-V")
            .assert()
            .success()
            .stderr(predicates::str::contains(format!(
                "cryfs {}",
                cryfs_config::CRYFS_VERSION,
            )));
    }

    #[test]
    fn show_version_long_and_vaultdir_mountdir() {
        cryfs_cmd()
            .args(["--version", "vaultdir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"Error: the argument '--version' cannot be used with other arguments"#,
            ));
    }

    #[test]
    fn show_version_short_and_vaultdir_mountdir() {
        cryfs_cmd()
            .args(["-V", "vaultdir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"Error: the argument '--version' cannot be used with other arguments"#,
            ));
    }

    #[test]
    fn show_version_short_and_ciphers() {
        cryfs_cmd()
            .args(["-V", "--show-ciphers"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"Error: the argument '--version' cannot be used with other arguments"#,
            ));
    }

    #[test]
    fn show_version_long_and_ciphers() {
        cryfs_cmd()
            .args(["--version", "--show-ciphers"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"Error: the argument '--version' cannot be used with other arguments"#,
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
            .stdout(cryfs_config::config::ALL_CIPHERS.join("\n") + "\n");
    }

    #[test]
    fn show_ciphers_and_vaultdir_mountdir() {
        cryfs_cmd()
            .args(["--show-ciphers", "vaultdir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"error: the argument '--show-ciphers' cannot be used with:
  <VAULTDIR>
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
                r#"Error: the argument '--version' cannot be used with other arguments"#,
            ));
    }

    #[test]
    fn show_ciphers_and_version_long() {
        cryfs_cmd()
            .args(["--show-ciphers", "--version"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"Error: the argument '--version' cannot be used with other arguments"#,
            ));
    }
}

mod foreground {
    use super::*;

    mod missing_vaultdir_and_mountdir {
        use super::*;

        #[test]
        fn short() {
            cryfs_cmd()
                .arg("-f")
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }

        #[test]
        fn long() {
            cryfs_cmd()
                .arg("--foreground")
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }
    }

    mod missing_mountdir {
        use super::*;

        #[test]
        fn short_after_vaultdir() {
            cryfs_cmd()
                .args(["vaultdir", "-f"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }

        #[test]
        fn short_before_vaultdir() {
            cryfs_cmd()
                .args(["-f", "vaultdir"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }

        #[test]
        fn long_after_vaultdir() {
            cryfs_cmd()
                .args(["vaultdir", "--foreground"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }

        #[test]
        fn long_before_vaultdir() {
            cryfs_cmd()
                .args(["--foreground", "vaultdir"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }
    }

    // TODO Test -f flag with both vaultdir and mountdir present, i.e. successfully mounts. In different orderings.
}

mod debug_build_warning {
    use super::*;

    fn debug_build_warning() -> ContainsPredicate {
        predicates::str::contains("WARNING! This is a debug build.")
    }

    #[test]
    fn debug_build() {
        cryfs_cmd_debug()
            // TODO Test this by actually mounting a test file system (probably with test scrypt parameters for performance), not with "--version"
            .arg("--version")
            .assert()
            .success()
            .stderr(debug_build_warning());
    }

    #[test]
    fn release_build() {
        cryfs_cmd_release()
            // TODO Test this by actually mounting a test file system (probably with test scrypt parameters for performance), not with "--version"
            .arg("--version")
            .assert()
            .success()
            .stderr(debug_build_warning().not());
    }
}

/// `--daemon` is the sentinel argv flag that the parent cryfs process passes
/// when it re-execs itself as the background daemon child. It is hidden from
/// `--help` and refuses to run unless invoked through the fork+exec spawn
/// path (which sets up inherited pipes on fds 3 and 4).
mod daemon_flag {
    use super::*;

    #[test]
    fn hidden_from_help() {
        // Implementation detail: clap's `hide = true` keeps the flag out of
        // the rendered help. If a future refactor accidentally drops that
        // attribute, the flag becomes discoverable and users could try to
        // invoke it manually.
        cryfs_cmd()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicates::str::contains("--daemon").not());
    }

    #[test]
    fn rejected_when_combined_with_other_args() {
        // clap's `exclusive = true` rejects any combination of `--daemon`
        // with another argument before any of our code runs. This replaces
        // a hand-rolled "argv length must be exactly 2" check.
        cryfs_cmd()
            .args(["--daemon", "--foreground", "/tmp/v", "/tmp/m"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                "the argument '--daemon' cannot be used with",
            ));
    }

    #[test]
    fn rejected_when_invoked_from_shell() {
        // Defensive `fstat(3)`/`fstat(4)` check in
        // `cryfs_runner::run_as_background_daemon`. A curious user running
        // `cryfs --daemon` from a shell has no pipes on fds 3/4, so the
        // daemon refuses to start with a message pointing them at the right
        // mental model. Without this guard, the daemon would silently
        // attempt to deserialize from stdin (or whatever else happens to be
        // open at fd 3) and produce confusing failures.
        cryfs_cmd()
            .arg("--daemon")
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                "internal to cryfs; do not invoke it directly",
            ));
    }
}

// TODO Test that invalid arguments show the usage info (but with an error exit code)
//    - missing vaultdir/mountdir
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
// TODO Test absolute and relative paths work
