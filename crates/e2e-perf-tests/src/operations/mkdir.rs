use crate::filesystem_test_ext::FilesystemTestExt as _;
use crate::rstest::FixtureFactory;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blockstore::ActionCounts;
use cryfs_rustfs::AbsolutePath;
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
            exists: 1, // Check if a blob with the new blob id already exists before creating it.
            loaded: 0,
            stored: 2, // Create new directory blob and add an entry for it to the root blob.
            removed: 0,
            created: 0,
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_rootdir(fixture: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    let fixture = fixture.create_filesystem(atime_behavior).await;

    // First create it so that it already exists
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/existing").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/existing").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            exists: 1, // Check if a blob with the new blob id already exists before creating it.
            loaded: 1, // TODO What are we loading here? The root dir should already be cached in the device.
            stored: 0,
            removed: 0,
            created: 0,
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_nesteddir(
    fixture: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture.create_filesystem(atime_behavior).await;

    // First create the nested dir
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested/notexisting").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            exists: 1, // Check if a blob with the new blob id already exists before creating it.
            loaded: 2, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
            stored: 3, // Create new directory blob and add an entry for it to the parent dir and update parent dir timestamps in the root blob.
            removed: 0,
            created: 0,
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_nesteddir(
    fixture: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture.create_filesystem(atime_behavior).await;

    // First create the nested dir
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested").unwrap())
                .await
                .unwrap();
        })
        .await;

    // Then create the dir so it's already existing
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested/existing").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested/existing").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            exists: 1, // Check if a blob with the new blob id already exists before creating it.
            loaded: 2, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
            stored: 1, // TODO What are we storing here? We didn't make any changes.
            removed: 0,
            created: 0,
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_deeplynesteddir(
    fixture: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture.create_filesystem(atime_behavior).await;

    // First create the nested dir
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3/notexisting").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            exists: 1, // Check if a blob with the new blob id already exists before creating it.
            loaded: 4, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
            stored: 3, // Create new directory blob and add an entry for it to the parent dir and update parent dir timestamps in the root blob.
            removed: 0,
            created: 0,
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_deeplynesteddir(
    fixture: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture.create_filesystem(atime_behavior).await;

    // First create the nested dir
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
        })
        .await;

    // Then create the dir so it's already existing
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3/existing").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3/existing").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            exists: 1, // Check if a blob with the new blob id already exists before creating it.
            loaded: 4, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
            stored: 1, // TODO What are we storing here? We didn't make any changes.
            removed: 0,
            created: 0,
        }
    );
}
