use crate::filesystem_driver::FilesystemDriver;
use crate::filesystem_fixture::ActionCounts;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;

crate::perf_test_macro::perf_test!(init, [init,]);

fn init(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_uninitialized_filesystem()
        .setup(async |fixture| {
            // No setup needed
        })
        .test_no_counter_reset(async |fixture, ()| {
            fixture.filesystem.init().await.unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_try_create: 1, // create root dir blob
                store_load: 1,       // load root dir blob for sanity checking the file system
                blob_read_all: 1, // read content of root dir blob for sanity checking the file system
                blob_read: 1, // read header of root dir blob for sanity checking the file system
                blob_write: 1, // write to root dir blob. TODO Why don't we directly create it with the data?
                blob_flush: 1, // flush root dir blob
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 15,
                blob_data_mut: 2,
                store_try_create: 1,
                store_flush_block: 1,
                store_overhead: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                exists: 1,
                store: 1,
                overhead: 1,
                ..LLActionCounts::ZERO
            },
        })
}
