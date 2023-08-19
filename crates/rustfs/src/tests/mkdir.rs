use mockall::predicate::{always, eq};
use std::time::{Duration, SystemTime};

use super::utils::{make_mock_filesystem, Runner};
use crate::common::{
    FsError, Gid, HandleWithGeneration, InodeNumber, Mode, NodeAttrs, NumBytes, PathComponent, Uid,
};
use crate::low_level_api::ReplyEntry;

const ROOT_INO: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID);
const SOME_INO: InodeNumber = InodeNumber::from_const(14034);
fn some_path_component() -> &'static PathComponent {
    PathComponent::try_from_str("some_path_component").unwrap()
}

#[tokio::test]
async fn simple() {
    let mut mock_filesystem = make_mock_filesystem();
    mock_filesystem.expect_access().returning(|_, _, _| Ok(()));
    mock_filesystem
        .expect_lookup()
        .once()
        .with(always(), eq(ROOT_INO), eq(some_path_component()))
        .returning(|_, _, _| Err(FsError::NodeDoesNotExist));
    mock_filesystem
        .expect_mkdir()
        .once()
        .with(
            always(),
            eq(ROOT_INO),
            eq(some_path_component()),
            always(),
            always(),
        )
        .returning(|_, _, _, mode, _| {
            let now = SystemTime::now();
            Ok(ReplyEntry {
                ino: HandleWithGeneration {
                    handle: SOME_INO,
                    generation: 0,
                },
                attr: NodeAttrs {
                    nlink: 1,
                    mode,
                    uid: Uid::from(1000),
                    gid: Gid::from(1000),
                    num_bytes: NumBytes::from(532),
                    num_blocks: None,
                    atime: now,
                    mtime: now,
                    ctime: now,
                },
                ttl: Duration::from_secs(1),
            })
        });
    let runner = Runner::start(mock_filesystem);
    let driver = runner.driver();
    driver
        .mkdir(&format!("/{}", some_path_component()), Mode::default())
        .await
        .unwrap();
}
