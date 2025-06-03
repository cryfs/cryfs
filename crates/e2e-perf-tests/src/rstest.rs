use cryfs_blockstore::{InMemoryBlockStore, LLBlockStore, OptimizedBlockStoreWriter};
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use rstest_reuse::{self, *};

use crate::{
    filesystem_driver::{
        FilesystemDriver, FusemtFilesystemDriver, FusemtMountingFilesystemDriver,
        FuserFilesystemDriver, FuserMountingFilesystemDriver, WithInodeCache, WithoutInodeCache,
    },
    fixture::FilesystemFixture,
};

// TODO If rust stabilizes custom test frameworks, we can make `perf_test` a per-test macro instead of taking multiple macro names listing all tests in a file. See https://bheisler.github.io/criterion.rs/book/user_guide/custom_test_framework.html

#[cfg(not(feature = "benchmark"))]
#[crabtime::function]
fn perf_test_(_group: String, names: Vec<String>, disable_fusemt: u8) {
    // TODO Deduplicate normalize_identifier
    fn normalize_identifier(name: String) -> String {
        name.replace(|c: char| !c.is_alphanumeric(), "_")
    }
    let mut fixtures = vec![
        ("ll_cache", "crate::rstest::LLFixtureWithInodeCache"),
        ("ll_nocache", "crate::rstest::LLFixtureWithoutInodeCache"),
    ];
    if disable_fusemt == 0 {
        fixtures.push(("hl", "crate::rstest::HLFixture"));
    }
    let atime_behaviors = [
        ("noatime", "cryfs_rustfs::AtimeUpdateBehavior::Noatime"),
        (
            "strictatime",
            "cryfs_rustfs::AtimeUpdateBehavior::Strictatime",
        ),
        ("relatime", "cryfs_rustfs::AtimeUpdateBehavior::Relatime"),
        (
            "nodiratimerelatime",
            "cryfs_rustfs::AtimeUpdateBehavior::NodiratimeRelatime",
        ),
        (
            "nodiratimestrictatime",
            "cryfs_rustfs::AtimeUpdateBehavior::NodiratimeStrictatime",
        ),
    ];
    for name in names {
        let name_str = normalize_identifier(name.clone());
        crabtime::output_str!(
            r#"
            mod test_{name_str} {{
                use super::*;
        "#
        );
        for (fixture_name, fixture_value) in &fixtures {
            for (atime_name, atime_value) in atime_behaviors {
                crabtime::output_str!(
                    r#"
                    #[test]
                    fn {fixture_name}_{atime_name}() {{
                        let fixture_factory = {fixture_value};
                        let atime_behavior = {atime_value};
                        let test_driver = crate::test_driver::TestDriverImpl::new(cryfs_blockstore::InMemoryBlockStore::new, fixture_factory, atime_behavior);
                        let test = {name}(test_driver);
                        test.assert_op_counts();
                    }}
                    "#
                );
            }
        }
        crabtime::output_str!(
            r#"
            }}"#
        );
    }
}

// TODO Is it better for perf_test in benchmark mode to just output one bench function that contains all of the benchmarks? Currently, we create a separate function for each test.
// TODO Test if macro_rules has better compile times here than crabtime.

#[cfg(feature = "benchmark")]
#[crabtime::function]
fn perf_test_(group: String, names: Vec<String>, disable_fusemt: u8) {
    // TODO Deduplicate normalize_identifier
    fn normalize_identifier(name: String) -> String {
        name.replace(|c: char| !c.is_alphanumeric(), "_")
    }
    for name in &names {
        let name_str = normalize_identifier(name.to_owned());
        let name_str_with_group = format!("\"{group}::{name_str}\"");
        crabtime::output! {
            fn bench_{{name_str}}(criterion: &mut criterion::Criterion) {
                let mut bench = criterion.benchmark_group({{name_str_with_group}});
                bench.sample_size(10);  // TODO Using a small sample size for now to speed up testing. Remove this later for real benchmarking!
                use cryfs_rustfs::AtimeUpdateBehavior;
                let atime_behaviors = [
                    ("noatime", AtimeUpdateBehavior::Noatime),
                    ("strictatime", AtimeUpdateBehavior::Strictatime),
                    ("relatime", AtimeUpdateBehavior::Relatime),
                    ("nodiratimerelatime", AtimeUpdateBehavior::NodiratimeRelatime),
                    ("nodiratimestrictatime", AtimeUpdateBehavior::NodiratimeStrictatime),
                ];
                for (atime_name, atime_value) in atime_behaviors {
                    // fuser
                    let test_driver = crate::test_driver::TestDriverImpl::new(cryfs_blockstore::TempDirBlockStore::new, crate::rstest::MountingFuserFixture, atime_value);
                    let test = {{name}}(test_driver);
                    bench.bench_function(&format!("fuser:{atime_name}"), move |b| {
                        test.run_benchmark(b);
                    });

                    // fusemt
                    if disable_fusemt == 0 {
                        let test_driver = crate::test_driver::TestDriverImpl::new(cryfs_blockstore::TempDirBlockStore::new, crate::rstest::MountingFusemtFixture, atime_value);
                        let test = {{name}}(test_driver);
                        bench.bench_function(&format!("fusemt:{atime_name}"), move |b| {
                            test.run_benchmark(b);
                        });
                    }
                }
            }
        }
    }
    crabtime::output_str!("criterion::criterion_group!(benches_{group}");
    for name in names {
        let name_str = normalize_identifier(name);
        crabtime::output_str!(", bench_{name_str}");
    }
    crabtime::output_str!(");");
}

