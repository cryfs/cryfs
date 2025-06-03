use criterion::{Criterion, criterion_group, criterion_main};

#[cfg(not(feature = "benchmark"))]
fn main() {
    panic!(
        "This binary is only for benchmarking purposes. Please run with the 'benchmark' feature enabled."
    );
}
#[cfg(feature = "benchmark")]
criterion_main!(
    cryfs_e2e_perf_tests::operations::chmod::benches_chmod,
    cryfs_e2e_perf_tests::operations::chown::benches_chown,
    cryfs_e2e_perf_tests::operations::create_file::benches_create_file,
    cryfs_e2e_perf_tests::operations::fchmod::benches_fchmod,
    cryfs_e2e_perf_tests::operations::fchown::benches_fchown,
    cryfs_e2e_perf_tests::operations::fgetattr::benches_fgetattr,
    cryfs_e2e_perf_tests::operations::flush::benches_flush,
    cryfs_e2e_perf_tests::operations::fsync::benches_fsync_datasync,
    cryfs_e2e_perf_tests::operations::fsync::benches_fsync_fullsync,
    cryfs_e2e_perf_tests::operations::ftruncate::benches_ftruncate,
    cryfs_e2e_perf_tests::operations::futimens::benches_futimens,
    cryfs_e2e_perf_tests::operations::getattr::benches_getattr,
    cryfs_e2e_perf_tests::operations::init::benches_init,
);
