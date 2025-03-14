use anyhow::{Context, Result, anyhow, bail};
use std::env::VarError;
use std::path::PathBuf;

use crate::error::{CliError, CliErrorKind, CliResultExt};

const FRONTEND_KEY: &str = "CRYFS_FRONTEND";
const FRONTEND_NONINTERACTIVE: &str = "noninteractive";
#[cfg(feature = "check_for_updates")]
const NOUPDATECHECK_KEY: &str = "CRYFS_NO_UPDATE_CHECK";
const LOCALSTATEDIR_KEY: &str = "CRYFS_LOCAL_STATE_DIR";

pub struct EnvVarDoc {
    pub key: &'static str,
    pub value: &'static str,
    pub description: &'static str,
}

pub const ENV_VARS_DOCUMENTATION: &[EnvVarDoc] = &[
    EnvVarDoc {
        key: FRONTEND_KEY,
        value: FRONTEND_NONINTERACTIVE,
        description: "Work better together with tools. With this option set, CryFS won't ask anything, but use default values for options you didn't specify on command line. Furthermore, it won't ask you to enter a new password a second time (password confirmation).",
    },
    #[cfg(feature = "check_for_updates")]
    EnvVarDoc {
        key: NOUPDATECHECK_KEY,
        value: "true",
        description: "By default, CryFS connects to the internet to check for known security vulnerabilities and new versions. This option disables this.",
    },
    EnvVarDoc {
        key: LOCALSTATEDIR_KEY,
        value: "[path]",
        description: "Sets the directory cryfs uses to store local state. This local state is used to recognize known file systems and run integrity checks, i.e. check that they haven't been modified by an attacker.\nDefault value: /home/heinzi/.local/share/cryfs",
    },
];

#[derive(Debug, Clone)]
pub struct Environment {
    pub is_noninteractive: bool,
    #[cfg(feature = "check_for_updates")]
    pub no_update_check: bool,
    pub local_state_dir: PathBuf,
}

impl Environment {
    pub(crate) fn read_env() -> Result<Self, CliError> {
        Ok(Self {
            is_noninteractive: Self::is_noninteractive(),
            #[cfg(feature = "check_for_updates")]
            no_update_check: Self::no_update_check(),
            local_state_dir: Self::local_state_dir()
                .map_cli_error(CliErrorKind::InaccessibleLocalStateDir)?,
        })
    }

    fn is_noninteractive() -> bool {
        match std::env::var(FRONTEND_KEY) {
            Ok(frontend) => frontend == FRONTEND_NONINTERACTIVE,
            Err(VarError::NotPresent) | Err(VarError::NotUnicode(..)) => false,
        }
    }

    #[cfg(feature = "check_for_updates")]
    fn no_update_check() -> bool {
        match std::env::var(NOUPDATECHECK_KEY) {
            Ok(val) => val == "true",
            Err(VarError::NotPresent) | Err(VarError::NotUnicode(..)) => false,
        }
    }

