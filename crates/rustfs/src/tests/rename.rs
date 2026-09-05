use cryfs_utils::path::{AbsolutePath, PathComponent};
use mockall::predicate::{always, eq};

use super::utils::{MockHelper, ROOT_INO, Runner, make_mock_filesystem};

fn path(path: &str) -> &AbsolutePath {
    AbsolutePath::try_from_str(path).unwrap()
}

fn pathcomp(path_component: &str) -> &PathComponent {
    PathComponent::try_from_str(path_component).unwrap()
}

/// `rename(2)` has no flags, and the kernel sends it as `FUSE_RENAME` rather than `FUSE_RENAME2`,
/// so the file system must see `flags == 0`. This is the case that has to keep working after we
/// started rejecting non-zero flags.
#[tokio::test]
async fn plain_rename_passes_no_flags() {
    let mut mock_filesystem = make_mock_filesystem();
    let mut mock_helper = MockHelper::new(&mut mock_filesystem.fs);
    mock_helper.expect_lookup_path_is_file(path("/source"));
    mock_helper.expect_lookup_doesnt_exist(ROOT_INO, pathcomp("target"));
    mock_filesystem
        .fs
        .expect_rename()
        .once()
        .with(
            always(),
            eq(ROOT_INO),
            eq(pathcomp("source").to_owned()),
            eq(ROOT_INO),
            eq(pathcomp("target").to_owned()),
            eq(0u32),
        )
        .return_once(|_, _, _, _, _, _| Ok(()));

    let runner = Runner::start(mock_filesystem).await;
    runner.driver().rename("/source", "/target").await.unwrap();
}

/// The `renameat2()` flags really do reach the file system - the kernel forwards them in
/// `FUSE_RENAME2` instead of handling them itself. That is what makes ignoring them dangerous:
/// a `RENAME_EXCHANGE` answered with a plain rename reports success while destroying one of the
/// two files.
///
/// These tests cover the path from the syscall to [crate::low_level_api::AsyncFilesystemLL::rename]
/// and back. That the object based API then refuses those flags is covered by the unit tests of
/// `reject_unsupported_rename_flags` in [crate::object_based_api]; here the file system is a mock,
/// so we assert on what it is handed and that its refusal arrives at the caller as `EINVAL`.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod renameat2_flags {
    use super::*;
    use crate::common::FsError;
    use nix::errno::Errno;
    use nix::fcntl::RenameFlags;

    /// The target must exist for `RENAME_EXCHANGE`, otherwise the VFS answers `ENOENT` without
    /// ever asking the file system.
    #[tokio::test]
    async fn exchange_reaches_the_filesystem_and_is_rejected_with_einval() {
        let mut mock_filesystem = make_mock_filesystem();
        let mut mock_helper = MockHelper::new(&mut mock_filesystem.fs);
        mock_helper.expect_lookup_path_is_file(path("/source"));
        mock_helper.expect_lookup_path_is_file(path("/target"));
        mock_filesystem
            .fs
            .expect_rename()
            .once()
            .with(
                always(),
                eq(ROOT_INO),
                eq(pathcomp("source").to_owned()),
                eq(ROOT_INO),
                eq(pathcomp("target").to_owned()),
                eq(libc::RENAME_EXCHANGE),
            )
            .return_once(|_, _, _, _, _, _| Err(FsError::InvalidOperation));

        let runner = Runner::start(mock_filesystem).await;
        let result = runner
            .driver()
            .renameat2("/source", "/target", RenameFlags::RENAME_EXCHANGE)
            .await;
        assert_eq!(Err(Errno::EINVAL), result);
    }

    /// The target must *not* exist for `RENAME_NOREPLACE`, otherwise the VFS answers `EEXIST`
    /// without ever asking the file system.
    #[tokio::test]
    async fn noreplace_reaches_the_filesystem_and_is_rejected_with_einval() {
        let mut mock_filesystem = make_mock_filesystem();
        let mut mock_helper = MockHelper::new(&mut mock_filesystem.fs);
        mock_helper.expect_lookup_path_is_file(path("/source"));
        mock_helper.expect_lookup_doesnt_exist(ROOT_INO, pathcomp("target"));
        mock_filesystem
            .fs
            .expect_rename()
            .once()
            .with(
                always(),
                eq(ROOT_INO),
                eq(pathcomp("source").to_owned()),
                eq(ROOT_INO),
                eq(pathcomp("target").to_owned()),
                eq(libc::RENAME_NOREPLACE),
            )
            .return_once(|_, _, _, _, _, _| Err(FsError::InvalidOperation));

        let runner = Runner::start(mock_filesystem).await;
        let result = runner
            .driver()
            .renameat2("/source", "/target", RenameFlags::RENAME_NOREPLACE)
            .await;
        assert_eq!(Err(Errno::EINVAL), result);
    }

    /// `renameat2()` without flags is a plain rename and must not be rejected.
    #[tokio::test]
    async fn empty_flags_are_a_plain_rename() {
        let mut mock_filesystem = make_mock_filesystem();
        let mut mock_helper = MockHelper::new(&mut mock_filesystem.fs);
        mock_helper.expect_lookup_path_is_file(path("/source"));
        mock_helper.expect_lookup_doesnt_exist(ROOT_INO, pathcomp("target"));
        mock_filesystem
            .fs
            .expect_rename()
            .once()
            .with(
                always(),
                eq(ROOT_INO),
                eq(pathcomp("source").to_owned()),
                eq(ROOT_INO),
                eq(pathcomp("target").to_owned()),
                eq(0u32),
            )
            .return_once(|_, _, _, _, _, _| Ok(()));

        let runner = Runner::start(mock_filesystem).await;
        runner
            .driver()
            .renameat2("/source", "/target", RenameFlags::empty())
            .await
            .unwrap();
    }
}
