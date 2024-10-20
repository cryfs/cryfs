use anyhow::Result;
use cryfs_cli_utils::CliError;

use crate::{background_process::BackgroundProcess, MountArgs};

pub enum Mounter {
    MountInForeground,
    MountInBackgroud { rpc: BackgroundProcess },
}

impl Mounter {
    pub fn run_in_foreground() -> Result<Mounter> {
        Ok(Mounter::MountInForeground)
    }

    pub fn run_in_background() -> Result<Mounter> {
        let rpc = BackgroundProcess::daemonize()?;
        Ok(Mounter::MountInBackgroud { rpc })
    }

    /// This function will block until the filesystem is unmounted if we're in foreground mode.
    /// In background mode, it will return after a successful mount.
    /// In both cases, it will call on_successful_mount if mounting is successful.
    pub async fn mount_filesystem(
        &mut self,
        mount_args: MountArgs,
        on_successfully_mounted: impl Fn() + Send + Sync,
    ) -> Result<(), CliError> {
        match self {
            Self::MountInForeground => {
                super::runner::mount_filesystem(mount_args, on_successfully_mounted).await
            }
            Self::MountInBackgroud { ref mut rpc } => {
                // TODO Make rpc async?
                rpc.mount_filesystem(mount_args)?;
                on_successfully_mounted();
                Ok(())
            }
        }
    }
}
