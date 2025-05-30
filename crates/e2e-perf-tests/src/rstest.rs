use cryfs_rustfs::AtimeUpdateBehavior;
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
                let mut bench = criterion.benchmark_group({{name_str}});
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
                    let test_driver = TestDriver::new(crate::rstest::MountingFuserFixture, atime_value);
                    let test = {{name}}(test_driver);
                    bench.bench_function(&format!("fuser:{atime_name}"), move |b| {
                        test.run_benchmark(b);
                    });

                    // fusemt
                    let test_driver = TestDriver::new(crate::rstest::MountingFusemtFixture, atime_value);
                    let test = {{name}}(test_driver);
                    bench.bench_function(&format!("fusemt:{atime_name}"), move |b| {
                        test.run_benchmark(b);
                    });
                }
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

// TODO Can we remove FixtureFactory? Not sure what purpose it adds on top of FilesystemFixture / FilesystemDriver

pub trait FixtureFactory: 'static {
    type Driver: FilesystemDriver;

    fn fixture_type(&self) -> FixtureType;

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
