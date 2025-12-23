use assert_cmd::Command;
use is_executable::IsExecutable;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use std::sync::OnceLock;
use tempdir::TempDir;
use thiserror::Error;

/// Error returned when a cargo build or process execution fails.
///
/// This error captures the exit code and output streams from a failed process,
/// making it easier to diagnose build failures or runtime errors.
///
/// # Example
///
/// ```no_run
/// use tempproject::TempProjectBuilder;
///
/// let project = TempProjectBuilder::new()?
///     .cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"")
///     .main("fn main() { compile_error!(\"oops\"); }")
///     .build()?;
///
/// let err = project.build_debug().unwrap_err();
/// println!("Exit code: {:?}", err.exit_code);
/// if let Ok(stderr) = &err.stderr {
///     println!("Compiler output: {}", stderr);
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Error, Debug, Clone)]
#[error("Process exited with code {exit_code:?}\n\nstdout:\n{stdout:?}\n\nstderr:\n{stderr:?}")]
pub struct ProcessError {
    /// The exit code of the failed process, if available.
    ///
    /// This may be `None` if the process was terminated by a signal.
    pub exit_code: Option<i32>,

    /// The standard output of the failed process.
    ///
    /// Contains `Err` if the output was not valid UTF-8.
    pub stdout: Result<String, FromUtf8Error>,

    /// The standard error output of the failed process.
    ///
    /// For build failures, this typically contains compiler error messages.
    /// Contains `Err` if the output was not valid UTF-8.
    pub stderr: Result<String, FromUtf8Error>,
}

/// A temporary Cargo project that can be built and run.
///
/// `TempProject` represents a temporary Cargo project created by
/// [`TempProjectBuilder`](crate::TempProjectBuilder). It provides methods to
/// build and run the project in either debug or release mode.
///
/// # Build Caching
///
/// Build results are cached internally using [`OnceLock`]. This means:
/// - Calling [`build_debug`](Self::build_debug) multiple times only compiles once
/// - Calling [`run_debug`](Self::run_debug) will reuse a previous debug build
/// - Debug and release builds are cached independently
///
/// # Automatic Cleanup
///
/// The temporary directory containing the project is automatically deleted when
/// the `TempProject` is dropped.
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
/// // Build in debug mode
/// let executable_path = project.build_debug()?;
/// println!("Built: {:?}", executable_path);
///
/// // Run in debug mode (reuses the cached build)
/// let mut cmd = project.run_debug()?;
/// cmd.assert().success().stdout("Hello!\n");
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug)]
pub struct TempProject {
    folder: TempDir,
    executable_debug: OnceLock<Result<PathBuf, ProcessError>>,
    executable_release: OnceLock<Result<PathBuf, ProcessError>>,
}

impl TempProject {
    pub(crate) fn new(folder: TempDir) -> Self {
        Self {
            folder,
            executable_debug: OnceLock::new(),
            executable_release: OnceLock::new(),
        }
    }

    fn target_dir(&self) -> PathBuf {
        self.folder.path().join("target")
    }

    /// Builds the project in debug mode and returns the path to the executable.
    ///
    /// This runs `cargo build` in the project directory. The build result is
    /// cached, so subsequent calls return immediately with the cached result.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessError`] if the build fails, containing the compiler's
    /// exit code and error output.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    /// use assert_cmd::Command;
    ///
    /// let project = TempProjectBuilder::new()?
    ///     .cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"")
    ///     .main("fn main() { println!(\"Built!\"); }")
    ///     .build()?;
    ///
    /// let executable = project.build_debug()?;
    /// Command::new(&executable).assert().success();
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn build_debug(&self) -> Result<PathBuf, ProcessError> {
        let result = self._build_debug();
        let _ignore_error = self.executable_debug.set(result.clone());
        result
    }

    fn _build_debug(&self) -> Result<PathBuf, ProcessError> {
        let mut command = Command::new(env!("CARGO"));
        let command = command
            .current_dir(self.folder.path())
            .arg("build")
            .arg("--target-dir")
            .arg(self.target_dir())
            .assert();
        let output = command.get_output();
        if output.status.success() {
            Ok(find_single_binary_in(&self.target_dir().join("debug")))
        } else {
            Err(ProcessError {
                exit_code: output.status.code(),
                stdout: String::from_utf8(output.stdout.clone()),
                stderr: String::from_utf8(output.stderr.clone()),
            })
        }
    }

