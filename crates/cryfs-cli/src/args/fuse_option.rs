use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FuseOption {
    /// Allow all users to access files on this filesystem. By default access is restricted to the user who mounted it.
    #[clap(rename_all = "snake_case")]
    AllowOther,

    /// Allow the root user to access this filesystem, in addition to the user who mounted it.
    #[clap(rename_all = "snake_case")]
    AllowRoot,
}

impl From<&FuseOption> for cryfs_runner::FuseOption {
    fn from(value: &FuseOption) -> Self {
        match value {
            FuseOption::AllowOther => cryfs_runner::FuseOption::AllowOther,
            FuseOption::AllowRoot => cryfs_runner::FuseOption::AllowRoot,
        }
    }
}
