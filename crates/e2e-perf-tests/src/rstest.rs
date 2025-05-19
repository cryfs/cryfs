use cryfs_rustfs::AtimeUpdateBehavior;
use rstest_reuse::{self, *};

use crate::{
    filesystem_driver::{
        FilesystemDriver, FusemtFilesystemDriver, FuserFilesystemDriver, WithInodeCache,
        WithoutInodeCache,
    },
    fixture::FilesystemFixture,
};

// TODO If rust stabilizes custom test frameworks, we can make `perf_test` a per-test macro instead of taking multiple macro names listing all tests in a file. See https://bheisler.github.io/criterion.rs/book/user_guide/custom_test_framework.html

#[cfg(not(feature = "benchmark"))]
#[crabtime::function]
fn perf_test(names: Vec<String>) {
    let fixtures = [
        ("hl", "crate::rstest::HLFixture"),
        ("ll_cache", "crate::rstest::LLFixtureWithInodeCache"),
        ("ll_nocache", "crate::rstest::LLFixtureWithoutInodeCache"),
    ];
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
        crabtime::output_str!(
            r#"
            mod test_{name} {{
                use super::*;
        "#
        );
        for (fixture_name, fixture_value) in fixtures {
            for (atime_name, atime_value) in atime_behaviors {
                crabtime::output_str!(
                    r#"
                    #[test]
                    fn {fixture_name}_{atime_name}() {{
                        let fixture_factory = {fixture_value};
                        let atime_behavior = {atime_value};
                        let test_driver = TestDriver::new(fixture_factory, atime_behavior);
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

#[cfg(feature = "benchmark")]
#[crabtime::function]
fn perf_test(names: Vec<String>) {
    for name in &names {
        let name_str = format!("\"{}\"", name);
        crabtime::output! {
            fn bench_{{name}}(criterion: &mut criterion::Criterion) {
                // TODO Benchmark should use Fuse{r,mt}MountingFilesystemDriver, and probably should also use a real on-disk blockstore
                let fixture_factory = crate::rstest::LLFixtureWithInodeCache; // TODO create different benches for different fixture types
                let atime_behavior = cryfs_rustfs::AtimeUpdateBehavior::Noatime; // TODO create different benches for different atime behaviors
                let test_driver = TestDriver::new(fixture_factory, atime_behavior);
                let test = {{name}}(test_driver);
                criterion.bench_function({ { name_str } }, move |b| {
                    test.run_benchmark(b);
                });
            }
        }
    }
    crabtime::output_str!("criterion::criterion_group!(benches_chmod");
    for name in &names {
        crabtime::output_str!(", bench_{name}");
    }
    crabtime::output_str!(");");
}

pub(crate) use perf_test;

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

// TODO FixtureFactory seems used for benchmarks as well, not just rstest. We should probably put it somewhere else?
//      Or maybe the whole rstest setup doesn't make sense anymore since we kinda want the same duplication for benchmarks as well? Maybe custom write the duplication logic?

pub trait FixtureFactory: 'static {
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

#[template]
pub fn all_fusemt_fixtures(
    #[values(crate::rstest::HLFixture)] fixture_factory: impl FixtureFactory,
) {
}
