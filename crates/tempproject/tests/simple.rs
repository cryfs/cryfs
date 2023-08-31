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
            eprintln!("Hello, world!");
            exit(1);
        }
    ));
    project.build().unwrap()
}

mod run_debug {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();

        project
            .run_debug()
            .unwrap()
            .assert()
            .success()
            .stdout(predicates::str::contains("Hello, world!\n"));
        assert!(project.run_debug().unwrap().assert().try_failure().is_err());
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();

        project
            .run_debug()
            .unwrap()
            .assert()
            .failure()
            .stderr(predicates::str::contains("Hello, world!\n"));
        assert!(project.run_debug().unwrap().assert().try_success().is_err());
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();

        let err = project.run_debug().unwrap_err();
        assert!(err
            .stderr
            .as_ref()
            .unwrap()
            .contains("cannot find function `nonexisting_func` in this scope"));
    }
}

mod build_debug {
    use super::*;

    #[test]
    fn success() {
        let project = project_success();

        let executable = project.build_debug().unwrap();
        Command::new(executable)
            .assert()
            .success()
            .stdout(predicates::str::contains("Hello, world!\n"));
    }

    #[test]
    fn failure_at_run_time() {
        let project = project_failing_at_run_time();

        let executable = project.build_debug().unwrap();
        Command::new(executable)
            .assert()
            .failure()
            .stderr(predicates::str::contains("Hello, world!\n"));
    }

    #[test]
    fn failure_at_build_time() {
        let project = project_failing_at_build_time();

        project.build_debug().unwrap_err();
    }
}
