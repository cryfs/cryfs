use anyhow::Result;
use clap_logflag::LoggingConfig;
use cryfs_cli_utils::CliError;

use crate::{MountArgs, background_process::BackgroundProcess};

pub enum Mounter {
    MountInForeground,
    MountInBackgroud { rpc: BackgroundProcess },
}

impl Mounter {
    pub fn run_in_foreground() -> Result<Mounter> {
        Ok(Mounter::MountInForeground)
    }

    /// Spawn the daemon and ship `daemon_log_config` to it as the first
    /// IPC bootstrap message. `daemon_log_config` should already be
    /// resolved against the daemon-mode default (typically syslog) — the
    /// daemon installs this verbatim and does not re-resolve.
    pub fn run_in_background(daemon_log_config: LoggingConfig) -> Result<Mounter> {
        let rpc = BackgroundProcess::daemonize(daemon_log_config)?;
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
            Self::MountInBackgroud { rpc } => {
                // TODO Make rpc async?
                rpc.mount_filesystem(mount_args)?;
                on_successfully_mounted();
                Ok(())
            }
        }
    }
}
