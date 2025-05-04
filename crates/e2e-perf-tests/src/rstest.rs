use cryfs_rustfs::AtimeUpdateBehavior;
use rstest_reuse::{self, *};

use crate::{
    filesystem_driver::{
        FilesystemDriver, FusemtFilesystemDriver, FuserFilesystemDriver, WithInodeCache,
        WithoutInodeCache,
    },
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
    FuserWithInodeCache,
    FuserWithoutInodeCache,
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

pub struct LLFixtureWithInodeCache;
impl FixtureFactory for LLFixtureWithInodeCache {
    type Driver = FuserFilesystemDriver<WithInodeCache>;

    fn fixture_type(&self) -> FixtureType {
        FixtureType::FuserWithInodeCache
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

pub struct LLFixtureWithoutInodeCache;
impl FixtureFactory for LLFixtureWithoutInodeCache {
    type Driver = FuserFilesystemDriver<WithoutInodeCache>;

    fn fixture_type(&self) -> FixtureType {
        FixtureType::FuserWithoutInodeCache
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
    #[values(
        crate::rstest::HLFixture,
        crate::rstest::LLFixtureWithInodeCache,
        crate::rstest::LLFixtureWithoutInodeCache
    )]
    fixture_factory: impl FixtureFactory,
) {
}

#[template]
pub fn all_fuser_fixtures(
    #[values(
        crate::rstest::LLFixtureWithInodeCache,
        crate::rstest::LLFixtureWithoutInodeCache
    )]
    fixture_factory: impl FixtureFactory,
) {
}
