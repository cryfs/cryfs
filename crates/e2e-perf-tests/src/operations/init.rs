use crate::filesystem_test_ext::FilesystemTestExt as _;
use crate::rstest::FixtureFactory;
use crate::rstest::all_atime_behaviors;
use crate::rstest::all_fixtures;
use cryfs_blockstore::{HLActionCounts, LLActionCounts};
use cryfs_rustfs::AtimeUpdateBehavior;
use rstest::rstest;
use rstest_reuse::apply;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn init(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    use crate::fixture::ActionCounts;

    let fixture = fixture_factory
        .create_uninitialized_filesystem(atime_behavior)
        .await;

    let mut counts = fixture.totals();

    counts += fixture
        .run_operation(async |fs| fs.init().await.unwrap())
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            low_level: LLActionCounts {
                exists: 1,
                loaded: 0,
                stored: 1,
                removed: 0,
                created: 0,
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                loaded: 2,
                read: 15,
                written: 2,
                overwritten: 0,
                created: 1,
                removed: 0,
                resized: 0,
                flushed: 1,
            },
        }
    );
}
