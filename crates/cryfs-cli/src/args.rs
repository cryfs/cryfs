use clap::{Args, Parser};
use cryfs_cli_utils::parse_path;
use std::path::PathBuf;

// TODO Evaluate `clap_mangen` as a potential automatic manpage generator
// TODO Evaluate `clap_complete` as a potenail shell completion generator

#[derive(Parser, Debug)]
pub struct CryfsArgs {
    #[command(flatten)]
    pub mount: Option<MountArgs>,

    /// Show a list with the supported encryption ciphers.
    #[arg(long, group = "immediate-exit", conflicts_with("mount"))]
    pub show_ciphers: bool,
    // TODO
    // boost::optional<boost::filesystem::path> _configFile;
    // bool _foreground;
    // bool _allowFilesystemUpgrade;
    // bool _allowReplacedFilesystem;
    // bool _createMissingBasedir;
    // bool _createMissingMountpoint;
    // boost::optional<double> _unmountAfterIdleMinutes;
    // boost::optional<boost::filesystem::path> _logFile;
    // boost::optional<std::string> _cipher;
    // boost::optional<uint32_t> _blocksizeBytes;
    // bool _allowIntegrityViolations;
    // boost::optional<bool> _missingBlockIsIntegrityViolation;
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
}

// TODO Tests
