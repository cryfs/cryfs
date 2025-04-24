use crate::filesystem_test_ext::FilesystemTestExt as _;
use crate::rstest::FixtureFactory;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blockstore::ActionCounts;
use cryfs_rustfs::AtimeUpdateBehavior;
use rstest::rstest;
use rstest_reuse::apply;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_rootdir(
    fixture: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    use cryfs_rustfs::AbsolutePath;

    let fixture = fixture.create_filesystem(atime_behavior).await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/notexisting").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            exists: 1,
            loaded: 0,
            // Add new directory blob and add it to its parent.
            stored: 2,
            removed: 0,
            created: 0,
        }
    );
}

// TODO existing_from_rootdir
// TODO nonexisting_from_nested_dir
// TODO existing_from_nested_dir
