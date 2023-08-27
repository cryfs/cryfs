use anyhow::Result;

pub struct Cli {}

impl Cli {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn main(&self) -> Result<()> {
        println!("Hello cryfs-recover");

        Ok(())
    }
}
