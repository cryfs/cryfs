use anyhow::Result;
use byte_unit::Byte;
use clap::{Args, Parser};
use cryfs_cli_utils::parse_path;
use std::path::PathBuf;

// TODO Evaluate `clap_mangen` as a potential automatic manpage generator
// TODO Evaluate `clap_complete` as a potenatial shell completion generator

#[derive(Parser, Debug)]
pub struct CryfsArgs {
    #[command(flatten)]
    pub mount: Option<MountArgs>,

    /// Show a list with the supported encryption ciphers.
    #[arg(long, group = "immediate-exit", conflicts_with("mount"))]
    pub show_ciphers: bool,
    // TODO
    // boost::optional<boost::filesystem::path> _configFile;
    // bool _allowFilesystemUpgrade;
    // bool _allowReplacedFilesystem;
    // boost::optional<boost::filesystem::path> _logFile;
    // std::vector<std::string> _fuseOptions;
    // bool _mountDirIsDriveLetter;
}

#[derive(Args, Debug)]
#[group(multiple = true, id = "mount")]
pub struct MountArgs {
    /// The directory containing the encrypted vault.
    #[arg(value_parser=parse_path)]
    pub basedir: PathBuf,

    /// The directory to mount the plaintext filesystem to.
    #[arg(value_parser=parse_path)]
    pub mountdir: PathBuf,

    /// Run CryFS in foreground mode, i.e. don't return to the shell until the filesystem is unmounted.
    #[arg(short, long)]
    pub foreground: bool,

    /// Creates the vault directory if it doesn't exist yet, skipping the normal confirmation message asking whether it should be created.
    #[arg(long)]
    pub create_missing_basedir: bool,

    /// Creates the mount directory if it doesn't exist yet, skipping the normal confirmation message asking whether it should be created.
    #[arg(long)]
    pub create_missing_mountpoint: bool,

    /// Whether to treat a missing block as an integrity violation. This makes sure you notice if an attacker deleted some of your files,
    /// but only works in single-client mode. You will not be able to use the file system on other devices.
    #[arg(long)]
    pub missing_block_is_integrity_violation: Option<bool>,

    /// Disable integrity checks. Integrity checks ensure that your file system was not manipulated or rolled back to an earlier version.
    /// Disabling them is needed if you want to load an old snapshot of your file system.
    #[arg(long)]
    pub allow_integrity_violations: bool,

    // TODO Make display of default cipher dynamic to show the actual default cipher
    /// Cipher to use for encryption.
    /// If creating a new file system, this will determine the cipher for the new file system.
    /// If opening an existing file system, this will verify that the file system actually uses that cipher.
    /// See possible values by calling cryfs with --show-ciphers. Default: xchacha20-poly1305
    #[arg(long)]
    pub cipher: Option<String>, // TODO This should probably be an enum

    // TODO Make display of default blocksize dynamic to show the actual default cipher
    /// The block size used when storing ciphertext blocks. Default: 16KiB
    /// If creating a new file system, this will determine the block size for the new file system.
    /// If opening an existing file system, this will verify that the file system actually uses that block size.
    #[arg(long, value_parser(parse_byte_amount))]
    pub blocksize: Option<Byte>,

    /// Automatically unmount if the file system hasn't been used for the specified duration.
    /// Values are human readable durations, e.g. 30sec, 5min, 1h30m, etc.
    #[arg(long)]
    pub unmount_idle: Option<humantime::Duration>,
}

fn parse_byte_amount(input: &str) -> Result<Byte> {
    Ok(Byte::parse_str(input, true)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod humantime_parsing {
        //! Test that durations in unmount_idle are parsed correctly

        #[test]
        fn test_human_time_parsing() {
            fn test_parsing(input: &str, expected_as_sec: u64) {
                let duration: humantime::Duration = input.parse().unwrap();
                assert_eq!(duration.as_secs(), expected_as_sec);
            }
            test_parsing("30s", 30);
            test_parsing("30sec", 30);
            test_parsing("5m", 300);
            test_parsing("5min", 300);
            test_parsing("1h30m", 5400);
            test_parsing("1h30min", 5400);
            test_parsing("1hour30min", 5400);
            test_parsing("2hours30min5sec", 9005);
        }
    }

    mod byte_parsing {
        //! Test that byte amounts in blocksize are parsed correctly

        use super::*;

        use byte_unit::Byte;

        #[test]
        fn test_byte_parsing() {
            fn test_parsing(input: &str, expected_as_bytes: u64) {
                let byte: Byte = parse_byte_amount(input).unwrap();
                assert_eq!(byte.as_u64(), expected_as_bytes);
            }

            test_parsing("16384", 16384);
            test_parsing("16384b", 16384);
            test_parsing("16384B", 16384);
            test_parsing("16KiB", 16384);
            test_parsing("16kib", 16384);
            test_parsing("16Kb", 16000);
            test_parsing("16kb", 16000);
            test_parsing("4MiB", 4 * 1024 * 1024)
        }
    }
}

// TODO Tests
