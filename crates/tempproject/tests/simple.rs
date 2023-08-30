use indoc::indoc;
use tempproject::TempProjectBuilder;

#[test]
fn sucess() {
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
    let project = project.build().unwrap();

    project
        .run()
        .assert()
        .success()
        .stdout(predicates::str::contains("Hello, world!\n"));
    assert!(project.run().assert().try_failure().is_err());
}

#[test]
fn run_failure() {
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
    let project = project.build().unwrap();

    project
        .run()
        .assert()
        .failure()
        .stderr(predicates::str::contains("Hello, world!\n"));
    assert!(project.run().assert().try_success().is_err());
}

#[test]
fn build_failure() {
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
    let project = project.build().unwrap();

    project
        .run()
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "cannot find function `nonexisting_func` in this scope",
        ));
    assert!(project.run().assert().try_success().is_err());
}
