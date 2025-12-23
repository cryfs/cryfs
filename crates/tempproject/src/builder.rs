use anyhow::Result;
use tempdir::TempDir;

use super::project::TempProject;

/// A builder for creating temporary Cargo projects.
///
/// `TempProjectBuilder` uses the builder pattern to configure a temporary
/// Cargo project with custom `Cargo.toml` and `main.rs` contents. The project
/// is created in a temporary directory that is automatically cleaned up when
/// the resulting [`TempProject`] is dropped.
///
/// # Example
///
/// ```no_run
/// use tempproject::TempProjectBuilder;
///
/// let project = TempProjectBuilder::new()?
///     .cargo(r#"
///         [package]
///         name = "my-test-project"
///         version = "0.1.0"
///         edition = "2021"
///     "#)
///     .main(r#"
///         fn main() {
///             println!("It works!");
///         }
///     "#)
///     .build()?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Panics
///
/// The [`build`](Self::build) method will panic if either [`cargo`](Self::cargo)
/// or [`main`](Self::main) has not been called.
#[derive(Debug)]
pub struct TempProjectBuilder {
    folder: TempDir,
    cargo: Option<String>,
    main: Option<String>,
}

impl TempProjectBuilder {
    /// Creates a new `TempProjectBuilder` with an empty temporary directory.
    ///
    /// The temporary directory is created in the system's default temp location
    /// and will be automatically deleted when the resulting [`TempProject`] is
    /// dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    ///
    /// let builder = TempProjectBuilder::new()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        Ok(Self {
            folder: TempDir::new("tempproject")?,
            cargo: None,
            main: None,
        })
    }

    /// Sets the contents of the project's `Cargo.toml` file.
    ///
    /// This method must be called before [`build`](Self::build).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    ///
    /// let builder = TempProjectBuilder::new()?
    ///     .cargo(r#"
    ///         [package]
    ///         name = "example"
    ///         version = "0.1.0"
    ///         edition = "2021"
    ///
    ///         [dependencies]
    ///         serde = "1.0"
    ///     "#);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn cargo(self, cargo_toml: impl Into<String>) -> Self {
        Self {
            cargo: Some(cargo_toml.into()),
            ..self
        }
    }

    /// Sets the contents of the project's `src/main.rs` file.
    ///
    /// This method must be called before [`build`](Self::build).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    ///
    /// let builder = TempProjectBuilder::new()?
    ///     .cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"")
    ///     .main(r#"
    ///         use std::env;
    ///
    ///         fn main() {
    ///             let args: Vec<String> = env::args().collect();
    ///             println!("Arguments: {:?}", args);
    ///         }
    ///     "#);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn main(self, main_rs: impl Into<String>) -> Self {
        Self {
            main: Some(main_rs.into()),
            ..self
        }
    }

    /// Builds the temporary project by writing all files to disk.
    ///
    /// This creates the `Cargo.toml` and `src/main.rs` files in the temporary
    /// directory and returns a [`TempProject`] that can be used to build and
    /// run the project.
    ///
    /// # Errors
    ///
    /// Returns an error if file I/O operations fail.
    ///
    /// # Panics
    ///
    /// Panics if [`cargo`](Self::cargo) or [`main`](Self::main) has not been
    /// called before this method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    ///
    /// let project = TempProjectBuilder::new()?
    ///     .cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"")
    ///     .main("fn main() { println!(\"Hello!\"); }")
    ///     .build()?;
    ///
    /// // Now you can build or run the project
    /// let executable = project.build_debug()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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
