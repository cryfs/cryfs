use path_absolutize::*;
use std::path::{Path, PathBuf};

/// Clap argument parser to parse a path given as a command line argument and absolutize it.
///
/// Example:
/// ```
/// use std::path::PathBuf;
/// use clap::Parser;
/// use cryfs_cli_utils::parse_path;
///
/// #[derive(Parser, Debug)]
/// pub struct CryfsArgs {
///    #[arg(value_parser=parse_path)]
///    pub basedir: PathBuf,
/// }
/// ```
pub fn parse_path(s: &str) -> Result<PathBuf, String> {
    Path::new(s)
        .absolutize()
        .map(|a| a.into_owned())
        .map_err(|e| e.to_string())
}

// TODO Tests
