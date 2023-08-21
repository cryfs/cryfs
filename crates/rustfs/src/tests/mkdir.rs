#![allow(non_snake_case)]

use mockall::predicate::{always, eq};
use nix::errno::Errno;
use rstest::rstest;
use std::future::Future;
use std::time::{Duration, SystemTime};

use super::utils::{
    assert_request_info_is_correct, make_mock_filesystem, FilesystemDriver, MockHelper, Runner,
    ROOT_INO,
};
use crate::common::{
    AbsolutePath, FsError, FsResult, Gid, HandleWithGeneration, InodeNumber, Mode, NodeAttrs,
    NodeKind, NumBytes, PathComponent, RequestInfo, Uid,
};
use crate::low_level_api::ReplyEntry;

const SOME_INO: InodeNumber = InodeNumber::from_const(20434);

struct Fixture {
    parent_ino: InodeNumber,
}

async fn test_mkdir<'a, F>(
    path: &AbsolutePath,
    call: impl FnOnce(FilesystemDriver) -> F,
    expectation: impl FnOnce(
            &Fixture,
            &RequestInfo,
            InodeNumber,
            &PathComponent,
            Mode,
            u32,
        ) -> FsResult<ReplyEntry>
        + Send
        + 'static,
) where
    F: Future<Output = nix::Result<()>>,
{
    let (parent, name) = path.split_last().unwrap();

    let mut mock_filesystem = make_mock_filesystem();
    let mut mock_helper = MockHelper::new(&mut mock_filesystem);
    let parent_ino = mock_helper.expect_lookup_path_is_dir(parent);
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
        .return_once(move |req, ino, name, mode, umask| {
            expectation(&Fixture { parent_ino }, req, ino, name, mode, umask)
        });
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

fn pathcomp(path_component: &str) -> &PathComponent {
    PathComponent::try_from_str(path_component).unwrap()
}

mod arguments {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_request_info(
        #[values(path("/some_component"), path("/some/nested/path"))] path: &AbsolutePath,
    ) {
        test_mkdir(
            path,
            |driver| async move { driver.mkdir(&path, Mode::default()).await },
            move |_, req, _parent_ino, _name, mode, _umask| {
                assert_request_info_is_correct(req);
                mkdir_return_ok(mode)
            },
        )
        .await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_parent_ino(
        #[values(path("/some_component"), path("/some/nested/path"))] path: &AbsolutePath,
    ) {
        test_mkdir(
            path,
            |driver| async move { driver.mkdir(&path, Mode::default()).await },
            |fixture: &Fixture, _req, parent_ino, _name, mode, _umask| {
                assert_eq!(fixture.parent_ino, parent_ino);
                mkdir_return_ok(mode)
            },
        )
        .await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_name(
        #[values(path("/some_component"), path("/some/nested/path"))] path: &'static AbsolutePath,
    ) {
        test_mkdir(
            path,
            |driver| async move { driver.mkdir(&path, Mode::default()).await },
            |_, _req, _parent_ino, name, mode, _umask| {
                assert_eq!(path.split_last().unwrap().1, name);
                mkdir_return_ok(mode)
            },
        )
        .await;
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

        #[rstest]
        #[case(MODE_DEFAULT_WITHOUT_DIR_FLAG, MODE_DEFAULT_WITH_DIR_FLAG)]
        #[case(MODE_DEFAULT_WITH_DIR_FLAG, MODE_DEFAULT_WITH_DIR_FLAG)]
        #[case(MODE_MORE_COMPLEX_WITHOUT_DIR_FLAG, MODE_MORE_COMPLEX_WITH_DIR_FLAG)]
        #[case(MODE_MORE_COMPLEX_WITH_DIR_FLAG, MODE_MORE_COMPLEX_WITH_DIR_FLAG)]
        #[tokio::test]
        async fn test_mode(
            #[values(path("/some_component"), path("/some/nested/path"))]
            path: &'static AbsolutePath,
            #[case] mode_arg: Mode,
            #[case] expected_mode_return: Mode,
        ) {
            test_mkdir(
                &path,
                |driver| async move { driver.mkdir(&path, mode_arg).await },
                move |_, _req, _parent_ino, _name, mode, _umask| {
                    assert_eq!(expected_mode_return, mode);
                    mkdir_return_ok(mode)
                },
            )
            .await;
        }
    }

    // TODO What is umask and how to test it?
    // In my tests with fuser 0.12.0, umask is already auto-applied to mode and the `umask` argument is always `0`.
    // See https://github.com/cberner/fuser/issues/256
}

mod result {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_success(
        #[values(path("/some_component"), path("/some/nested/path"))] path: &AbsolutePath,
    ) {
        test_mkdir(
            &path,
            |driver| async move {
                driver.mkdir(&path, Mode::default()).await.unwrap();
                Ok(())
            },
            |_, _req, _parent_ino, _name, mode, _umask| mkdir_return_ok(mode),
        )
        .await;
    }

    #[rstest]
    // TODO Test other error codes (check mkdir man page)
    #[case(FsError::NotImplemented, libc::ENOSYS)]
    #[case(FsError::NodeAlreadyExists, libc::EEXIST)]
    #[case(FsError::InvalidPath, libc::EINVAL)]
    #[case(FsError::NodeAlreadyExists, libc::EEXIST)]
    #[tokio::test]
    async fn test_error_in_mkdir(
        #[values(path("/some_component"), path("/some/nested/path"))] path: &'static AbsolutePath,
        #[case] error: FsError,
        #[case] expected_error_code: libc::c_int,
    ) {
        test_mkdir(
            &path,
            |driver| async move {
                let result = driver.mkdir(&path, Mode::default()).await.unwrap_err();
                assert_eq!(Errno::from_i32(expected_error_code), result);
                Ok(())
            },
            |_, _req, _parent_ino, _name, _mode, _umask| Err(error),
        )
        .await;
    }

    mod error_before_mkdir {
        use super::*;

        #[rstest]
        #[case(FsError::NodeDoesNotExist, libc::ENOENT)]
        #[tokio::test]
        async fn error_in_parent_lookup(
            #[case] error: FsError,
            #[case] expected_error_code: libc::c_int,
        ) {
            let mut mock_filesystem = make_mock_filesystem();
            MockHelper::new(&mut mock_filesystem).expect_lookup_fail(
                ROOT_INO,
                pathcomp("some"),
                error,
            );
            let runner = Runner::start(mock_filesystem);
            let driver = runner.driver();
            let result = driver
                .mkdir(path("/some/nested/dir"), Mode::default())
                .await
                .unwrap_err();
            assert_eq!(Errno::from_i32(expected_error_code), result);
        }

        #[rstest]
        #[case(FsError::NotImplemented, libc::ENOSYS)]
        #[case(FsError::InvalidOperation, libc::EINVAL)]
        #[tokio::test]
        async fn error_in_self_lookup(
            #[case] error: FsError,
            #[case] expected_error_code: libc::c_int,
        ) {
            let mut mock_filesystem = make_mock_filesystem();
            let mut mock_helper = MockHelper::new(&mut mock_filesystem);
            let parent_ino = mock_helper.expect_lookup_path_is_dir(path("/some/nested"));
            mock_helper.expect_lookup_fail(parent_ino, pathcomp("dir"), error);
            let runner = Runner::start(mock_filesystem);
            let driver = runner.driver();
            let result = driver
                .mkdir(path("/some/nested/dir"), Mode::default())
                .await
                .unwrap_err();
            assert_eq!(Errno::from_i32(expected_error_code), result);
        }

        #[rstest]
        #[tokio::test]
        async fn already_exists(
            #[values(NodeKind::Dir, NodeKind::File, NodeKind::Symlink)] kind: NodeKind,
        ) {
            let mut mock_filesystem = make_mock_filesystem();
            MockHelper::new(&mut mock_filesystem)
                .expect_lookup_path_is_kind(path("/some/nested/dir"), kind);
            let runner = Runner::start(mock_filesystem);
            let driver = runner.driver();
            let result = driver
                .mkdir(path("/some/nested/dir"), Mode::default())
                .await
                .unwrap_err();
            assert_eq!(Errno::from_i32(libc::EEXIST), result);
        }

        #[tokio::test]
        async fn parent_is_a_file() {
            let mut mock_filesystem = make_mock_filesystem();
            MockHelper::new(&mut mock_filesystem).expect_lookup_path_is_file(path("/some/nested"));
            let runner = Runner::start(mock_filesystem);
            let driver = runner.driver();
            let result = driver
                .mkdir(path("/some/nested/dir"), Mode::default())
                .await
                .unwrap_err();
            assert_eq!(Errno::from_i32(libc::ENOTDIR), result);
        }
    }
}
