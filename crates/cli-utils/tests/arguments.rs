use indoc::indoc;
use lazy_static::lazy_static;
use predicates::boolean::PredicateBooleanExt;
use rstest::rstest;
use tempproject::{TempProject, TempProjectBuilder};

const VERSION_MESSAGE: &str = "my-cli-name 1.2.3";
const MAIN_MESSAGE: &str = "my-testbin:main";

const CARGO_TOML: &str = concat!(
    indoc!(
        r#"
            [package]
            name = "my-testbin"
            version = "0.0.1"
            edition = "2024"
            
            [dependencies]
            cryfs-cli-utils = {path = ""#
    ),
    env!("CARGO_MANIFEST_DIR"),
    r#""}"#,
);

/// [TestConfig] defines how to build a test binary with given arguments and main code.
struct TestConfig {
    args: &'static str,
    main: &'static str,
}

impl TestConfig {
    /// Create a cargo project with a binary following our specification
    pub fn project(&self) -> TempProject {
        let main_use = stringify!(
            use cryfs_cli_utils::{
                reexports_for_tests::{
                    anyhow::Result,
                    clap::{self, Args},
                    cryfs_version::{Version, VersionInfo},
                },
                run, Application, Environment, CliError,
            };
            use std::process::ExitCode;
        );
        let main_cli = stringify!(
            struct Cli {
                args: MyArgs,
            }
        );
        let main_app = stringify!(
            type ConcreteArgs = MyArgs;
            const NAME: &'static str = "my-cli-name";
            const VERSION: VersionInfo<'static, 'static, 'static> = VersionInfo::new(
                Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                None,
            );

            fn new(args: MyArgs, _env: Environment) -> Result<Self, CliError> {
                Ok(Self { args })
            }
        );
        let main_main = stringify!(
            fn main() -> ExitCode {
                run::<Cli>()
            }
        );

        TempProjectBuilder::new()
            .unwrap()
            .cargo(CARGO_TOML)
            .main(format!(
                indoc!(
                    r#"
                {main_use}

                {self_args}

                {main_cli}

                impl Application for Cli {{
                    {main_app}

                    {self_main}
                }}

                {main_main}
                "#
                ),
                main_use = main_use,
                main_cli = main_cli,
                main_app = main_app,
                main_main = main_main,
                self_args = self.args,
                self_main = self.main,
            ))
            .build()
            .unwrap()
    }
}

struct TestProject {
    project: TempProject,
    expected_usage_header: &'static str,
    expected_usage_line: &'static str,
}

impl TestProject {
    pub fn expect_help_message(&self, run: assert_cmd::assert::Assert) {
        run.stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(self.expected_usage_header))
            .stdout(predicates::str::contains("-V, --version"))
            .stdout(predicates::str::contains("-h, --help"))
            .stdout(predicates::str::contains(self.expected_usage_line))
            .stdout(predicates::str::contains(MAIN_MESSAGE).not());
    }
}

