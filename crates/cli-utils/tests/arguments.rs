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
            edition = "2021"
            
            [dependencies]
            cryfs-cli-utils = {path = ""#
    ),
    env!("CARGO_MANIFEST_DIR"),
    r#""}"#,
);

// TODO Move the `EXPECTED_USAGE_XXX` consts into [TestConfig] and then parameterize the
// test cases based on `TestConfig`, not on `TempProject`. But that requires `TestConfig`
// to store the `TempProject` so we don't re-create it for every test case.
const EXPECTED_USAGE_NOARGS_MAIN: &str = "Usage: my-testbin";
const EXPECTED_USAGE_FLAG_MAIN: &str = "Usage: my-testbin [OPTIONS]";
const EXPECTED_USAGE_FLAG: &str = "-f, --flag     Flag Documentation";
const EXPECTED_USAGE_MANDATORY_POSITIONAL_MAIN: &str = "Usage: my-testbin <MANDATORY_POSITIONAL>";
const EXPECTED_USAGE_MANDATORY_POSITIONAL: &str = "Arguments:\n  <MANDATORY_POSITIONAL>";
const EXPECTED_USAGE_OPTIONAL_POSITIONAL_MAIN: &str = "Usage: my-testbin [OPTIONAL_POSITIONAL]";
const EXPECTED_USAGE_OPTIONAL_POSITIONAL: &str = "Arguments:\n  [OPTIONAL_POSITIONAL]";

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
                    async_trait::async_trait,
                    clap::{self, Args},
                    cryfs_version::{Version, VersionInfo},
                },
                run, Application, Environment,
            };
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

            fn new(args: MyArgs, env: Environment) -> Result<Self> {
                Ok(Self { args })
            }
        );
        let main_main = stringify!(
            fn main() -> Result<()> {
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

                #[async_trait]
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

const TESTCONFIG_NOARGS: TestConfig = TestConfig {
    args: stringify!(
        #[derive(Args, Debug)]
        struct MyArgs {}
    ),
    main: stringify!(
        async fn main(&self) -> Result<()> {
            println!("my-testbin:main");
            Ok(())
        }
    ),
};

const TESTCONFIG_FLAGS: TestConfig = TestConfig {
    args: stringify!(
        #[derive(Args, Debug)]
        struct MyArgs {
            /// Flag Documentation
            #[arg(short = 'f', long = "flag")]
            flag: bool,
        }
    ),
    main: stringify!(
        async fn main(&self) -> Result<()> {
            println!("my-testbin:main:{:?}", self.args.flag);
            Ok(())
        }
    ),
};

const TESTCONFIG_MANDATORY_POSITIONAL: TestConfig = TestConfig {
    args: stringify!(
        #[derive(Args, Debug)]
        struct MyArgs {
            mandatory_positional: String,
        }
    ),
    main: stringify!(
        async fn main(&self) -> Result<()> {
            println!("my-testbin:main:{}", self.args.mandatory_positional);
            Ok(())
        }
    ),
};

const TESTCONFIG_OPTIONAL_POSITIONAL: TestConfig = TestConfig {
    args: stringify!(
        #[derive(Args, Debug)]
        struct MyArgs {
            optional_positional: Option<String>,
        }
    ),
    main: stringify!(
        async fn main(&self) -> Result<()> {
            println!("my-testbin:main:{:?}", self.args.optional_positional);
            Ok(())
        }
    ),
};

lazy_static! {
    static ref PROJECT_NO_ARGS: TempProject = TESTCONFIG_NOARGS.project();
    static ref PROJECT_FLAGS: TempProject = TESTCONFIG_FLAGS.project();
    static ref PROJECT_MANDATORY_POSITIONAL: TempProject =
        TESTCONFIG_MANDATORY_POSITIONAL.project();
    static ref PROJECT_OPTIONAL_POSITIONAL: TempProject = TESTCONFIG_OPTIONAL_POSITIONAL.project();
}

mod common {
    //! Tests common to all `PROJECT_XXX`

    use super::*;
    #[rstest]
    #[case(&PROJECT_NO_ARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[test]
    fn no_args(#[case] project: &TempProject) {
        project
            .run()
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(MAIN_MESSAGE));
    }

    #[rstest]
    #[case(&PROJECT_NO_ARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[test]
    fn version_flag_long(#[case] project: &TempProject) {
        project
            .run()
            .arg("--version")
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(MAIN_MESSAGE).not());
    }

    #[rstest]
    #[case(&PROJECT_NO_ARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[test]
    fn version_flag_short(#[case] project: &TempProject) {
        project
            .run()
            .arg("-V")
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(MAIN_MESSAGE).not());
    }

    #[rstest]
    #[case(&PROJECT_NO_ARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[test]
    fn version_flag_bad(#[case] project: &TempProject) {
        project
            .run()
            .arg("--version=bad")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: unexpected value 'bad' for '--version' found; no more were expected",
            ));
    }

    #[rstest]
    #[case(&PROJECT_NO_ARGS, predicates::str::contains(EXPECTED_USAGE_NOARGS_MAIN), predicates::constant::always())]
    #[case(&PROJECT_FLAGS, predicates::str::contains(EXPECTED_USAGE_FLAG_MAIN), predicates::str::contains(EXPECTED_USAGE_FLAG))]
    #[case(&PROJECT_MANDATORY_POSITIONAL, predicates::str::contains(EXPECTED_USAGE_MANDATORY_POSITIONAL_MAIN), predicates::str::contains(EXPECTED_USAGE_MANDATORY_POSITIONAL))]
    #[case(&PROJECT_OPTIONAL_POSITIONAL, predicates::str::contains(EXPECTED_USAGE_OPTIONAL_POSITIONAL_MAIN), predicates::str::contains(EXPECTED_USAGE_OPTIONAL_POSITIONAL))]
    #[test]
    fn help_flag_long(
        #[case] project: &TempProject,
        #[case] usage_stdout_predicate: impl predicates::Predicate<str>,
        #[case] extra_stdout_predicate: impl predicates::Predicate<str>,
    ) {
        project
            .run()
            .arg("--help")
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(usage_stdout_predicate)
            .stdout(predicates::str::contains("-V, --version"))
            .stdout(predicates::str::contains("-h, --help"))
            .stdout(extra_stdout_predicate)
            .stdout(predicates::str::contains(MAIN_MESSAGE).not());
    }

    #[rstest]
    #[case(&PROJECT_NO_ARGS, predicates::str::contains(EXPECTED_USAGE_NOARGS_MAIN), predicates::constant::always())]
    #[case(&PROJECT_FLAGS, predicates::str::contains(EXPECTED_USAGE_FLAG_MAIN), predicates::str::contains(EXPECTED_USAGE_FLAG))]
    #[case(&PROJECT_MANDATORY_POSITIONAL, predicates::str::contains(EXPECTED_USAGE_MANDATORY_POSITIONAL_MAIN), predicates::str::contains(EXPECTED_USAGE_MANDATORY_POSITIONAL))]
    #[case(&PROJECT_OPTIONAL_POSITIONAL, predicates::str::contains(EXPECTED_USAGE_OPTIONAL_POSITIONAL_MAIN), predicates::str::contains(EXPECTED_USAGE_OPTIONAL_POSITIONAL))]
    #[test]
    fn help_flag_short(
        #[case] project: &TempProject,
        #[case] usage_stdout_predicate: impl predicates::Predicate<str>,
        #[case] extra_stdout_predicate: impl predicates::Predicate<str>,
    ) {
        project
            .run()
            .arg("-h")
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(usage_stdout_predicate)
            .stdout(predicates::str::contains("-V, --version"))
            .stdout(predicates::str::contains("-h, --help"))
            .stdout(extra_stdout_predicate)
            .stdout(predicates::str::contains(MAIN_MESSAGE).not());
    }

    #[rstest]
    #[case(&PROJECT_NO_ARGS)]
    #[case(&PROJECT_FLAGS)]
    #[case(&PROJECT_MANDATORY_POSITIONAL)]
    #[case(&PROJECT_OPTIONAL_POSITIONAL)]
    #[test]
    fn help_flag_bad(#[case] project: &TempProject) {
        project
            .run()
            .arg("--help=bad")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: unexpected value 'bad' for '--help' found; no more were expected",
            ));
    }
}

