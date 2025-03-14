use clap::Parser;

use super::MountArgs;

// TODO Set env var names and values from the constants in env.rs
const FOOTER: &str = color_print::cstr!(
    r#"<yellow,bold>Environment variables:</yellow,bold>
  <cyan>CRYFS_FRONTEND=</cyan><bright-cyan>noninteractive</bright-cyan>
	Work better together with tools.
	With this option set, CryFS won't ask anything, but use default values
	for options you didn't specify on command line. Furthermore, it won't
	ask you to enter a new password a second time (password confirmation).
  <cyan>CRYFS_NO_UPDATE_CHECK=</cyan><bright-cyan>true</bright-cyan>
	By default, CryFS connects to the internet to check for known
	security vulnerabilities and new versions. This option disables this.
  <cyan>CRYFS_LOCAL_STATE_DIR=</cyan><bright-cyan>[path]</bright-cyan>
	Sets the directory cryfs uses to store local state. This local state
	is used to recognize known file systems and run integrity checks,
	i.e. check that they haven't been modified by an attacker.
	Default value: /home/heinzi/.local/share/cryfs
"#
);

#[derive(Parser, Debug)]
#[command(after_help = FOOTER)]
pub struct CryfsArgs {
    #[command(flatten)]
    pub mount: Option<MountArgs>,

    /// Show a list with the supported encryption ciphers.
    #[arg(long, group = "immediate-exit", conflicts_with("mount"))]
    pub show_ciphers: bool,
    // TODO C++ had this, needed for Windows?
    // bool _mountDirIsDriveLetter;
}
