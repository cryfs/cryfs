use clap::Parser;
use cryfs_cli_utils::parse_path;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct CryfsRecoverArgs {
    #[arg(value_parser=parse_path)]
    pub basedir: PathBuf,
}
