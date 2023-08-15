use nix::sys::stat::Mode;
use std::path::Path;

use super::utils::{make_mock_filesystem, Runner};

#[tokio::test]
async fn simple() {
    // TODO This test doesn't work yet. It should fail in its current version because no mock is set up but it succeeds

    let mut mock_filesystem = make_mock_filesystem();
    // mock_filesystem.expect_access().returning(|_, _, _| Ok(()));
    let runner = Runner::start(mock_filesystem);
    let driver = runner.driver();
    driver.mkdir(Path::new("/foo"), Mode::empty());
}
