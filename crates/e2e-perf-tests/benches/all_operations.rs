use criterion::{Criterion, criterion_group, criterion_main};

#[cfg(not(feature = "benchmark"))]
fn main() {
    panic!(
        "This binary is only for benchmarking purposes. Please run with the 'benchmark' feature enabled."
    );
}
#[cfg(feature = "benchmark")]
criterion_main!(
    cryfs_e2e_perf_tests::operations::chmod::benches,
    cryfs_e2e_perf_tests::operations::chown::benches,
    cryfs_e2e_perf_tests::operations::create_file::benches,
);
