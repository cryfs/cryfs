use anyhow::{Result, bail};
use clap::ValueEnum;

use cryfs_rustfs::AtimeUpdateBehavior;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AtimeOption {
    /// Use `relatime`. See below.
    Atime,

    /// Update the file atime with every access
    Strictatime,

    /// Don't update the file atime at all. This saves a lot of writes, but some applications may not work correctly. This is the default.
    Noatime,

    /// Update the file atime only if the previous atime is older than mtime or ctime, or the previous atime is over 24 hours old
    Relatime,

    /// Update the directory atime only during write access. This can be combined with `relatime` or `strictatime`.
    Nodiratime,
}

struct Flags {
    atime: bool,
    strictatime: bool,
    noatime: bool,
    relatime: bool,
    nodiratime: bool,
}

impl AtimeOption {
    pub fn to_atime_behavior(options: &[AtimeOption]) -> Result<AtimeUpdateBehavior> {
        let mut flags = Flags {
            atime: false,
            strictatime: false,
            noatime: false,
            relatime: false,
            nodiratime: false,
        };
        for option in options {
            match option {
                AtimeOption::Atime => flags.atime = true,
                AtimeOption::Strictatime => flags.strictatime = true,
                AtimeOption::Noatime => flags.noatime = true,
                AtimeOption::Relatime => flags.relatime = true,
                AtimeOption::Nodiratime => flags.nodiratime = true,
            }
        }

        match flags {
            Flags {noatime: true, atime: false, relatime: false, strictatime: false, nodiratime: _} => {
                // note: can have nodiratime flag set but that is ignored because it is already included in the noatime policy.
                Ok(AtimeUpdateBehavior::Noatime)
            }
            Flags {noatime: true, atime: true, ..} => {
                bail!("Cannot use both noatime and atime");
            }
            Flags {noatime: true, relatime: true, ..} => {
                bail!("Cannot use both noatime and relatime");
            }
            Flags {noatime: true, strictatime: true, ..} => {
                bail!("Cannot use both noatime and strictatime");
            }
            Flags {atime: true, strictatime: true, ..} => {
                bail!("Cannot use both atime and strictatime");
            }
            Flags {relatime: true, strictatime: true, ..} => {
                bail!("Cannot use both relatime and strictatime");
            }
            Flags {atime: true, noatime: false, strictatime: false, nodiratime: true, relatime: _} | Flags {relatime: true, noatime: false, strictatime: false, nodiratime: true, atime: _} => {
                // note: atime and relatime can be combined because they're identical
                Ok(AtimeUpdateBehavior::NodiratimeRelatime)
            }
            Flags {atime: true, noatime: false, strictatime: false, nodiratime: false, relatime: _} | Flags {relatime: true, noatime: false, strictatime: false, nodiratime: false, atime: _}=> {
                // note: atime and relatime can be combined because they're identical
                Ok(AtimeUpdateBehavior::Relatime)
            }
            Flags {strictatime: true, noatime: false, atime: false, relatime: false, nodiratime: true} => {
                Ok(AtimeUpdateBehavior::NodiratimeStrictatime)
            }
            Flags {strictatime: true, noatime: false, atime: false, relatime: false, nodiratime: false} => {
                Ok(AtimeUpdateBehavior::Strictatime)
            }
            Flags {nodiratime: true, noatime: false, atime: false, relatime: false, strictatime: false} => {
                // note: nodiratime is ignored if noatime is not set
                Ok(AtimeUpdateBehavior::Noatime)
            }
            Flags {noatime: false, atime: false, relatime: false, strictatime: false, nodiratime: false} => {
                // Default is NOATIME, this reduces the probability for synchronization conflicts
                Ok(AtimeUpdateBehavior::Noatime)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_one_flag(
        #[values(
            (AtimeOption::Atime, AtimeUpdateBehavior::Relatime), 
            (AtimeOption::Strictatime, AtimeUpdateBehavior::Strictatime),
            (AtimeOption::Noatime, AtimeUpdateBehavior::Noatime),
            (AtimeOption::Relatime, AtimeUpdateBehavior::Relatime),
            (AtimeOption::Nodiratime, AtimeUpdateBehavior::Noatime)
        )] option: (AtimeOption, AtimeUpdateBehavior),
    ) {
        assert_eq!(
            AtimeOption::to_atime_behavior(&[option.0]).unwrap(),
            option.1,
        );
    }

    #[rstest]
    fn test_double_flag(
        #[values(
            (AtimeOption::Atime, AtimeUpdateBehavior::Relatime), 
            (AtimeOption::Strictatime, AtimeUpdateBehavior::Strictatime),
            (AtimeOption::Noatime, AtimeUpdateBehavior::Noatime),
            (AtimeOption::Relatime, AtimeUpdateBehavior::Relatime),
            (AtimeOption::Nodiratime, AtimeUpdateBehavior::Noatime)
        )] option: (AtimeOption, AtimeUpdateBehavior),
    ) {
        assert_eq!(
            AtimeOption::to_atime_behavior(&[option.0, option.0]).unwrap(),
            option.1,
        );
    }

    #[rstest]
    fn test_atime_allowed_combinations(
        #[values(
            (AtimeOption::Atime, AtimeUpdateBehavior::Relatime),
            (AtimeOption::Relatime, AtimeUpdateBehavior::Relatime),
            (AtimeOption::Nodiratime, AtimeUpdateBehavior::NodiratimeRelatime),
        )] option: (AtimeOption, AtimeUpdateBehavior),
    ) {
        assert_eq!(
            option.1,
            AtimeOption::to_atime_behavior(&[AtimeOption::Atime, option.0]).unwrap(),
        );
    }

    #[rstest]
    fn test_atime_forbidden_combinations(
        #[values(
            (AtimeOption::Strictatime, "Cannot use both atime and strictatime"),
            (AtimeOption::Noatime, "Cannot use both noatime and atime"),
        )] option: (AtimeOption, &str),
    ) {
        assert_eq!(option.1, AtimeOption::to_atime_behavior(&[AtimeOption::Atime, option.0]).unwrap_err().to_string());
    }

    #[rstest]
    fn test_strictatime_allowed_combinations(
        #[values(
            (AtimeOption::Strictatime, AtimeUpdateBehavior::Strictatime),
            (AtimeOption::Nodiratime, AtimeUpdateBehavior::NodiratimeStrictatime),
        )] option: (AtimeOption, AtimeUpdateBehavior),
    ) {
        assert_eq!(
            option.1,
            AtimeOption::to_atime_behavior(&[AtimeOption::Strictatime, option.0]).unwrap(),
        );
    }

    #[rstest]
    fn test_strictatime_forbidden_combinations(
        #[values(
            (AtimeOption::Atime, "Cannot use both atime and strictatime"),
            (AtimeOption::Noatime, "Cannot use both noatime and strictatime"),
            (AtimeOption::Relatime, "Cannot use both relatime and strictatime"),
        )] option: (AtimeOption, &str),
    ) {
        assert_eq!(option.1, AtimeOption::to_atime_behavior(&[AtimeOption::Strictatime, option.0]).unwrap_err().to_string());
    }

    #[rstest]
    fn test_noatime_allowed_combinations(
        #[values(
            (AtimeOption::Noatime, AtimeUpdateBehavior::Noatime),
            (AtimeOption::Nodiratime, AtimeUpdateBehavior::Noatime),
        )] option: (AtimeOption, AtimeUpdateBehavior),
    ) {
        assert_eq!(
            option.1,
            AtimeOption::to_atime_behavior(&[AtimeOption::Noatime, option.0]).unwrap(),
        );
    }

    #[rstest]
    fn test_noatime_forbidden_combinations(
        #[values(
            (AtimeOption::Atime, "Cannot use both noatime and atime"),
            (AtimeOption::Strictatime, "Cannot use both noatime and strictatime"),
            (AtimeOption::Relatime, "Cannot use both noatime and relatime"),
        )] option: (AtimeOption, &str),
    ) {
        assert_eq!(option.1, AtimeOption::to_atime_behavior(&[AtimeOption::Noatime, option.0]).unwrap_err().to_string());
    }

    #[rstest]
    fn test_relatime_allowed_combinations(
        #[values(
            (AtimeOption::Relatime, AtimeUpdateBehavior::Relatime),
            (AtimeOption::Atime, AtimeUpdateBehavior::Relatime),
            (AtimeOption::Nodiratime, AtimeUpdateBehavior::NodiratimeRelatime),
        )] option: (AtimeOption, AtimeUpdateBehavior),
    ) {
        assert_eq!(
            option.1,
            AtimeOption::to_atime_behavior(&[AtimeOption::Relatime, option.0]).unwrap(),
        );
    }

    #[rstest]
    fn test_relatime_forbidden_combinations(
        #[values(
            (AtimeOption::Strictatime, "Cannot use both relatime and strictatime"),
            (AtimeOption::Noatime, "Cannot use both noatime and relatime"),
        )] option: (AtimeOption, &str),
    ) {
        assert_eq!(option.1, AtimeOption::to_atime_behavior(&[AtimeOption::Relatime, option.0]).unwrap_err().to_string());
    }

    #[rstest]
    fn test_nodiratime_allowed_combinations(
        #[values(
            (AtimeOption::Nodiratime, AtimeUpdateBehavior::Noatime),
            (AtimeOption::Noatime, AtimeUpdateBehavior::Noatime),
            (AtimeOption::Strictatime, AtimeUpdateBehavior::NodiratimeStrictatime),
            (AtimeOption::Atime, AtimeUpdateBehavior::NodiratimeRelatime),
            (AtimeOption::Relatime, AtimeUpdateBehavior::NodiratimeRelatime),
        )] option: (AtimeOption, AtimeUpdateBehavior),
    ) {
        assert_eq!(
            option.1,
            AtimeOption::to_atime_behavior(&[AtimeOption::Nodiratime, option.0]).unwrap(),
        );
    }

}
