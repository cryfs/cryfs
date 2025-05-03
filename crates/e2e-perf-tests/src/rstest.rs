use cryfs_rustfs::AtimeUpdateBehavior;
use rstest_reuse::{self, *};

use crate::{
    filesystem_driver::{FilesystemDriver, FusemtFilesystemDriver, FuserFilesystemDriver},
    fixture::FilesystemFixture,
};

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
    type Driver: FilesystemDriver;

    fn fixture_type(&self) -> FixtureType;

    async fn create_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Driver>;

    async fn create_uninitialized_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Driver>;
}

pub struct HLFixture;
impl FixtureFactory for HLFixture {
    type Driver = FusemtFilesystemDriver;

    fn fixture_type(&self) -> FixtureType {
        FixtureType::Fusemt
    }

    async fn create_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Driver> {
        FilesystemFixture::create_filesystem(atime_behavior).await
    }
    async fn create_uninitialized_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Driver> {
        FilesystemFixture::create_uninitialized_filesystem(atime_behavior).await
    }
}

pub struct LLFixture;
impl FixtureFactory for LLFixture {
    type Driver = FuserFilesystemDriver;

    fn fixture_type(&self) -> FixtureType {
        FixtureType::Fuser
    }
    async fn create_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Driver> {
        FilesystemFixture::create_filesystem(atime_behavior).await
    }
    async fn create_uninitialized_filesystem(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<Self::Driver> {
        FilesystemFixture::create_uninitialized_filesystem(atime_behavior).await
    }
}

#[template]
pub fn all_fixtures(
    #[values(crate::rstest::HLFixture, crate::rstest::LLFixture)] fixture_factory: impl FixtureFactory,
) {
}
