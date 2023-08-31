use assert_cmd::Command;
use is_executable::IsExecutable;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tempdir::TempDir;

#[derive(Debug)]
pub struct TempProject {
    folder: TempDir,
    executable: OnceLock<PathBuf>,
}

impl TempProject {
    pub(crate) fn new(folder: TempDir) -> Self {
        Self {
            folder,
            executable: OnceLock::new(),
        }
    }

    fn target_dir(&self) -> PathBuf {
        self.folder.path().join("target")
    }

    pub fn build(&self) -> PathBuf {
        let mut command = Command::new(env!("CARGO"));
        command
            .current_dir(self.folder.path())
            .arg("build")
            .arg("--target-dir")
            .arg(self.target_dir());
        command.assert().success();
        find_single_binary_in(&self.target_dir().join("debug"))
    }

    pub fn run(&self) -> Command {
        let executable = self.executable.get_or_init(|| self.build());
        let mut command = Command::new(executable);
        command.current_dir(self.folder.path());
        command
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