macro_rules! perf_test {
    ($group:ident, $tests:tt) => {
        $crate::rstest::perf_test_!($group, $tests, 0);
    };
}

/// Like [perf_test!], but only runs the fuser tests, not fuse-mt.
macro_rules! perf_test_only_fuser {
    ($group:ident, $tests:tt) => {
        $crate::rstest::perf_test_!($group, $tests, 1);
    };
}

pub(crate) use perf_test;
pub(crate) use perf_test_;
pub(crate) use perf_test_only_fuser;

// TODO Remove the rstest templates here now that we have the `perf_test` macro

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

#[derive(Debug, Clone, Copy)]
pub enum FixtureType {
    FuserWithInodeCache,
    FuserWithoutInodeCache,
    Fusemt,
}

// TODO Can we remove FixtureFactory? Not sure what purpose it adds on top of FilesystemFixture / FilesystemDriver

pub trait FixtureFactory: 'static {
    type Driver: FilesystemDriver;

    fn fixture_type(&self) -> FixtureType;

    async fn create_filesystem<B>(
        &self,
        blockstore: AsyncDropGuard<B>,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<B, Self::Driver>
    where
        B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    {
        FilesystemFixture::create_filesystem(blockstore, atime_behavior).await
    }

    async fn create_uninitialized_filesystem<B>(
        &self,
        blockstore: AsyncDropGuard<B>,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<B, Self::Driver>
    where
        B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    {
        FilesystemFixture::create_uninitialized_filesystem(blockstore, atime_behavior).await
    }

    // TODO Remove
    async fn create_filesystem_deprecated(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<InMemoryBlockStore, Self::Driver> {
        self.create_filesystem(InMemoryBlockStore::new(), atime_behavior)
            .await
    }

    // TODO Remove
    async fn create_uninitialized_filesystem_deprecated(
        &self,
        atime_behavior: AtimeUpdateBehavior,
    ) -> FilesystemFixture<InMemoryBlockStore, Self::Driver> {
        self.create_uninitialized_filesystem(InMemoryBlockStore::new(), atime_behavior)
            .await
    }
}

pub struct HLFixture;
impl FixtureFactory for HLFixture {
    type Driver = FusemtFilesystemDriver;

    fn fixture_type(&self) -> FixtureType {
        FixtureType::Fusemt
    }
}

pub struct LLFixtureWithInodeCache;
impl FixtureFactory for LLFixtureWithInodeCache {
    type Driver = FuserFilesystemDriver<WithInodeCache>;

    fn fixture_type(&self) -> FixtureType {
        FixtureType::FuserWithInodeCache
    }
}

pub struct LLFixtureWithoutInodeCache;
impl FixtureFactory for LLFixtureWithoutInodeCache {
    type Driver = FuserFilesystemDriver<WithoutInodeCache>;

    fn fixture_type(&self) -> FixtureType {
        FixtureType::FuserWithoutInodeCache
    }
}

pub struct MountingFuserFixture;
impl FixtureFactory for MountingFuserFixture {
    type Driver = FuserMountingFilesystemDriver;

    fn fixture_type(&self) -> FixtureType {
        // Note: This fixture type here doesn't actually matter since we don't use [MountingFuserFixture] for operation counting, only for benchmarks. And only operation counting needs the fixture type.
        FixtureType::FuserWithInodeCache
    }
}

pub struct MountingFusemtFixture;
impl FixtureFactory for MountingFusemtFixture {
    type Driver = FusemtMountingFilesystemDriver;

    fn fixture_type(&self) -> FixtureType {
        // Note: This fixture type here doesn't actually matter since we don't use [MountingFusemtFixture] for operation counting, only for benchmarks. And only operation counting needs the fixture type.
        FixtureType::Fusemt
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

#[template]
pub fn all_fusemt_fixtures(
    #[values(crate::rstest::HLFixture)] fixture_factory: impl FixtureFactory,
) {
}
