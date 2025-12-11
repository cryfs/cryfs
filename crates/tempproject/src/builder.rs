use anyhow::Result;
use tempdir::TempDir;

use super::project::TempProject;

#[derive(Debug)]
pub struct TempProjectBuilder {
    folder: TempDir,
    cargo: Option<String>,
    main: Option<String>,
}

impl TempProjectBuilder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            folder: TempDir::new("tempproject")?,
            cargo: None,
            main: None,
        })
    }

    pub fn cargo(self, cargo_toml: impl Into<String>) -> Self {
        Self {
            cargo: Some(cargo_toml.into()),
            ..self
        }
    }

    pub fn main(self, main_rs: impl Into<String>) -> Self {
        Self {
            main: Some(main_rs.into()),
            ..self
        }
    }

    pub fn build(self) -> Result<TempProject> {
        self._build_cargo_toml()?;
        self._build_main_rs()?;
        Ok(TempProject::new(self.folder))
    }

    fn _build_cargo_toml(&self) -> Result<()> {
        let Some(cargo) = &self.cargo else {
            panic!(
                "You must call TempProjectBuilder::cargo before calling TempProjectBuilder::build"
            );
        };
        std::fs::write(self.folder.path().join("Cargo.toml"), cargo)?;
        Ok(())
    }

    fn _build_main_rs(&self) -> Result<()> {
        let Some(main) = &self.main else {
            panic!(
                "You must call TempProjectBuilder::main before calling TempProjectBuilder::build"
            );
        };
        let src_dir = self.folder.path().join("src");
        std::fs::create_dir(&src_dir)?;
        std::fs::write(src_dir.join("main.rs"), main)?;
        Ok(())
    }
}
