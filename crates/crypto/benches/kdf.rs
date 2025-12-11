use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use cryfs_crypto::kdf::{
    PasswordBasedKDF,
    scrypt::{
        ScryptSettings,
        backends::{openssl::ScryptOpenssl, scrypt::ScryptScrypt},
    },
};

fn bench_scrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrypt");

    for log_n in [10, 15, 18] {
        for r in [1, 2, 4, 8] {
            for p in [1, 2, 4, 8] {
                if log_n == 18 && r == 1 {
                    // scrypt has a requirement: n < 2^(128 * r / 8)
                    continue;
                }
                let settings = ScryptSettings {
                    log_n,
                    r,
                    p,
                    salt_len: 32,
                };
                group.bench_with_input(
                    BenchmarkId::new("scrypt-scrypt", format!("{settings:?}")),
                    &settings,
                    |b, settings| {
                        let params = ScryptScrypt::generate_parameters(&settings).unwrap();
                        b.iter(|| black_box(ScryptScrypt::derive_key(64, "password", &params)));
                    },
                );
                group.bench_with_input(
                    BenchmarkId::new("scrypt-openssl", format!("{settings:?}")),
                    &settings,
                    |b, settings| {
                        let params = ScryptScrypt::generate_parameters(&settings).unwrap();
                        b.iter(|| black_box(ScryptOpenssl::derive_key(64, "password", &params)));
                    },
                );
            }
        }
    }
}

criterion_group!(benches, bench_scrypt);
criterion_main!(benches);
