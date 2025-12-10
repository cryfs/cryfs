// cryfs-cli only makes sense if either fuser or fuse_mt is enabled
#![cfg(any(feature = "fuser", feature = "fuse_mt"))]

use assert_cmd::Command;
use lazy_static::lazy_static;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::ContainsPredicate;
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
                cryfs_filesystem::CRYFS_VERSION,
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
                cryfs_filesystem::CRYFS_VERSION,
            )));
    }

    #[test]
    fn show_version_long_and_basedir_mountdir() {
        cryfs_cmd()
            .args(["--version", "basedir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"Error: the argument '--version' cannot be used with other arguments"#,
            ));
    }

    #[test]
    fn show_version_short_and_basedir_mountdir() {
        cryfs_cmd()
            .args(["-V", "basedir", "mountdir"])
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
            .stdout(cryfs_filesystem::config::ALL_CIPHERS.join("\n") + "\n");
    }

    #[test]
    fn show_ciphers_and_basedir_mountdir() {
        cryfs_cmd()
            .args(["--show-ciphers", "basedir", "mountdir"])
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                r#"error: the argument '--show-ciphers' cannot be used with:
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

    mod missing_basedir_and_mountdir {
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
        fn short_after_basedir() {
            cryfs_cmd()
                .args(["basedir", "-f"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }

        #[test]
        fn short_before_basedir() {
            cryfs_cmd()
                .args(["-f", "basedir"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }

        #[test]
        fn long_after_basedir() {
            cryfs_cmd()
                .args(["basedir", "--foreground"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }

        #[test]
        fn long_before_basedir() {
            cryfs_cmd()
                .args(["--foreground", "basedir"])
                .assert()
                .failure()
                .stderr(predicates::str::contains("Usage:"));
        }
    }

    // TODO Test -f flag with both basedir and mountdir present, i.e. successfully mounts. In different orderings.
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
// TODO Test absolute and relative paths work
