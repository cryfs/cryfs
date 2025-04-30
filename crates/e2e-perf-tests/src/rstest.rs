use cryfs_blobstore::BlobStoreOnBlocks;
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AtimeUpdateBehavior,
    object_based_api::{ObjectBasedFsAdapter, ObjectBasedFsAdapterLL},
};
use cryfs_utils::async_drop::AsyncDropArc;
use rstest_reuse::{self, *};

use crate::{filesystem_test_ext::FilesystemTestExt, fixture::FilesystemFixture};

#[template]
pub fn all_atime_behaviors(
    #[values(
        AtimeUpdateBehavior::Noatime,
        AtimeUpdateBehavior::Strictatime,
        AtimeUpdateBehavior::Relatime,
        AtimeUpdateBehavior::NodiratimeRelatime,
        AtimeUpdateBehavior::NodiratimeStrictatime
    )]
    atime_behavior: AtimeUpdateBehavior,
) {
}

pub enum FixtureType {
    Fuser,
    Fusemt,
}

pub trait FixtureFactory {
    type Filesystem: FilesystemTestExt;

    fn fixture_type(&self) -> FixtureType;

    async fn create_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Filesystem>;

    async fn create_uninitialized_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Filesystem>;
}

pub struct HLFixture;
impl FixtureFactory for HLFixture {
    type Filesystem = ObjectBasedFsAdapter<
        CryDevice<
            AsyncDropArc<
                BlobStoreOnBlocks<
                    HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                >,
            >,
        >,
    >;
    fn fixture_type(&self) -> FixtureType {
        FixtureType::Fusemt
    }

    async fn create_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Filesystem> {
        FilesystemFixture::create_filesystem(atime_behavior).await
    }
    async fn create_uninitialized_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Filesystem> {
        FilesystemFixture::create_uninitialized_filesystem(atime_behavior).await
    }
}

pub struct LLFixture;
impl FixtureFactory for LLFixture {
    type Filesystem = ObjectBasedFsAdapterLL<
        CryDevice<
            AsyncDropArc<
                BlobStoreOnBlocks<
                    HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                >,
            >,
        >,
    >;
    fn fixture_type(&self) -> FixtureType {
        FixtureType::Fuser
    }
    async fn create_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Filesystem> {
        FilesystemFixture::create_filesystem(atime_behavior).await
    }
    async fn create_uninitialized_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Filesystem> {
        FilesystemFixture::create_uninitialized_filesystem(atime_behavior).await
    }
}

#[template]
pub fn all_fixtures(
    #[values(crate::rstest::HLFixture, crate::rstest::LLFixture)] fixture_factory: impl FixtureFactory,
) {
}