lazy_static! {
    static ref PROJECT_NOARGS: TestProject = TestProject {
        project: TestConfig {
            args: stringify!(
                #[derive(Args, Debug)]
                struct MyArgs {}
            ),
            main: stringify!(
                fn main(self) -> Result<(), CliError> {
                    println!("my-testbin:main");
                    Ok(())
                }
            ),
        }
        .project(),
        expected_usage_header: "Usage: my-testbin",
        expected_usage_line: "",
    };

    static ref PROJECT_FLAGS: TestProject = TestProject {
        project: TestConfig {
            args: stringify!(
                #[derive(Args, Debug)]
                struct MyArgs {
                    /// Flag Documentation
                    #[arg(short = 'f', long = "flag")]
                    flag: bool,
                }
            ),
            main: stringify!(
                fn main(self) -> Result<(), CliError> {
                    println!("my-testbin:main:{:?}", self.args.flag);
                    Ok(())
                }
            ),
        }
        .project(),
        expected_usage_header: "Usage: my-testbin [OPTIONS]",
        expected_usage_line: "-f, --flag     Flag Documentation",
    };

    static ref PROJECT_MANDATORY_POSITIONAL: TestProject = TestProject {
        project: TestConfig {
            args: stringify!(
                #[derive(Args, Debug)]
                struct MyArgs {
                    mandatory_positional: String,
                }
            ),
            main: stringify!(
                fn main(self) -> Result<(), CliError> {
                    println!("my-testbin:main:{}", self.args.mandatory_positional);
                    Ok(())
                }
            ),
        }
        .project(),
        expected_usage_header: "Usage: my-testbin <MANDATORY_POSITIONAL>",
        expected_usage_line: "Arguments:\n  <MANDATORY_POSITIONAL>",
    };

    static ref PROJECT_OPTIONAL_POSITIONAL: TestProject = TestProject {
        project: TestConfig {
            args: stringify!(
                #[derive(Args, Debug)]
                struct MyArgs {
                    optional_positional: Option<String>,
                }
            ),
            main: stringify!(
                fn main(self) -> Result<(), CliError> {
                    println!("my-testbin:main:{:?}", self.args.optional_positional);
                    Ok(())
                }
            ),
        }
        .project(),
        expected_usage_header: "Usage: my-testbin [OPTIONAL_POSITIONAL]",
        expected_usage_line: "Arguments:\n  [OPTIONAL_POSITIONAL]",
    };

    static ref PROJECT_MANDATORY_ARGUMENT: TestProject = TestProject {
        project: TestConfig {
            args: stringify!(
                #[derive(Args, Debug)]
                struct MyArgs {
                    /// Mandatory Arg Documentation
                    #[arg(short='a', long)]
                    mandatory_argument: i32,
                }
            ),
            main: stringify!(
                fn main(self) -> Result<(), CliError> {
                    println!("my-testbin:main:{}", self.args.mandatory_argument);
                    Ok(())
                }
            ),
        }
        .project(),
        expected_usage_header: "Usage: my-testbin --mandatory-argument <MANDATORY_ARGUMENT>",
        expected_usage_line: "-a, --mandatory-argument <MANDATORY_ARGUMENT>  Mandatory Arg Documentation",
    };

    static ref PROJECT_OPTIONAL_ARGUMENT: TestProject = TestProject {
        project: TestConfig {
            args: stringify!(
                #[derive(Args, Debug)]
                struct MyArgs {
                    /// Optional Arg Documentation
                    #[arg(short='a', long)]
                    optional_argument: Option<i32>,
                }
            ),
            main: stringify!(
                fn main(self) -> Result<(), CliError> {
                    println!("my-testbin:main:{:?}", self.args.optional_argument);
                    Ok(())
                }
            ),
        }
        .project(),
        expected_usage_header: "Usage: my-testbin [OPTIONS]",
        expected_usage_line: "-a, --optional-argument <OPTIONAL_ARGUMENT>  Optional Arg Documentation",
    };
}

mod common {
    //! Tests common to most `PROJECT_XXX`

