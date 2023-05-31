use clap::Parser;
use path_absolutize::*;
use std::path::{Path, PathBuf};

// TODO Evaluate `clap_mangen` as a potential automatic manpage generator
// TODO Evaluate `clap_complete` as a potenail shell completion generator

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(value_parser=parse_path, group = "mount", required_unless_present("immediate-exit"))]
    pub basedir: Option<PathBuf>,

    #[arg(value_parser=parse_path, group = "mount", required_unless_present("immediate-exit"))]
    pub mountdir: Option<PathBuf>,

    #[arg(short = 'V', long, group = "immediate-exit", conflicts_with("mount"))]
    pub version: bool,

    #[arg(long, group = "immediate-exit", conflicts_with("mount"))]
    pub show_ciphers: bool,
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

fn parse_path(s: &str) -> Result<PathBuf, String> {
    Path::new(s)
        .absolutize()
        .map(|a| a.into_owned())
        .map_err(|e| e.to_string())
}

// TODO Tests
