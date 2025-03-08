use clap::Parser;

use super::MountArgs;

#[derive(Parser, Debug)]
pub struct CryfsArgs {
    #[command(flatten)]
    pub mount: Option<MountArgs>,

    /// Show a list with the supported encryption ciphers.
    #[arg(long, group = "immediate-exit", conflicts_with("mount"))]
    pub show_ciphers: bool,
    // TODO
    // std::vector<std::string> _fuseOptions;
    // bool _mountDirIsDriveLetter;
}