    use super::*;
    #[rstest]
    #[case(&PROJECT_NOARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_ARGUMENT)]
    #[test]
    fn no_args(#[case] test_project: &TestProject) {
        test_project
            .project
            .run_debug()
            .unwrap()
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(MAIN_MESSAGE));
    }

    #[rstest]
    #[case(&PROJECT_NOARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[case(&PROJECT_MANDATORY_ARGUMENT)]
    #[case(&PROJECT_OPTIONAL_ARGUMENT)]
    fn with_version_flag(
        #[case] test_project: &TestProject,
        #[values("--version", "-V")] version_flag: &str,
    ) {
        test_project
            .project
            .run_debug()
            .unwrap()
            .arg(version_flag)
            .assert()
            .success()
            // TODO For `--version`, the VERSION_MESSAGE should be on stdout
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(MAIN_MESSAGE).not());
    }

    #[rstest]
    #[case(&PROJECT_NOARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[case(&PROJECT_MANDATORY_ARGUMENT)]
    #[case(&PROJECT_OPTIONAL_ARGUMENT)]
    #[test]
    fn with_version_flag_bad(#[case] test_project: &TestProject) {
        test_project
            .project
            .run_debug()
            .unwrap()
            .arg("--version=bad")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: unexpected value 'bad' for '--version' found; no more were expected",
            ));
    }

    #[rstest]
    #[case(&PROJECT_NOARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[case(&PROJECT_MANDATORY_ARGUMENT)]
    #[case(&PROJECT_OPTIONAL_ARGUMENT)]
    #[test]
    fn with_help_flag(
        #[case] test_project: &TestProject,
        #[values("--help", "-h")] help_flag: &str,
    ) {
        let run = test_project
            .project
            .run_debug()
            .unwrap()
            .arg(help_flag)
            .assert()
            .success();
        test_project.expect_help_message(run);
    }

    #[rstest]
    #[case(&PROJECT_NOARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[case(&PROJECT_MANDATORY_ARGUMENT)]
    #[case(&PROJECT_OPTIONAL_ARGUMENT)]
    #[test]
    fn help_flag_bad(#[case] test_project: &TestProject) {
        test_project
            .project
            .run_debug()
            .unwrap()
            .arg("--help=bad")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: unexpected value 'bad' for '--help' found; no more were expected",
            ));
    }

    #[rstest]
    #[case(&PROJECT_NOARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[case(&PROJECT_MANDATORY_ARGUMENT)]
    #[case(&PROJECT_OPTIONAL_ARGUMENT)]
    #[test]
    fn help_and_version(
        #[case] test_project: &TestProject,
        #[values("--help", "-h")] help_flag: &str,
        #[values("--version", "-V")] version_flag: &str,
    ) {
        let run = test_project
            .project
            .run_debug()
            .unwrap()
            .arg(help_flag)
            .arg(version_flag)
            .assert()
            .success();
        test_project.expect_help_message(run);
    }

    #[rstest]
    #[case(&PROJECT_NOARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[case(&PROJECT_MANDATORY_ARGUMENT)]
    #[case(&PROJECT_OPTIONAL_ARGUMENT)]
    #[test]
    fn version_and_help(
        #[case] test_project: &TestProject,
        #[values("--help", "-h")] help_flag: &str,
        #[values("--version", "-V")] version_flag: &str,
    ) {
        let run = test_project
            .project
            .run_debug()
            .unwrap()
            .arg(version_flag)
            .arg(help_flag)
            .assert()
            .success();
        test_project.expect_help_message(run);
    }
}

mod flag {
    //! Tests specific to [PROJECT_FLAGS]

    use super::*;

