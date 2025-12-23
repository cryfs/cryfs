use assert_cmd::Command;
use indoc::indoc;
use tempproject::{TempProject, TempProjectBuilder};

fn project_success() -> TempProject {
    let project = TempProjectBuilder::new().unwrap();
    let project = project.cargo(indoc!(
        r#"
            [package]
            name = "simple"
            version = "0.1.0"
        "#
    ));
    let project = project.main(stringify!(
        fn main() {
            println!("Hello, world!");
        }
    ));
    project.build().unwrap()
}

fn project_failing_at_build_time() -> TempProject {
    let project = TempProjectBuilder::new().unwrap();
    let project = project.cargo(indoc!(
        r#"
            [package]
            name = "simple"
            version = "0.1.0"
        "#
    ));
    let project = project.main(stringify!(
        fn main() {
            nonexisting_func()
        }
    ));
    project.build().unwrap()
}

fn project_failing_at_run_time() -> TempProject {
    let project = TempProjectBuilder::new().unwrap();
    let project = project.cargo(indoc!(
        r#"
            [package]
            name = "simple"
            version = "0.1.0"
        "#
    ));
    let project = project.main(stringify!(
        use std::process::exit;

        fn main() {
            println!("Hello, world!");
            exit(1);
        }
    ));
    project.build().unwrap()
}

fn expect_build_debug_success(project: &TempProject) {
    let executable = project.build_debug().unwrap();
    Command::new(executable)
        .assert()
        .stdout(predicates::str::contains("Hello, world!\n"));
}

fn expect_build_release_success(project: &TempProject) {
    let executable = project.build_release().unwrap();
    Command::new(executable)
        .assert()
        .stdout(predicates::str::contains("Hello, world!\n"));
}

fn expect_run_debug_success(project: &TempProject) {
    project
        .run_debug()
        .unwrap()
        .assert()
        .success()
        .stdout(predicates::str::contains("Hello, world!\n"));
    assert!(project.run_debug().unwrap().assert().try_failure().is_err());
}

fn expect_run_release_success(project: &TempProject) {
    project
        .run_release()
        .unwrap()
        .assert()
        .success()
        .stdout(predicates::str::contains("Hello, world!\n"));
    assert!(
        project
            .run_release()
            .unwrap()
            .assert()
            .try_failure()
            .is_err()
    );
}

fn expect_build_debug_fails(project: &TempProject) {
    project.build_debug().unwrap_err();
}

fn expect_build_release_fails(project: &TempProject) {
    project.build_release().unwrap_err();
}

fn expect_run_debug_fails_at_build_time(project: &TempProject) {
    let err = project.run_debug().unwrap_err();
    assert!(
        err.stderr
            .as_ref()
            .unwrap()
            .contains("cannot find function `nonexisting_func` in this scope")
    );
}

fn expect_run_release_fails_at_build_time(project: &TempProject) {
    let err = project.run_release().unwrap_err();
    assert!(
        err.stderr
            .as_ref()
            .unwrap()
            .contains("cannot find function `nonexisting_func` in this scope")
    );
}

fn expect_run_debug_fails_at_run_time(project: &TempProject) {
    project
        .run_debug()
        .unwrap()
        .assert()
        .failure()
        .stdout(predicates::str::contains("Hello, world!\n"));
    assert!(project.run_debug().unwrap().assert().try_success().is_err());
}

fn expect_run_release_fails_at_run_time(project: &TempProject) {
    project
        .run_release()
        .unwrap()
        .assert()
        .failure()
        .stdout(predicates::str::contains("Hello, world!\n"));
    assert!(
        project
            .run_release()
            .unwrap()
            .assert()
            .try_success()
            .is_err()
    );
}

mod build_debug {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();
        expect_build_debug_success(&project);
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();
        expect_build_debug_success(&project);
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();
        expect_build_debug_fails(&project);
    }
}

mod run_debug {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();
        expect_run_debug_success(&project);
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();
        expect_run_debug_fails_at_run_time(&project);
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();
        expect_run_debug_fails_at_build_time(&project);
    }
}

mod build_then_run_debug {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();
        expect_build_debug_success(&project);
        expect_run_debug_success(&project);
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();
        expect_build_debug_success(&project);
        expect_run_debug_fails_at_run_time(&project);
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();
        expect_build_debug_fails(&project);
        expect_run_debug_fails_at_build_time(&project);
    }
}

mod build_release {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();
        expect_build_release_success(&project);
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();
        expect_build_release_success(&project);
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();
        expect_build_release_fails(&project);
    }
}

mod run_release {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();
        expect_run_release_success(&project);
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();
        expect_run_release_fails_at_run_time(&project);
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();
        expect_run_release_fails_at_build_time(&project);
    }
}

mod build_then_run_release {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();
        expect_build_release_success(&project);
        expect_run_release_success(&project);
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();
        expect_build_release_success(&project);
        expect_run_release_fails_at_run_time(&project);
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();
        expect_build_release_fails(&project);
        expect_run_release_fails_at_build_time(&project);
    }
}

mod edge_cases {
    use super::*;

    #[test]
    #[should_panic(expected = "You must call TempProjectBuilder::cargo")]
    fn build_without_cargo_panics() {
        let builder = TempProjectBuilder::new().unwrap();
        let builder = builder.main("fn main() {}");
        let _ = builder.build();
    }

    #[test]
    #[should_panic(expected = "You must call TempProjectBuilder::main")]
    fn build_without_main_panics() {
        let builder = TempProjectBuilder::new().unwrap();
        let builder = builder.cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"");
        let _ = builder.build();
    }

    #[test]
    #[should_panic(expected = "You must call TempProjectBuilder::cargo")]
    fn build_without_both_panics_on_cargo_first() {
        let builder = TempProjectBuilder::new().unwrap();
        let _ = builder.build();
    }
}
