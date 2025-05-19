use criterion::{Criterion, criterion_group, criterion_main};

pub fn dummy_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| {}));
}

criterion_group!(dummy_benches, dummy_benchmark);

#[cfg(not(feature = "benchmark"))]
fn main() {
    panic!(
        "This binary is only for benchmarking purposes. Please run with the 'benchmark' feature enabled."
    );
}
#[cfg(feature = "benchmark")]
criterion_main!(
    dummy_benches,
    cryfs_e2e_perf_tests::operations::chmod::benches_chmod
);