mod flag {
    //! Tests specific to [PROJECT_FLAGS]

    use super::*;

    #[test]
    fn without_flag() {
        PROJECT_FLAGS
            .run()
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(
                format!("{}:false", MAIN_MESSAGE,),
            ));
    }

    #[test]
    fn with_flag_short() {
        PROJECT_FLAGS
            .run()
            .arg("-f")
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!("{}:true", MAIN_MESSAGE,)));
    }

    #[test]
    fn with_flag_long() {
        PROJECT_FLAGS
            .run()
            .arg("--flag")
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!("{}:true", MAIN_MESSAGE,)));
    }

    #[test]
    fn with_flag_bad() {
        PROJECT_FLAGS
            .run()
            .arg("--flag=bad")
            .assert()
            .failure()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stderr(predicates::str::contains(
                "error: unexpected value 'bad' for '--flag' found; no more were expected",
            ));
    }
}

mod mandatory_positional {
    //! Tests specific to [PROJECT_MANDATORY_POSITIONAL]
    use super::*;

    #[test]
    fn with_positional_arg() {
        PROJECT_MANDATORY_POSITIONAL
            .run()
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
        PROJECT_MANDATORY_POSITIONAL.run()
        .assert()
        .failure()
        .stderr(predicates::str::contains(VERSION_MESSAGE))
        .stderr(predicates::str::contains(
            "error: the following required arguments were not provided:\n  <MANDATORY_POSITIONAL>",
        ));
    }
}

mod optional_positional {
    //! Tests specific to [PROJECT_OPTIONAL_POSITIONAL]
    use super::*;

    #[test]
    fn with_positional_arg() {
        PROJECT_OPTIONAL_POSITIONAL
            .run()
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
            .run()
            .assert()
            .success()
            .stderr(predicates::str::contains(VERSION_MESSAGE))
            .stdout(predicates::str::contains(format!("{}:None", MAIN_MESSAGE,)));
    }
}

// TODO Test `--version` and `--help` combination
// TODO Test combination of the flag/positional argument and `--version` or `--help
// TODO Add integration tests for:
//  - optional argument
//  - mandatory argument
//  - subcommand
