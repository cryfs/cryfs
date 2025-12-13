use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{RngCore, SeedableRng, rngs::StdRng};
use std::hint::black_box;

use cryfs_crypto::hash::{HashAlgorithm as _, OpensslSha512, Salt, Sha2Sha512, Sha512};
use cryfs_utils::data::Data;

fn data(size: usize, seed: u64) -> Data {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; size];
    rng.fill_bytes(&mut res);
    res.into()
}

fn bench_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash");

    for size in [1, 32, 1024, 16 * 1024, 1024 * 1024 /*, 100*1024*1024*/] {
        group.bench_with_input(BenchmarkId::new("default", size), &size, |b, &size| {
            let salt = Salt::generate_random();
            let data = data(size, 42);
            b.iter(|| black_box(Sha512::hash(&data, salt)));
        });
        group.bench_with_input(BenchmarkId::new("sha2", size), &size, |b, &size| {
            let salt = Salt::generate_random();
            let data = data(size, 42);
            b.iter(|| black_box(Sha2Sha512::hash(&data, salt)));
        });
        group.bench_with_input(BenchmarkId::new("openssl", size), &size, |b, &size| {
            let salt = Salt::generate_random();
            let data = data(size, 42);
            b.iter(|| black_box(OpensslSha512::hash(&data, salt)));
        });
    }
}

criterion_group!(benches, bench_hash);
criterion_main!(benches);