    /// Builds the project in release mode and returns the path to the executable.
    ///
    /// This runs `cargo build --release` in the project directory. The build
    /// result is cached, so subsequent calls return immediately with the cached
    /// result.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessError`] if the build fails, containing the compiler's
    /// exit code and error output.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    /// use assert_cmd::Command;
    ///
    /// let project = TempProjectBuilder::new()?
    ///     .cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"")
    ///     .main("fn main() { println!(\"Optimized!\"); }")
    ///     .build()?;
    ///
    /// let executable = project.build_release()?;
    /// Command::new(&executable).assert().success();
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn build_release(&self) -> Result<PathBuf, ProcessError> {
        let result = self._build_release();
        let _ignore_error = self.executable_release.set(result.clone());
        result
    }

    fn _build_release(&self) -> Result<PathBuf, ProcessError> {
        let mut command = Command::new(env!("CARGO"));
        let command = command
            .current_dir(self.folder.path())
            .arg("build")
            .arg("--release")
            .arg("--target-dir")
            .arg(self.target_dir())
            .assert();
        let output = command.get_output();
        if output.status.success() {
            Ok(find_single_binary_in(&self.target_dir().join("release")))
        } else {
            Err(ProcessError {
                exit_code: output.status.code(),
                stdout: String::from_utf8(output.stdout.clone()),
                stderr: String::from_utf8(output.stderr.clone()),
            })
        }
    }

    /// Builds (if necessary) and returns a [`Command`] to run the debug executable.
    ///
    /// If the project hasn't been built yet, this method builds it first. The
    /// returned [`Command`] from the `assert_cmd` crate can be used to run the
    /// executable and make assertions about its output.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessError`] if the build fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    ///
    /// let project = TempProjectBuilder::new()?
    ///     .cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"")
    ///     .main(r#"
    ///         fn main() {
    ///             let args: Vec<String> = std::env::args().collect();
    ///             println!("Args: {:?}", args);
    ///         }
    ///     "#)
    ///     .build()?;
    ///
    /// // Run with arguments and check output
    /// project.run_debug()?
    ///     .arg("--help")
    ///     .assert()
    ///     .success()
    ///     .stdout(predicates::str::contains("--help"));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn run_debug(&self) -> Result<Command, ProcessError> {
        let executable = match self.executable_debug.get_or_init(|| self._build_debug()) {
            Ok(executable) => executable,
            Err(err) => return Err(err.clone()),
        };

        let mut command = Command::new(executable);
        command.current_dir(self.folder.path());
        Ok(command)
    }

    /// Builds (if necessary) and returns a [`Command`] to run the release executable.
    ///
    /// If the project hasn't been built in release mode yet, this method builds
    /// it first. The returned [`Command`] from the `assert_cmd` crate can be used
    /// to run the executable and make assertions about its output.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessError`] if the build fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tempproject::TempProjectBuilder;
    ///
    /// let project = TempProjectBuilder::new()?
    ///     .cargo("[package]\nname = \"test\"\nversion = \"0.1.0\"")
    ///     .main("fn main() { println!(\"Fast!\"); }")
    ///     .build()?;
    ///
    /// // Run release build and verify output
    /// project.run_release()?
    ///     .assert()
    ///     .success()
    ///     .stdout("Fast!\n");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn run_release(&self) -> Result<Command, ProcessError> {
        let executable = match self
            .executable_release
            .get_or_init(|| self._build_release())
        {
            Ok(executable) => executable,
            Err(err) => return Err(err.clone()),
        };

        let mut command = Command::new(executable);
        command.current_dir(self.folder.path());
        Ok(command)
    }
}

fn find_single_binary_in(path: &Path) -> PathBuf {
    let mut binaries = std::fs::read_dir(path).unwrap().filter_map(|entry| {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            if path.is_executable() {
                Some(entry.path())
            } else {
                None
            }
        } else {
            None
        }
    });
    let Some(binary) = binaries.next() else {
        panic!("`cargo build` produced no binaries");
    };
    if let Some(second_binary) = binaries.next() {
        panic!(
            "`cargo build` produced at least two binaries: {:?} and {:?}",
            binary, second_binary,
        );
    }
    binary
}