    #[test]
    fn without_flag() {
        PROJECT_FLAGS
            .project
            .run_debug()
            .unwrap()
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(
                format!("{}:false", MAIN_MESSAGE,),
            ));
    }

    #[rstest]
    #[test]
    fn with_flag(#[values("-f", "--flag")] flag: &str) {
        PROJECT_FLAGS
            .project
            .run_debug()
            .unwrap()
            .arg(flag)
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!("{}:true", MAIN_MESSAGE,)));
    }

    #[test]
    fn with_flag_bad() {
        PROJECT_FLAGS
            .project
            .run_debug()
            .unwrap()
            .arg("--flag=bad")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: unexpected value 'bad' for '--flag' found; no more were expected",
            ));
    }

    #[rstest]
    #[test]
    fn with_flag_and_help_flag(
        #[values("-f", "--flag")] flag: &str,
        #[values("-h", "--help")] help_flag: &str,
    ) {
        let run = PROJECT_FLAGS
            .project
            .run_debug()
            .unwrap()
            .arg(flag)
            .arg(help_flag)
            .assert()
            .success();
        PROJECT_FLAGS.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_help_flag_and_flag(
        #[values("-f", "--flag")] flag: &str,
        #[values("-h", "--help")] help_flag: &str,
    ) {
        let run = PROJECT_FLAGS
            .project
            .run_debug()
            .unwrap()
            .arg(help_flag)
            .arg(flag)
            .assert()
            .success();
        PROJECT_FLAGS.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_flag_and_version_flag(
        #[values("-f", "--flag")] flag: &str,
        #[values("-V", "--version")] version_flag: &str,
    ) {
        PROJECT_FLAGS
            .project
            .run_debug()
            .unwrap()
            .arg(flag)
            .arg(version_flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }

    #[rstest]
    #[test]
    fn with_version_flag_and_flag(
        #[values("-f", "--flag")] flag: &str,
        #[values("-V", "--version")] version_flag: &str,
    ) {
        PROJECT_FLAGS
            .project
            .run_debug()
            .unwrap()
            .arg(version_flag)
            .arg(flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }
}

mod mandatory_positional {
    //! Tests specific to [PROJECT_MANDATORY_POSITIONAL]
    use super::*;

    #[test]
    fn with_positional_arg() {
        PROJECT_MANDATORY_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg("some_value")
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!(
                "{}:some_value",
                MAIN_MESSAGE,
            )));
    }

    #[test]
    fn missing_positional_arg() {
        PROJECT_MANDATORY_POSITIONAL
        .project
        .run_debug()
        .unwrap()
        .assert()
        .failure()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stderr(predicates::str::contains(
            "error: the following required arguments were not provided:\n  <MANDATORY_POSITIONAL>",
        ));
    }

    #[rstest]
    #[test]
    fn with_positional_and_help_flag(#[values("-h", "--help")] help_flag: &str) {
        let run = PROJECT_MANDATORY_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg("some_value")
            .arg(help_flag)
            .assert()
            .success();
        PROJECT_MANDATORY_POSITIONAL.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_help_flag_and_positional(#[values("-h", "--help")] help_flag: &str) {
        let run = PROJECT_MANDATORY_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg(help_flag)
            .arg("some_value")
            .assert()
            .success();
        PROJECT_MANDATORY_POSITIONAL.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_positional_and_version_flag(#[values("-V", "--version")] version_flag: &str) {
        PROJECT_MANDATORY_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg("positional")
            .arg(version_flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }

    #[rstest]
    #[test]
    fn with_version_flag_and_positional(#[values("-V", "--version")] version_flag: &str) {
        PROJECT_MANDATORY_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg(version_flag)
            .arg("positional")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }
}

mod optional_positional {
    //! Tests specific to [PROJECT_OPTIONAL_POSITIONAL]
    use super::*;

    #[test]
    fn with_positional_arg() {
        PROJECT_OPTIONAL_POSITIONAL
            .project
            .run_debug()
            .unwrap()
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
        PROJECT_OPTIONAL_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!("{}:None", MAIN_MESSAGE,)));
    }

    #[rstest]
    #[test]
    fn with_positional_and_help_flag(#[values("-h", "--help")] help_flag: &str) {
        let run = PROJECT_OPTIONAL_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg("some_value")
            .arg(help_flag)
            .assert()
            .success();
        PROJECT_OPTIONAL_POSITIONAL.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_help_flag_and_positional(#[values("-h", "--help")] help_flag: &str) {
        let run = PROJECT_OPTIONAL_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg(help_flag)
            .arg("some_value")
            .assert()
            .success();
        PROJECT_OPTIONAL_POSITIONAL.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_positional_and_version_flag(#[values("-V", "--version")] version_flag: &str) {
        PROJECT_OPTIONAL_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg("positional")
            .arg(version_flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }

    #[rstest]
    #[test]
    fn with_version_flag_and_positional(#[values("-V", "--version")] version_flag: &str) {
        PROJECT_OPTIONAL_POSITIONAL
            .project
            .run_debug()
            .unwrap()
            .arg(version_flag)
            .arg("positional")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }
}

mod mandatory_argument {
    //! Tests specific to [PROJECT_MANDATORY_ARGUMENT]

    use super::*;

    #[test]
    fn without_argument() {
        PROJECT_MANDATORY_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: the following required arguments were not provided:\n  --mandatory-argument <MANDATORY_ARGUMENT>",
            ));
    }

    #[rstest]
    #[test]
    fn with_argument(
        #[values(&["-a", "12345"], &["-a=12345"], &["--mandatory-argument", "12345"], &["--mandatory-argument=12345"])]
        argument: &[&str],
    ) {
        PROJECT_MANDATORY_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(
                format!("{}:12345", MAIN_MESSAGE,),
            ));
    }

    #[rstest]
    #[test]
    fn with_argument_bad(
        #[values(&["-a", "bad"], &["-a=bad"], &["--mandatory-argument", "bad"], &["--mandatory-argument=bad"])]
        argument: &[&str],
    ) {
        PROJECT_MANDATORY_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: invalid value 'bad' for '--mandatory-argument <MANDATORY_ARGUMENT>': invalid digit found in string",
            ));
    }

    #[rstest]
    #[test]
    fn with_argument_and_help_flag(
        #[values(&["-a", "12345"], &["-a=12345"], &["--mandatory-argument", "12345"], &["--mandatory-argument=12345"])]
        argument: &[&str],
        #[values("-h", "--help")] help_flag: &str,
    ) {
        let run = PROJECT_MANDATORY_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .arg(help_flag)
            .assert()
            .success();
        PROJECT_MANDATORY_ARGUMENT.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_help_flag_and_argument(
        #[values(&["-a", "12345"], &["-a=12345"], &["--mandatory-argument", "12345"], &["--mandatory-argument=12345"])]
        argument: &[&str],
        #[values("-h", "--help")] help_flag: &str,
    ) {
        let run = PROJECT_MANDATORY_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .arg(help_flag)
            .args(argument)
            .assert()
            .success();
        PROJECT_MANDATORY_ARGUMENT.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_argument_and_version_flag(
        #[values(&["-a", "12345"], &["-a=12345"], &["--mandatory-argument", "12345"], &["--mandatory-argument=12345"])]
        argument: &[&str],
        #[values("-V", "--version")] version_flag: &str,
    ) {
        PROJECT_MANDATORY_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .arg(version_flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }

    #[rstest]
    #[test]
    fn with_version_flag_and_argument(
        #[values(&["-a", "12345"], &["-a=12345"], &["--mandatory-argument", "12345"], &["--mandatory-argument=12345"])]
        argument: &[&str],
        #[values("-V", "--version")] version_flag: &str,
    ) {
        PROJECT_MANDATORY_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .arg(version_flag)
            .args(argument)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }
}

mod optional_argument {
    //! Tests specific to [PROJECT_OPTIONAL_ARGUMENT]

    use super::*;

    #[test]
    fn without_argument() {
        PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!("{}:None", MAIN_MESSAGE,)));
    }

    #[rstest]
    #[test]
    fn with_argument(
        #[values(&["-a", "12345"], &["-a=12345"], &["--optional-argument", "12345"], &["--optional-argument=12345"])]
        argument: &[&str],
    ) {
        PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!(
                "{}:Some(12345)",
                MAIN_MESSAGE,
            )));
    }

    #[rstest]
    #[test]
    fn with_argument_bad(
        #[values(&["-a", "bad"], &["-a=bad"], &["--optional-argument", "bad"], &["--optional-argument=bad"])]
        argument: &[&str],
    ) {
        PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: invalid value 'bad' for '--optional-argument <OPTIONAL_ARGUMENT>': invalid digit found in string",
            ));
    }

    #[rstest]
    #[test]
    fn with_argument_and_help_flag(
        #[values(&["-a", "12345"], &["-a=12345"], &["--optional-argument", "12345"], &["--optional-argument=12345"])]
        argument: &[&str],
        #[values("-h", "--help")] help_flag: &str,
    ) {
        let run = PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .arg(help_flag)
            .assert()
            .success();
        PROJECT_OPTIONAL_ARGUMENT.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_help_flag_and_argument(
        #[values(&["-a", "12345"], &["-a=12345"], &["--optional-argument", "12345"], &["--optional-argument=12345"])]
        argument: &[&str],
        #[values("-h", "--help")] help_flag: &str,
    ) {
        let run = PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .arg(help_flag)
            .args(argument)
            .assert()
            .success();
        PROJECT_OPTIONAL_ARGUMENT.expect_help_message(run);
    }

    #[rstest]
    #[test]
    fn with_argument_and_version_flag(
        #[values(&["-a", "12345"], &["-a=12345"], &["--optional-argument", "12345"], &["--optional-argument=12345"])]
        argument: &[&str],
        #[values("-V", "--version")] version_flag: &str,
    ) {
        PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .args(argument)
            .arg(version_flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }

    #[rstest]
    #[test]
    fn with_version_flag_and_argument(
        #[values(&["-a", "12345"], &["-a=12345"], &["--optional-argument", "12345"], &["--optional-argument=12345"])]
        argument: &[&str],
        #[values("-V", "--version")] version_flag: &str,
    ) {
        PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .arg(version_flag)
            .args(argument)
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "Error: the argument '--version' cannot be used with other arguments",
            ));
    }
}

mod debug_build_warning {
    use predicates::str::ContainsPredicate;

    use super::*;

    fn debug_build_warning() -> ContainsPredicate {
        predicates::str::contains("WARNING! This is a debug build.")
    }

    #[test]
    fn debug_build() {
        PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_debug()
            .unwrap()
            .assert()
            .success()
            .stderr(debug_build_warning());
    }

    #[test]
    fn release_build() {
        PROJECT_OPTIONAL_ARGUMENT
            .project
            .run_release()
            .unwrap()
            .assert()
            .success()
            .stderr(debug_build_warning().not());
    }
}

// TODO Add integration tests for:
//  - subcommand
