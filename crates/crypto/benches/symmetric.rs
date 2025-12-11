use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{RngCore, SeedableRng, rngs::StdRng};
use std::hint::black_box;

// TODO Separate out InfallibleUnwrap from lockable and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

use cryfs_crypto::symmetric::{self, Cipher, CipherDef, EncryptionKey, LibsodiumAes256GcmNonce12};
use cryfs_utils::data::Data;

type Aes256Gcm = symmetric::Aes256Gcm;
type AeadAes256Gcm = symmetric::AeadAes256Gcm;
type OpensslAes256Gcm = symmetric::OpensslAes256Gcm;
type Aes128Gcm = symmetric::Aes128Gcm;
type AeadAes128Gcm = symmetric::AeadAes128Gcm;
type OpensslAes128Gcm = symmetric::OpensslAes128Gcm;
type XChaCha20Poly1305 = symmetric::XChaCha20Poly1305;
type AeadXChaCha20Poly1305 = symmetric::AeadXChaCha20Poly1305;
type LibsodiumXChaCha20Poly1305 = symmetric::LibsodiumXChaCha20Poly1305;

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

fn make_plaintext<C: CipherDef>(_c: &C, size: usize) -> Data {
    let mut plaintext = data(size, 0);
    plaintext.reserve(C::CIPHERTEXT_OVERHEAD_PREFIX, C::CIPHERTEXT_OVERHEAD_SUFFIX);
    plaintext
}

fn make_ciphertext(cipher: &impl CipherDef, size: usize) -> Data {
    let plaintext = make_plaintext(cipher, size);
    cipher.encrypt(plaintext).unwrap()
}

fn bench_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt");

    for size in [1, 1024, 16 * 1024, 1024 * 1024 /*, 100*1024*1024*/] {
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-default", size),
            &size,
            |b, &size| {
                let cipher = Aes256Gcm::new(make_key(Aes256Gcm::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        if LibsodiumAes256GcmNonce12::is_available() {
            group.bench_with_input(
                BenchmarkId::new("aes256gcm-libsodium", size),
                &size,
                |b, &size| {
                    let cipher = LibsodiumAes256GcmNonce12::new(make_key(
                        LibsodiumAes256GcmNonce12::KEY_SIZE,
                    ))
                    .unwrap();
                    let plaintext = make_plaintext(&cipher, size);
                    b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
                },
            );
        }
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-aead", size),
            &size,
            |b, &size| {
                let cipher = AeadAes256Gcm::new(make_key(AeadAes256Gcm::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-openssl", size),
            &size,
            |b, &size| {
                let cipher = OpensslAes256Gcm::new(make_key(OpensslAes256Gcm::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes128gcm-default", size),
            &size,
            |b, &size| {
                let cipher = Aes128Gcm::new(make_key(Aes128Gcm::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes128gcm-aead", size),
            &size,
            |b, &size| {
                let cipher = AeadAes128Gcm::new(make_key(AeadAes128Gcm::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes128gcm-openssl", size),
            &size,
            |b, &size| {
                let cipher = OpensslAes128Gcm::new(make_key(OpensslAes128Gcm::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("xchacha20poly1305-default", size),
            &size,
            |b, &size| {
                let cipher = XChaCha20Poly1305::new(make_key(XChaCha20Poly1305::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("xchacha20poly1305-aead", size),
            &size,
            |b, &size| {
                let cipher =
                    AeadXChaCha20Poly1305::new(make_key(AeadXChaCha20Poly1305::KEY_SIZE)).unwrap();
                let plaintext = make_plaintext(&cipher, size);
                b.iter(|| black_box(cipher.encrypt(plaintext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("xchacha20poly1305-libsodium", size),
            &size,
            |b, &size| {
                let cipher =
                    LibsodiumXChaCha20Poly1305::new(make_key(LibsodiumXChaCha20Poly1305::KEY_SIZE))
                        .unwrap();
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
            BenchmarkId::new("aes256gcm-default", size),
            &size,
            |b, &size| {
                let cipher = Aes256Gcm::new(make_key(Aes256Gcm::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        if LibsodiumAes256GcmNonce12::is_available() {
            group.bench_with_input(
                BenchmarkId::new("aes256gcm-libsodium", size),
                &size,
                |b, &size| {
                    let cipher = LibsodiumAes256GcmNonce12::new(make_key(
                        LibsodiumAes256GcmNonce12::KEY_SIZE,
                    ))
                    .unwrap();
                    let ciphertext = make_ciphertext(&cipher, size);
                    b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
                },
            );
        }
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-aead", size),
            &size,
            |b, &size| {
                let cipher = AeadAes256Gcm::new(make_key(AeadAes256Gcm::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes256gcm-openssl", size),
            &size,
            |b, &size| {
                let cipher = OpensslAes256Gcm::new(make_key(OpensslAes256Gcm::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes128gcm-default", size),
            &size,
            |b, &size| {
                let cipher = Aes128Gcm::new(make_key(Aes128Gcm::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes128gcm-aead", size),
            &size,
            |b, &size| {
                let cipher = AeadAes128Gcm::new(make_key(AeadAes128Gcm::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("aes128gcm-openssl", size),
            &size,
            |b, &size| {
                let cipher = OpensslAes128Gcm::new(make_key(OpensslAes128Gcm::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("xchacha20poly1305-default", size),
            &size,
            |b, &size| {
                let cipher = XChaCha20Poly1305::new(make_key(XChaCha20Poly1305::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("xchacha20poly1305-aead", size),
            &size,
            |b, &size| {
                let cipher =
                    AeadXChaCha20Poly1305::new(make_key(AeadXChaCha20Poly1305::KEY_SIZE)).unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("xchacha20poly1305-libsodium", size),
            &size,
            |b, &size| {
                let cipher =
                    LibsodiumXChaCha20Poly1305::new(make_key(LibsodiumXChaCha20Poly1305::KEY_SIZE))
                        .unwrap();
                let ciphertext = make_ciphertext(&cipher, size);
                b.iter(|| black_box(cipher.decrypt(ciphertext.clone()).unwrap()));
            },
        );
    }
}

criterion_group!(benches, bench_encrypt, bench_decrypt);
criterion_main!(benches);
