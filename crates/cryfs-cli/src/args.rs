use clap::{Args, Parser};
use cryfs_cli_utils::parse_path;
use std::path::PathBuf;

// TODO Evaluate `clap_mangen` as a potential automatic manpage generator
// TODO Evaluate `clap_complete` as a potenail shell completion generator

#[derive(Parser, Debug)]
pub struct CryfsArgs {
    #[command(flatten)]
    pub mount: Option<MountArgs>,

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
    #[arg(value_parser=parse_path)]
    pub basedir: PathBuf,

    #[arg(value_parser=parse_path)]
    pub mountdir: PathBuf,
}

// TODO Tests
