// TODO If rust stabilizes custom test frameworks, we can make `perf_test` a per-test macro instead of taking multiple macro names listing all tests in a file. See https://bheisler.github.io/criterion.rs/book/user_guide/custom_test_framework.html

/// Macro that instantiates all performance counter test cases (similar to rstest duplication)
#[cfg(not(feature = "benchmark"))]
#[crabtime::function]
fn perf_test_(_group: String, names: Vec<String>, disable_fusemt: u8, disable_fuser: u8) {
    // TODO Deduplicate normalize_identifier
    fn normalize_identifier(name: String) -> String {
        name.replace(|c: char| !c.is_alphanumeric(), "_")
    }
    let mut fixtures = vec![];
    if disable_fuser == 0 {
        fixtures.extend([
            (
                "ll_cache",
                "crate::filesystem_driver::FuserFilesystemDriver::<crate::filesystem_driver::WithInodeCache>",
                "crate::perf_test_macro::FixtureType::FuserWithInodeCache",
            ),
            (
                "ll_nocache",
                "crate::filesystem_driver::FuserFilesystemDriver::<crate::filesystem_driver::WithoutInodeCache>",
                "crate::perf_test_macro::FixtureType::FuserWithoutInodeCache",
            ),
        ]);
    }
    if disable_fusemt == 0 {
        fixtures.push((
            "hl",
            "crate::filesystem_driver::FusemtFilesystemDriver",
            "crate::perf_test_macro::FixtureType::Fusemt",
        ));
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
        for (fixture_name, filesystem_driver, fixture_type) in &fixtures {
            for (atime_name, atime_value) in atime_behaviors {
                crabtime::output_str!(
                    r#"
                    #[test]
                    fn {fixture_name}_{atime_name}() {{
                        crate::env_logger::init();
                        let filesystem_driver = std::marker::PhantomData::<{filesystem_driver}>;
                        let fixture_type = {fixture_type};
                        let atime_behavior = {atime_value};
                        let test_driver = crate::test_driver::TestDriverImpl::new(cryfs_blockstore::InMemoryBlockStore::new, filesystem_driver, fixture_type, atime_behavior);
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

/// Macro that instantiates all benchmarks (similar to rstest duplication)
#[cfg(feature = "benchmark")]
#[crabtime::function]
fn perf_test_(group: String, names: Vec<String>, disable_fusemt: u8, disable_fuser: u8) {
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
                    if {{disable_fuser}}== 0 {
                        let filesystem_driver = std::marker::PhantomData::<crate::filesystem_driver::FuserMountingFilesystemDriver>;
                        let test_driver = crate::test_driver::TestDriverImpl::new(cryfs_blockstore::TempDirBlockStore::new, filesystem_driver, crate::perf_test_macro::FixtureType::FuserWithInodeCache, atime_value);
                        let test = {{name}}(test_driver);
                        bench.bench_function(&format!("fuser:{atime_name}"), move |b| {
                            test.run_benchmark(b);
                        });
                    }

                    // fusemt
                    if {{disable_fusemt}} == 0 {
                        let filesystem_driver = std::marker::PhantomData::<crate::filesystem_driver::FusemtMountingFilesystemDriver>;
                        let test_driver = crate::test_driver::TestDriverImpl::new(cryfs_blockstore::TempDirBlockStore::new, filesystem_driver, crate::perf_test_macro::FixtureType::Fusemt, atime_value);
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

/// The [perf_test!] should be invoked for each test, and it will instantiate
/// the test as a counter test (if run in `cargo test`) or as a benchmark
/// (if run in `cargo bench --features benchmark`).
///
/// It will also duplicate the test for different cases, e.g. fuser vs fuse-mt,
/// different atime behaviors, similar to how rstest duplication with #[values()]
/// or #[case] would do it.
macro_rules! perf_test {
    ($group:ident, $tests:tt) => {
        $crate::perf_test_macro::perf_test_!($group, $tests, 0, 0);
    };
}

/// Like [perf_test!], but only runs the fuser tests, not fuse-mt.
macro_rules! perf_test_only_fuser {
    ($group:ident, $tests:tt) => {
        $crate::perf_test_macro::perf_test_!($group, $tests, 1, 0);
    };
}

/// Like [perf_test!], but only runs the fuse-mt tests, not fuser.
macro_rules! perf_test_only_fusemt {
    ($group:ident, $tests:tt) => {
        $crate::perf_test_macro::perf_test_!($group, $tests, 0, 1);
    };
}

pub(crate) use perf_test;
pub(crate) use perf_test_;
pub(crate) use perf_test_only_fusemt;
pub(crate) use perf_test_only_fuser;

#[derive(Debug, Clone, Copy)]
pub enum FixtureType {
    FuserWithInodeCache,
    FuserWithoutInodeCache,
    Fusemt,
}
