use anyhow::Result;
use clap::Parser;

use crate::args::Args;

pub struct Cli {}

impl Cli {
    pub fn new() -> Self {
        Self {}
    }

    pub fn main(&self) -> Result<()> {
        let args = Args::parse();
        Ok(())
    }
}
