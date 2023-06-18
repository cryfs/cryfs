use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;

use cryfs_version::Version;

const OUR_GITVERSION: Option<Version> = match cryfs_version::GITINFO {
    None => None,
    Some(gitinfo) => match gitinfo.tag_info {
        Some(tag_info) => Some(konst::unwrap_ctx!(Version::parse_const(tag_info.tag))),
        None => None,
    },
};

mod test_macro_package_version {
    use super::*;

    #[test]
    fn matches_git_version() {
        if let Some(gitversion) = OUR_GITVERSION {
            let project_dir = make_version_test_project(&gitversion.to_string());
            run_version_test_project_expect_success(project_dir.path(), &gitversion);
        } else {
            // If we're not in git, just test with an arbitrary version
            let project_dir = make_version_test_project("0.1.0-alpha");
            run_version_test_project_expect_success(
                project_dir.path(),
                &Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                    prerelease: Some("alpha"),
                },
            );
        }
    }

    #[test]
    fn doesnt_match_git_version() {
        if let Some(_our_gitversion) = OUR_GITVERSION {
            let project_dir = make_version_test_project("1.2.3");
            run_version_test_project_expect_error(
                project_dir.path(),
                "Version mismatch: The version in the git tag does not match the version in Cargo.toml",
            );
        }
    }

    fn make_version_test_project(version: &str) -> TempDir {
        let dir = TempDir::new("cryfs-version-test").unwrap();
        let dir_path = dir.path();
        let path_to_cryfs_version_crate = env!("CARGO_MANIFEST_DIR");

        create_file(
            &dir_path.join("Cargo.toml"),
            &format!(
                r#"
[package]
authors = ["Sebastian Messmer <messmer@cryfs.org>"]
edition = "2021"
name = "cryfs-version-test"
version = "{version}"

[workspace]

[dependencies]
cryfs-version = {{path = "{path_to_cryfs_version_crate}"}}
serde_json = "^1.0.96"
            "#
            ),
        );

        create_file(
            &dir_path.join("src/main.rs"),
            r#"
    fn main() {
        println!("{}", serde_json::to_string(&cryfs_version::package_version!().version()).unwrap());
    }
            "#,
        );

        dir
    }

    fn run_version_test_project_expect_success(project_dir: &Path, expected_version: &Version) {
        let output = check_run_process(
            Command::new(env!("CARGO"))
                .arg("run")
                .current_dir(project_dir),
        );

        let actual_version: Version = serde_json::from_str(&output).unwrap();
        assert_eq!(expected_version, &actual_version);
    }

    fn run_version_test_project_expect_error(project_dir: &Path, expected_error: &str) {
        run_process_expect_error(
            Command::new(env!("CARGO"))
                .arg("run")
                .current_dir(project_dir),
            expected_error,
        );
    }
}

mod test_macro_cargo_version {
    use super::*;

    #[test]
    fn version_0_1_0_alpha() {
        let project_dir = make_version_test_project("0.1.0-alpha");
        run_version_test_project(
            project_dir.path(),
            &Version {
                major: 0,
                minor: 1,
                patch: 0,
                prerelease: Some("alpha"),
            },
        );
    }

    #[test]
    fn version_1_2_3() {
        let project_dir = make_version_test_project("1.2.3");
        run_version_test_project(
            project_dir.path(),
            &Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            },
        );
    }

    fn make_version_test_project(version: &str) -> TempDir {
        let dir = TempDir::new("cryfs-version-test").unwrap();
        let dir_path = dir.path();
        let path_to_cryfs_version_crate = env!("CARGO_MANIFEST_DIR");

        create_file(
            &dir_path.join("Cargo.toml"),
            &format!(
                r#"
[package]
authors = ["Sebastian Messmer <messmer@cryfs.org>"]
edition = "2021"
name = "cryfs-version-test"
version = "{version}"

[workspace]

[dependencies]
cryfs-version = {{path = "{path_to_cryfs_version_crate}"}}
serde_json = "^1.0.96"
            "#
            ),
        );

        create_file(
            &dir_path.join("src/main.rs"),
            r#"
                fn main() {
                    println!("{}", serde_json::to_string(&cryfs_version::cargo_version!()).unwrap());
                }
            "#,
        );

        dir
    }

    fn run_version_test_project(project_dir: &Path, expected_version: &Version) {
        let output = check_run_process(
            Command::new(env!("CARGO"))
                .arg("run")
                .current_dir(project_dir),
        );

        let actual_version: Version = serde_json::from_str(&output).unwrap();
        assert_eq!(expected_version, &actual_version);
    }
}

mod test_macro_assert_cargo_version_equals_git_version {
    use super::*;

    #[test]
    fn matches_git_version() {
        if let Some(our_gitversion) = OUR_GITVERSION {
            let project_dir = make_version_test_project(&our_gitversion.to_string());
            run_version_test_project_expect_success(project_dir.path());
        }
    }

    #[test]
    fn doesnt_match_git_version() {
        if let Some(_our_gitversion) = OUR_GITVERSION {
            let project_dir = make_version_test_project("0.1.0");
            run_version_test_project_expect_error(project_dir.path(), "Version mismatch: The version in the git tag does not match the version in Cargo.toml");
        }
    }

    fn make_version_test_project(version: &str) -> TempDir {
        let dir = TempDir::new("cryfs-version-test").unwrap();
        let dir_path = dir.path();
        let path_to_cryfs_version_crate = env!("CARGO_MANIFEST_DIR");

        create_file(
            &dir_path.join("Cargo.toml"),
            &format!(
                r#"
[package]
authors = ["Sebastian Messmer <messmer@cryfs.org>"]
edition = "2021"
name = "cryfs-version-test"
version = "{version}"

[workspace]

[dependencies]
cryfs-version = {{path = "{path_to_cryfs_version_crate}"}}
serde_json = "^1.0.96"
            "#
            ),
        );

        create_file(
            &dir_path.join("src/main.rs"),
            r#"
                cryfs_version::assert_cargo_version_equals_git_version!();

                fn main() {}
            "#,
        );

        dir
    }

    fn run_version_test_project_expect_success(project_dir: &Path) {
        check_run_process(
            Command::new(env!("CARGO"))
                .arg("run")
                .current_dir(project_dir),
        );
    }

    fn run_version_test_project_expect_error(project_dir: &Path, expected_error: &str) {
        run_process_expect_error(
            Command::new(env!("CARGO"))
                .arg("run")
                .current_dir(project_dir),
            expected_error,
        );
    }
}

fn create_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    File::create(path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
}

fn check_run_process(cmd: &mut Command) -> String {
    let output = cmd.output().unwrap();
    if !output.status.success() {
        panic!(
            "Command {:?} failed with status {:?} and stdin:\n{}\n\nstderr:\n{}",
            cmd,
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_process_expect_error(cmd: &mut Command, expected_error: &str) {
    let output = cmd.output().unwrap();
    if output.status.success() {
        panic!("Expected command {:?} to fail but it succeeded", cmd,);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.contains(expected_error) {
        panic!(
            "Expected command {:?} to fail with error '{}' but it failed with '{}' instead",
            cmd, expected_error, stderr
        );
    }
}