    fn local_state_dir() -> Result<PathBuf> {
        match std::env::var(LOCALSTATEDIR_KEY) {
            Ok(local_state_dir) => std::fs::canonicalize(&local_state_dir).with_context(|| {
                anyhow!("Failed to access specified local state directory at {local_state_dir}")
            }),
            Err(VarError::NotUnicode(local_state_dir)) => {
                bail!("Failed to access specified local state directory at {local_state_dir:?}")
            }
            Err(VarError::NotPresent) => {
                let mut local_state_dir =
                    dirs::data_local_dir().context("Tried to query location of local data dir")?;
                local_state_dir.push("cryfs");
                Ok(local_state_dir)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use envtestkit::{lock::lock_test, set_env};

    mod is_noninteractive {
        use super::*;

        #[test]
        fn when_not_set_then_returns_false() {
            let _lock = lock_test();
            assert!(
                std::env::var(FRONTEND_KEY).is_err(),
                "This test assumes that the env var isn't set but it seems set?"
            );
            assert_eq!(false, Environment::read_env().unwrap().is_noninteractive);
        }

        #[test]
        fn when_set_to_noninteractive_then_returns_true() {
            let _lock = lock_test();
            let _var = set_env(FRONTEND_KEY.into(), FRONTEND_NONINTERACTIVE);
            assert_eq!(true, Environment::read_env().unwrap().is_noninteractive);
        }

        #[test]
        fn when_set_to_empty_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(FRONTEND_KEY.into(), "");
            assert_eq!(false, Environment::read_env().unwrap().is_noninteractive);
        }

        #[test]
        fn when_set_to_something_else_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(FRONTEND_KEY.into(), "something");
            assert_eq!(false, Environment::read_env().unwrap().is_noninteractive);
        }
    }

    #[cfg(feature = "check_for_updates")]
    mod no_update_check {
        use super::*;

        #[test]
        fn when_not_set_then_returns_false() {
            let _lock = lock_test();
            assert!(
                std::env::var(NOUPDATECHECK_KEY).is_err(),
                "This test assumes that the env var isn't set but it seems set?"
            );
            assert_eq!(false, Environment::read_env().unwrap().no_update_check);
        }

        #[test]
        fn when_set_to_true_then_returns_true() {
            let _lock = lock_test();
            let _var = set_env(NOUPDATECHECK_KEY.into(), "true");
            assert_eq!(true, Environment::read_env().unwrap().no_update_check);
        }

        #[test]
        fn when_set_to_empty_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(NOUPDATECHECK_KEY.into(), "");
            assert_eq!(false, Environment::read_env().unwrap().no_update_check);
        }

        #[test]
        fn when_set_to_something_else_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(NOUPDATECHECK_KEY.into(), "something");
            assert_eq!(false, Environment::read_env().unwrap().no_update_check);
        }
    }

    mod local_state_dir {
        use super::*;
        use std::fs::canonicalize;
        use tempdir::TempDir;

        #[test]
        fn when_not_set_then_returns_default() {
            let _lock = lock_test();
            assert!(
                std::env::var(LOCALSTATEDIR_KEY).is_err(),
                "This test assumes that the env var isn't set but it seems set?"
            );
            let local_state_dir = Environment::read_env().unwrap().local_state_dir;
            assert_eq!(
                dirs::data_local_dir().unwrap().join("cryfs"),
                local_state_dir,
            );
        }

        #[test]
        fn when_set_to_nonexisting_absolute_dir_then_returns_error() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let nonexisting_path = tmpdir.path().join("nonexisting");
            let _var = set_env(LOCALSTATEDIR_KEY.into(), &nonexisting_path);
            let env = Environment::read_env();
            assert!(env.is_err());
            assert_eq!(
                format!(
                    "Failed to access specified local state directory at {}",
                    nonexisting_path.to_str().unwrap(),
                ),
                env.unwrap_err().to_string()
            );
        }

        #[test]
        fn when_set_to_existing_absolute_dir_then_returns_dir() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let _var = set_env(LOCALSTATEDIR_KEY.into(), tmpdir.path());
            let local_state_dir = Environment::read_env().unwrap().local_state_dir;
            assert_eq!(
                canonicalize(tmpdir.path()).unwrap(),
                canonicalize(local_state_dir).unwrap(),
            );
        }

        #[test]
        fn when_set_to_nonexisting_relative_dir_without_dot_then_returns_dir() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let nonexisting_path = tmpdir.path().join("nonexisting");
            let _var = set_env(LOCALSTATEDIR_KEY.into(), &nonexisting_path);
            let env = Environment::read_env();
            assert!(env.is_err());
            assert_eq!(
                format!(
                    "Failed to access specified local state directory at {}",
                    nonexisting_path.to_str().unwrap(),
                ),
                env.unwrap_err().to_string(),
            );
        }

        #[test]
        fn when_set_to_existing_relative_dir_without_dot_then_returns_dir() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let relative_path =
                pathdiff::diff_paths(tmpdir.path(), std::env::current_dir().unwrap()).unwrap();
            let _var = set_env(LOCALSTATEDIR_KEY.into(), relative_path);
            let local_state_dir = Environment::read_env().unwrap().local_state_dir;
            assert_eq!(
                canonicalize(tmpdir.path()).unwrap(),
                canonicalize(local_state_dir).unwrap(),
            );
        }

        #[test]
        fn when_set_to_nonexisting_relative_dir_with_dot_then_returns_dir() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let nonexisting_path =
                format!("./{}", tmpdir.path().join("nonexisting").to_str().unwrap());
            let _var = set_env(LOCALSTATEDIR_KEY.into(), &nonexisting_path);
            let env = Environment::read_env();
            assert!(env.is_err());
            assert_eq!(
                format!(
                    "Failed to access specified local state directory at {}",
                    &nonexisting_path,
                ),
                env.unwrap_err().to_string(),
            );
        }

        #[test]
        fn when_set_to_existing_relative_dir_with_dot_then_returns_dir() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let relative_path = format!(
                "./{}",
                pathdiff::diff_paths(tmpdir.path(), std::env::current_dir().unwrap())
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
            let _var = set_env(LOCALSTATEDIR_KEY.into(), relative_path);
            let local_state_dir = Environment::read_env().unwrap().local_state_dir;
            assert_eq!(
                canonicalize(tmpdir.path()).unwrap(),
                canonicalize(local_state_dir).unwrap(),
            );
        }
    }
}
