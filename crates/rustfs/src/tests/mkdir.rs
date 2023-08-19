use mockall::{
    predicate::{always, eq},
    Predicate,
};
use std::future::Future;
use std::time::{Duration, SystemTime};

use super::utils::{make_mock_filesystem, FilesystemDriver, Runner};
use crate::common::{
    FsError, Gid, HandleWithGeneration, InodeNumber, Mode, NodeAttrs, NumBytes, PathComponent,
    RequestInfo, Uid,
};
use crate::low_level_api::ReplyEntry;

const ROOT_INO: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID);
const SOME_INO: InodeNumber = InodeNumber::from_const(14034);
fn some_path_component() -> &'static PathComponent {
    PathComponent::try_from_str("some_path_component").unwrap()
}

async fn test_mkdir<'a, F>(
    call: impl FnOnce(FilesystemDriver) -> F,
    expectation: impl FnOnce(&RequestInfo, InodeNumber, &PathComponent, Mode, u32) + Send + 'static,
) where
    F: Future<Output = nix::Result<()>>,
{
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
        .with(always(), eq(ROOT_INO), always(), always(), always())
        .return_once(|req, ino, name, mode, umask| {
            expectation(req, ino, name, mode, umask);

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
    call(driver).await.unwrap();
}

mod arguments {
    use super::*;

    mod request_info {
        use super::*;

        // TODO Tests
    }

    mod parent {
        use super::*;

        // TODO Test with root
        // TODO Test with non-root ino
    }

    mod name {
        use super::*;

        #[tokio::test]
        async fn name() {
            test_mkdir(
                |driver| async move {
                    driver
                        .mkdir(&format!("/{}", some_path_component()), Mode::default())
                        .await
                },
                |_req, _ino, name, _mode, _umask| {
                    assert_eq!(some_path_component(), name);
                },
            )
            .await;
        }

        // TODO More name tests
    }

    mod mode {
        use super::*;

        async fn test_mode(mode_arg: Mode, expected_mode_return: Mode) {
            test_mkdir(
                move |driver| async move {
                    driver
                        .mkdir(&format!("/{}", some_path_component()), mode_arg)
                        .await
                },
                move |_req, _ino, _name, mode, _umask| {
                    assert_eq!(expected_mode_return, mode);
                },
            )
            .await;
        }

        #[tokio::test]
        async fn default_without_dir_flag() {
            test_mode(Mode::default(), Mode::default().add_dir_flag()).await;
        }

        #[tokio::test]
        async fn default_with_dir_flag() {
            test_mode(
                Mode::default().add_dir_flag(),
                Mode::default().add_dir_flag(),
            )
            .await;
        }

        // TODO More mode tests
    }

    // TODO What is umask and how to test it?
}
