use assert_cmd::Command;
use is_executable::IsExecutable;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use std::sync::OnceLock;
use tempdir::TempDir;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("Process exited with code {exit_code:?}\n\nstdout:\n{stdout:?}\n\nstderr:\n{stderr:?}")]
pub struct ProcessError {
    pub exit_code: Option<i32>,
    pub stdout: Result<String, FromUtf8Error>,
    pub stderr: Result<String, FromUtf8Error>,
}

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

    pub fn run_debug(&self) -> Result<Command, ProcessError> {
        let executable = match self.executable_debug.get_or_init(|| self._build_debug()) {
            Ok(executable) => executable,
            Err(err) => return Err(err.clone()),
        };

        let mut command = Command::new(executable);
        command.current_dir(self.folder.path());
        Ok(command)
    }

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
