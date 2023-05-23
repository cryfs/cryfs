use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use generic_array::ArrayLength;
use rand::{rngs::StdRng, RngCore, SeedableRng};

// TODO Separate out InfallibleUnwrap from lockable and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

use cryfs_utils::{crypto::symmetric::*, data::Data};

fn data(size: usize, seed: u64) -> Data {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; size];
    rng.fill_bytes(&mut res);
    res.into()
}

fn make_key(size: usize) -> EncryptionKey {
    EncryptionKey::new(size, |key_data| {
        let mut rng = StdRng::seed_from_u64(0);
        rng.fill_bytes(key_data);
        Ok(())
    })
    .infallible_unwrap()
}

fn make_plaintext<C: Cipher>(_c: &C, size: usize) -> Data {
    let mut plaintext = data(size, 0);
    plaintext.reserve(C::CIPHERTEXT_OVERHEAD_PREFIX, C::CIPHERTEXT_OVERHEAD_SUFFIX);
    plaintext
}

fn make_ciphertext(cipher: &impl Cipher, size: usize) -> Data {
    let plaintext = make_plaintext(cipher, size);
    cipher.encrypt(plaintext).unwrap()
}

fn bench_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt");

    for size in [1, 1024, 16 * 1024, 1024 * 1024 /*, 100*1024*1024*/] {
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-auto", size),
            &size,
            |b, &size| {
                let cipher = Aes256Gcm::new(make_key(Aes256Gcm::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        if Aes256GcmHardwareAccelerated::is_available() {
            group.bench_with_input(
                BenchmarkId::new("aes256gcm-hardware", size),
                &size,
                |b, &size| {
                    let cipher = Aes256GcmHardwareAccelerated::new(make_key(
                        Aes256GcmHardwareAccelerated::KEY_SIZE,
                    ))
                    .unwrap();
                    let plaintext = make_plaintext(&cipher, size);
                    b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
                },
            );
        }
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-software", size),
            &size,
            |b, &size| {
                let cipher = Aes256GcmSoftwareImplemented::new(make_key(
                    Aes256GcmSoftwareImplemented::KEY_SIZE,
                ))
                .unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(BenchmarkId::new("aes128gcm", size), &size, |b, &size| {
            let cipher = Aes128Gcm::new(make_key(Aes128Gcm::KEY_SIZE)).unwrap();
            let plaintext = make_plaintext(&cipher, size);
            b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
        });
        group.bench_with_input(
            BenchmarkId::new("xchacha20-poly1305", size),
            &size,
            |b, &size| {
                let cipher = XChaCha20Poly1305::new(make_key(XChaCha20Poly1305::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
    }
}

fn bench_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("decrypt");

    for size in [1, 1024, 16 * 1024, 1024 * 1024 /*, 100*1024*1024*/] {
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-auto", size),
            &size,
            |b, &size| {
                let cipher = Aes256Gcm::new(make_key(Aes256Gcm::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        if Aes256GcmHardwareAccelerated::is_available() {
            group.bench_with_input(
                BenchmarkId::new("aes256gcm-hardware", size),
                &size,
                |b, &size| {
                    let cipher = Aes256GcmHardwareAccelerated::new(make_key(
                        Aes256GcmHardwareAccelerated::KEY_SIZE,
                    ))
                    .unwrap();
                    let ciphertext = make_ciphertext(&cipher, size);
                    b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
                },
            );
        }
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-software", size),
            &size,
            |b, &size| {
                let cipher = Aes256GcmSoftwareImplemented::new(make_key(
                    Aes256GcmSoftwareImplemented::KEY_SIZE,
                ))
                .unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(BenchmarkId::new("aes128gcm", size), &size, |b, &size| {
            let cipher = Aes128Gcm::new(make_key(Aes128Gcm::KEY_SIZE)).unwrap();
            let ciphertext = make_ciphertext(&cipher, size);
            b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
        });
        group.bench_with_input(
            BenchmarkId::new("xchacha20-poly1305", size),
            &size,
            |b, &size| {
                let cipher = XChaCha20Poly1305::new(make_key(XChaCha20Poly1305::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
    }
}

criterion_group!(benches, bench_encrypt, bench_decrypt);
criterion_main!(benches);
