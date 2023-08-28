use anyhow::{anyhow, bail, Context, Result};
use std::env::VarError;
use std::path::PathBuf;

const FRONTEND_KEY: &str = "CRYFS_FRONTEND";
const FRONTEND_NONINTERACTIVE: &str = "noninteractive";
const NOUPDATECHECK_KEY: &str = "CRYFS_NO_UPDATE_CHECK";
const LOCALSTATEDIR_KEY: &str = "CRYFS_LOCAL_STATE_DIR";

pub struct Environment {}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub fn is_noninteractive(&self) -> bool {
        match std::env::var(FRONTEND_KEY) {
            Ok(frontend) => frontend == FRONTEND_NONINTERACTIVE,
            Err(VarError::NotPresent) | Err(VarError::NotUnicode(..)) => false,
        }
    }

    #[cfg(feature = "check_for_updates")]
    pub fn no_update_check(&self) -> bool {
        match std::env::var(NOUPDATECHECK_KEY) {
            Ok(val) => val == "true",
            Err(VarError::NotPresent) | Err(VarError::NotUnicode(..)) => false,
        }
    }

    pub fn local_state_dir(&self) -> Result<PathBuf> {
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
            assert_eq!(false, Environment::new().is_noninteractive());
        }

        #[test]
        fn when_set_to_noninteractive_then_returns_true() {
            let _lock = lock_test();
            let _var = set_env(FRONTEND_KEY.into(), FRONTEND_NONINTERACTIVE);
            assert_eq!(true, Environment::new().is_noninteractive());
        }

        #[test]
        fn when_set_to_empty_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(FRONTEND_KEY.into(), "");
            assert_eq!(false, Environment::new().is_noninteractive());
        }

        #[test]
        fn when_set_to_something_else_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(FRONTEND_KEY.into(), "something");
            assert_eq!(false, Environment::new().is_noninteractive());
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
            assert_eq!(false, Environment::new().no_update_check());
        }

        #[test]
        fn when_set_to_true_then_returns_true() {
            let _lock = lock_test();
            let _var = set_env(NOUPDATECHECK_KEY.into(), "true");
            assert_eq!(true, Environment::new().no_update_check());
        }

        #[test]
        fn when_set_to_empty_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(NOUPDATECHECK_KEY.into(), "");
            assert_eq!(false, Environment::new().no_update_check());
        }

        #[test]
        fn when_set_to_something_else_then_returns_false() {
            let _lock = lock_test();
            let _var = set_env(NOUPDATECHECK_KEY.into(), "something");
            assert_eq!(false, Environment::new().no_update_check());
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
            let local_state_dir = Environment::new().local_state_dir().unwrap();
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
            let local_state_dir = Environment::new().local_state_dir();
            assert!(local_state_dir.is_err());
            assert_eq!(
                format!(
                    "Failed to access specified local state directory at {}",
                    nonexisting_path.to_str().unwrap(),
                ),
                local_state_dir.unwrap_err().to_string()
            );
        }

        #[test]
        fn when_set_to_existing_absolute_dir_then_returns_dir() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let _var = set_env(LOCALSTATEDIR_KEY.into(), tmpdir.path());
            let local_state_dir = Environment::new().local_state_dir().unwrap();
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
            let local_state_dir = Environment::new().local_state_dir();
            assert!(local_state_dir.is_err());
            assert_eq!(
                format!(
                    "Failed to access specified local state directory at {}",
                    nonexisting_path.to_str().unwrap(),
                ),
                local_state_dir.unwrap_err().to_string(),
            );
        }

        #[test]
        fn when_set_to_existing_relative_dir_without_dot_then_returns_dir() {
            let _lock = lock_test();
            let tmpdir = TempDir::new("some_path").unwrap();
            let relative_path =
                pathdiff::diff_paths(tmpdir.path(), std::env::current_dir().unwrap()).unwrap();
            let _var = set_env(LOCALSTATEDIR_KEY.into(), relative_path);
            let local_state_dir = Environment::new().local_state_dir().unwrap();
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
            let local_state_dir = Environment::new().local_state_dir();
            assert!(local_state_dir.is_err());
            assert_eq!(
                format!(
                    "Failed to access specified local state directory at {}",
                    &nonexisting_path,
                ),
                local_state_dir.unwrap_err().to_string(),
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
            let local_state_dir = Environment::new().local_state_dir().unwrap();
            assert_eq!(
                canonicalize(tmpdir.path()).unwrap(),
                canonicalize(local_state_dir).unwrap(),
            );
        }
    }
}
