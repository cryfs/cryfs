use cryfs_version::Version;
use tempproject::{TempProject, TempProjectBuilder};

const OUR_CRATE_PATH: &str = env!("CARGO_MANIFEST_DIR");
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
            let project = make_version_test_project(&gitversion.to_string());
            run_version_test_project_expect_success(&project, &gitversion);
        } else {
            // If we're not in git, just test with an arbitrary version
            let project = make_version_test_project("0.1.0-alpha");
            run_version_test_project_expect_success(
                &project,
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
            let project = make_version_test_project("1.2.3");
            run_version_test_project_expect_build_error(
                &project,
                "Version mismatch: The version in the git tag does not match the version in Cargo.toml",
            );
        }
    }

    fn make_version_test_project(version: &str) -> TempProject {
        let project = TempProjectBuilder::new().unwrap();
        let project = project.cargo(format!(
            r#"
[package]
authors = ["Sebastian Messmer <messmer@cryfs.org>"]
edition = "2024"
name = "cryfs-version-test"
version = "{version}"

[workspace]

[dependencies]
cryfs-version = {{path = "{OUR_CRATE_PATH}"}}
serde_json = "^1.0.96"
            "#
        ));
        let project = project.main(stringify!(
            fn main() {
                println!(
                    "{}",
                    serde_json::to_string(&cryfs_version::package_version!().version()).unwrap()
                );
            }
        ));

        project.build().unwrap()
    }

    fn run_version_test_project_expect_success(project: &TempProject, expected_version: &Version) {
        let run = project.run_debug().unwrap().assert().success();
        let output = run.get_output();

        let actual_version: Version = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(expected_version, &actual_version);
    }

    fn run_version_test_project_expect_build_error(project: &TempProject, expected_error: &str) {
        let error = project.run_debug().unwrap_err();
        assert!(error.stderr.unwrap().contains(expected_error));
    }
}

mod test_macro_cargo_version {
    use super::*;

    #[test]
    fn version_0_1_0_alpha() {
        let project = make_version_test_project("0.1.0-alpha");
        run_version_test_project(
            &project,
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
        let project = make_version_test_project("1.2.3");
        run_version_test_project(
            &project,
            &Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            },
        );
    }

    fn make_version_test_project(version: &str) -> TempProject {
        let project = TempProjectBuilder::new().unwrap();
        let project = project.cargo(format!(
            r#"
[package]
authors = ["Sebastian Messmer <messmer@cryfs.org>"]
edition = "2024"
name = "cryfs-version-test"
version = "{version}"

[workspace]

[dependencies]
cryfs-version = {{path = "{OUR_CRATE_PATH}"}}
serde_json = "^1.0.96"
            "#
        ));
        let project = project.main(stringify!(
            fn main() {
                println!(
                    "{}",
                    serde_json::to_string(&cryfs_version::cargo_version!()).unwrap()
                );
            }
        ));

        project.build().unwrap()
    }

    fn run_version_test_project(project: &TempProject, expected_version: &Version) {
        let run = project.run_debug().unwrap().assert().success();
        let output = run.get_output();

        let actual_version: Version = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(expected_version, &actual_version);
    }
}

mod test_macro_assert_cargo_version_equals_git_version {
    use super::*;

    #[test]
    fn matches_git_version() {
        if let Some(our_gitversion) = OUR_GITVERSION {
            let project = make_version_test_project(&our_gitversion.to_string());
            run_version_test_project_expect_success(&project);
        }
    }

    #[test]
    fn doesnt_match_git_version() {
        if let Some(_our_gitversion) = OUR_GITVERSION {
            let project = make_version_test_project("0.1.0");
            run_version_test_project_expect_build_error(
                &project,
                "Version mismatch: The version in the git tag does not match the version in Cargo.toml",
            );
        }
    }

    fn make_version_test_project(version: &str) -> TempProject {
        let project = TempProjectBuilder::new().unwrap();
        let project = project.cargo(format!(
            r#"
[package]
authors = ["Sebastian Messmer <messmer@cryfs.org>"]
edition = "2024"
name = "cryfs-version-test"
version = "{version}"

[workspace]

[dependencies]
cryfs-version = {{path = "{OUR_CRATE_PATH}"}}
serde_json = "^1.0.96"
            "#
        ));
        let project = project.main(stringify!(
            cryfs_version::assert_cargo_version_equals_git_version!();

            fn main() {}
        ));

        project.build().unwrap()
    }

    fn run_version_test_project_expect_success(project: &TempProject) {
        project.run_debug().unwrap().assert().success();
    }

    fn run_version_test_project_expect_build_error(project: &TempProject, expected_error: &str) {
        let error = project.run_debug().unwrap_err();
        assert!(error.stderr.unwrap().contains(expected_error));
    }
}
