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

const EXPECTED_USAGE_NOARGS_MAIN: &str = "Usage: my-testbin";
const EXPECTED_USAGE_FLAG_MAIN: &str = "Usage: my-testbin [OPTIONS]";
const EXPECTED_USAGE_FLAG: &str = "-f, --flag     Flag Documentation";
const EXPECTED_USAGE_MANDATORY_POSITIONAL_MAIN: &str = "Usage: my-testbin <MANDATORY_POSITIONAL>";
const EXPECTED_USAGE_MANDATORY_POSITIONAL: &str = "Arguments:\n  <MANDATORY_POSITIONAL>";
const EXPECTED_USAGE_OPTIONAL_POSITIONAL_MAIN: &str = "Usage: my-testbin [OPTIONAL_POSITIONAL]";
const EXPECTED_USAGE_OPTIONAL_POSITIONAL: &str = "Arguments:\n  [OPTIONAL_POSITIONAL]";

lazy_static! {
    // TODO Deduplicate code between different PROJECT_XXX.
    //      Probably by introducing a TestConfig struct that holds fields like the code to set up args, code from main, expected usage lines, etc.

    static ref PROJECT_NO_ARGS: TempProject = TempProjectBuilder::new()
        .unwrap()
        .cargo(CARGO_TOML)
        .main(stringify!(
            use cryfs_cli_utils::{
                reexports_for_tests::{
                    anyhow::Result,
                    async_trait::async_trait,
                    clap::{self, Args},
                    cryfs_version::{Version, VersionInfo},
                },
                run, Application, Environment,
            };

            #[derive(Args, Debug)]
            struct MyArgs {}

            struct Cli {}

            #[async_trait]
            impl Application for Cli {
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
                    Ok(Self {})
                }

                async fn main(&self) -> Result<()> {
                    println!("my-testbin:main");
                    Ok(())
                }
            }

            fn main() -> Result<()> {
                run::<Cli>()
            }
        ))
        .build()
        .unwrap();

    static ref PROJECT_FLAGS: TempProject = TempProjectBuilder::new().unwrap()
        .cargo(CARGO_TOML)
        .main(stringify!(
            use cryfs_cli_utils::{
                reexports_for_tests::{
                    anyhow::Result,
                    async_trait::async_trait,
                    clap::{self, Args},
                    cryfs_version::{Version, VersionInfo},
                },
                run, Application, Environment,
            };

            #[derive(Args, Debug)]
            struct MyArgs {
                /// Flag Documentation
                #[arg(short = 'f', long = "flag")]
                flag: bool,
            }

            struct Cli {
                args: MyArgs,
            }

            #[async_trait]
            impl Application for Cli {
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

                async fn main(&self) -> Result<()> {
                    println!("my-testbin:main:{:?}", self.args.flag,);
                    Ok(())
                }
            }

            fn main() -> Result<()> {
                run::<Cli>()
            }
        )).build().unwrap();

    static ref PROJECT_MANDATORY_POSITIONAL: TempProject = TempProjectBuilder::new().unwrap()
        .cargo(CARGO_TOML)
        .main(stringify!(
            use cryfs_cli_utils::{
                reexports_for_tests::{
                    anyhow::Result,
                    async_trait::async_trait,
                    clap::{self, Args},
                    cryfs_version::{Version, VersionInfo},
                },
                run, Application, Environment,
            };

            #[derive(Args, Debug)]
            struct MyArgs {
                mandatory_positional: String,
            }

            struct Cli {
                args: MyArgs,
            }

            #[async_trait]
            impl Application for Cli {
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

                async fn main(&self) -> Result<()> {
                    println!(
                        "my-testbin:main:{}",
                        self.args.mandatory_positional,
                    );
                    Ok(())
                }
            }

            fn main() -> Result<()> {
                run::<Cli>()
            }
        )).build().unwrap();

        static ref PROJECT_OPTIONAL_POSITIONAL: TempProject = TempProjectBuilder::new().unwrap()
        .cargo(CARGO_TOML)
        .main(stringify!(
            use cryfs_cli_utils::{
                reexports_for_tests::{
                    anyhow::Result,
                    async_trait::async_trait,
                    clap::{self, Args},
                    cryfs_version::{Version, VersionInfo},
                },
                run, Application, Environment,
            };

            #[derive(Args, Debug)]
            struct MyArgs {
                optional_positional: Option<String>,
            }

            struct Cli {
                args: MyArgs,
            }

            #[async_trait]
            impl Application for Cli {
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

                async fn main(&self) -> Result<()> {
                    println!(
                        "my-testbin:main:{:?}",
                        self.args.optional_positional,
                    );
                    Ok(())
                }
            }

            fn main() -> Result<()> {
                run::<Cli>()
            }
        )).build().unwrap();
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
