#![allow(non_snake_case)]

use mockall::predicate::{always, eq};
use std::future::Future;
use std::time::{Duration, SystemTime};

use super::utils::{
    make_mock_filesystem, FilesystemDriver, MockAsyncFilesystemLL, MockHelper, Runner,
};
use crate::common::{
    AbsolutePath, AbsolutePathBuf, FsError, FsResult, Gid, HandleWithGeneration, InodeNumber, Mode,
    NodeAttrs, NumBytes, PathComponent, RequestInfo, Uid,
};
use crate::low_level_api::ReplyEntry;

const SOME_INO: InodeNumber = InodeNumber::from_const(20434);

async fn test_mkdir<'a, F>(
    path: &AbsolutePath,
    call: impl FnOnce(FilesystemDriver) -> F,
    expectation: impl FnOnce(&RequestInfo, InodeNumber, &PathComponent, Mode, u32) -> FsResult<ReplyEntry>
        + Send
        + 'static,
) where
    F: Future<Output = nix::Result<()>>,
{
    let (parent, name) = path.split_last().unwrap();

    let mut mock_filesystem = make_mock_filesystem();
    let mut mock_helper = MockHelper::new(&mut mock_filesystem);
    let parent_ino = mock_helper.expect_lookup_dir_path_exists(parent);
    mock_helper.expect_lookup_doesnt_exist(parent_ino, name);
    mock_filesystem
        .expect_mkdir()
        .once()
        .with(
            always(),
            eq(parent_ino),
            eq(name.to_owned()),
            always(),
            always(),
        )
        .return_once(expectation);
    let runner = Runner::start(mock_filesystem);
    let driver = runner.driver();
    call(driver).await.unwrap();
}

fn mkdir_return_ok(mode: Mode) -> FsResult<ReplyEntry> {
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
}

fn path(path: &str) -> &AbsolutePath {
    AbsolutePath::try_from_str(path).unwrap()
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

        async fn test_name(path: &'static AbsolutePath) {
            test_mkdir(
                &path,
                |driver| async move { driver.mkdir(&path, Mode::default()).await },
                |_req, _ino, name, mode, _umask| {
                    assert_eq!(path.split_last().unwrap().1, name);
                    mkdir_return_ok(mode)
                },
            )
            .await;
        }

        #[tokio::test]
        async fn givenRootLevelNode() {
            test_name(path("/some_component")).await;
        }

        #[tokio::test]
        async fn givenNestedNode() {
            test_name(path("/some/nested/path")).await;
        }
    }

    mod mode {
        use super::*;

        const MODE_DEFAULT_WITHOUT_DIR_FLAG: Mode = Mode::default_const();
        const MODE_DEFAULT_WITH_DIR_FLAG: Mode = MODE_DEFAULT_WITHOUT_DIR_FLAG.add_dir_flag();
        const MODE_MORE_COMPLEX_WITHOUT_DIR_FLAG: Mode = Mode::default_const()
            .add_user_read_flag()
            .add_user_write_flag()
            .add_user_exec_flag()
            .add_group_read_flag()
            .add_group_write_flag()
            .add_other_read_flag();
        const MODE_MORE_COMPLEX_WITH_DIR_FLAG: Mode =
            MODE_MORE_COMPLEX_WITHOUT_DIR_FLAG.add_dir_flag();

        async fn test_mode(path: &AbsolutePath, mode_arg: Mode, expected_mode_return: Mode) {
            test_mkdir(
                &path,
                |driver| async move { driver.mkdir(&path, mode_arg).await },
                move |_req, _ino, _name, mode, _umask| {
                    assert_eq!(expected_mode_return, mode);
                    mkdir_return_ok(mode)
                },
            )
            .await;
        }

        #[tokio::test]
        async fn givenRootLevelNode_whenMkdirWithDefaultMode_withoutDirFlag() {
            test_mode(
                path("/some_component"),
                MODE_DEFAULT_WITHOUT_DIR_FLAG,
                MODE_DEFAULT_WITH_DIR_FLAG,
            )
            .await;
        }

        #[tokio::test]
        async fn givenRootLevelNode_whenMkdirWithDefaultMode_withDirFlag() {
            test_mode(
                path("/some_component"),
                MODE_DEFAULT_WITH_DIR_FLAG,
                MODE_DEFAULT_WITH_DIR_FLAG,
            )
            .await;
        }

        #[tokio::test]
        async fn givenRootLevelNode_whenMkdirWithMoreComplexMode_withoutDirFlag() {
            test_mode(
                path("/some_component"),
                MODE_MORE_COMPLEX_WITHOUT_DIR_FLAG,
                MODE_MORE_COMPLEX_WITH_DIR_FLAG,
            )
            .await;
        }

        #[tokio::test]
        async fn givenRootLevelNode_whenMkdirWithMoreComplexMode_withDirFlag() {
            test_mode(
                path("/some_component"),
                MODE_MORE_COMPLEX_WITH_DIR_FLAG,
                MODE_MORE_COMPLEX_WITH_DIR_FLAG,
            )
            .await;
        }

        #[tokio::test]
        async fn givenNestedNode_whenMkdirWithDefaultMode_withoutDirFlag() {
            test_mode(
                path("/some/nested/path"),
                MODE_DEFAULT_WITHOUT_DIR_FLAG,
                MODE_DEFAULT_WITH_DIR_FLAG,
            )
            .await;
        }

        #[tokio::test]
        async fn givenNestedNode_whenMkdirWithDefaultMode_withDirFlag() {
            test_mode(
                path("/some/nested/path"),
                MODE_DEFAULT_WITH_DIR_FLAG,
                MODE_DEFAULT_WITH_DIR_FLAG,
            )
            .await;
        }

        #[tokio::test]
        async fn givenNestedNode_whenMkdirWithMoreComplexMode_withoutDirFlag() {
            test_mode(
                path("/some/nested/path"),
                MODE_MORE_COMPLEX_WITHOUT_DIR_FLAG,
                MODE_MORE_COMPLEX_WITH_DIR_FLAG,
            )
            .await;
        }

        #[tokio::test]
        async fn givenNestedNode_whenMkdirWithMoreComplexMode_withDirFlag() {
            test_mode(
                path("/some/nested/path"),
                MODE_MORE_COMPLEX_WITH_DIR_FLAG,
                MODE_MORE_COMPLEX_WITH_DIR_FLAG,
            )
            .await;
        }
    }

    // TODO What is umask and how to test it?
}

// TODO Test returns, including ok and error
