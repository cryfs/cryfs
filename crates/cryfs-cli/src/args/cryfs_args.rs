use clap::Parser;
use std::fmt::Write;

use super::MountArgs;
use cryfs_cli_utils::{ENV_VARS_DOCUMENTATION, EnvVarDoc};

fn footer() -> String {
    let mut output = color_print::cformat!("<yellow,bold>Environment variables:</yellow,bold>\n");
    for EnvVarDoc {
        key,
        value,
        description,
    } in ENV_VARS_DOCUMENTATION
    {
        let description = description.replace('\n', "\n    ");
        color_print::cwrite!(
            &mut output,
            "  <cyan>{key}=</cyan><bright-cyan>{value}</bright-cyan>\n    {description}\n"
        )
        .unwrap();
    }
    output
}

#[derive(Parser, Debug)]
#[command(after_help = footer())]
pub struct CryfsArgs {
    #[command(flatten)]
    pub mount: Option<MountArgs>,

    /// Show a list with the supported encryption ciphers.
    #[arg(long, group = "immediate-exit", conflicts_with("mount"))]
    pub show_ciphers: bool,
    // TODO C++ had this, needed for Windows?
    // bool _mountDirIsDriveLetter;
}
