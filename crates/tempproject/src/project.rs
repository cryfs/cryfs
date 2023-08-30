use assert_cmd::Command;
use tempdir::TempDir;

#[derive(Debug)]
pub struct TempProject {
    folder: TempDir,
}

impl TempProject {
    pub(crate) fn new(folder: TempDir) -> Self {
        Self { folder }
    }

    pub fn run(&self) -> Command {
        let mut command = Command::new(env!("CARGO"));
        command.current_dir(self.folder.path()).arg("run").arg("--");
        command
    }
}
