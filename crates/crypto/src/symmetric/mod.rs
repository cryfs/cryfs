use anyhow::Result;
use derive_more::{Display, Error};
use static_assertions::const_assert;

use cryfs_utils::data::Data;

pub trait Cipher {
    // TODO Can we make this API safer? It requires the data block passed in to have at least CIPHERTEXT_OVERHEAD prefix bytes available.
    fn encrypt(&self, data: Data) -> Result<Data>;

    fn decrypt(&self, data: Data) -> Result<Data>;

    fn ciphertext_overhead_prefix(&self) -> usize;
    fn ciphertext_overhead_suffix(&self) -> usize;
}

#[derive(Error, Display, Debug)]
#[display("Expected key size of {expected} bytes, but got {got} bytes")]
pub struct InvalidKeySizeError {
    pub expected: usize,
    pub got: usize,
}

pub trait CipherDef: Cipher + Sized {
    fn new(key: EncryptionKey) -> Result<Self, InvalidKeySizeError>;

    const KEY_SIZE: usize;

    // How many bytes is a ciphertext larger than a plaintext?
    const CIPHERTEXT_OVERHEAD_PREFIX: usize;
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize;
}

// TODO https://github.com/shadowsocks/crypto2 looks pretty fast, maybe we can use them for faster implementations?
// TODO Ring also looks pretty fast, see https://kerkour.com/rust-symmetric-encryption-aead-benchmark

mod backends;
mod key;

#[cfg(test)]
mod cipher_tests;

pub use key::EncryptionKey;

// export ciphers
mod aesgcm;
pub use aesgcm::{
    AeadAes128Gcm, AeadAes256Gcm, Aes128Gcm, Aes256Gcm, LibsodiumAes256GcmNonce12,
    OpensslAes128Gcm, OpensslAes256Gcm,
};
// TODO Is there an openssl implementation for XChaCha20Poly1305?
mod xchacha20poly1305;
pub use xchacha20poly1305::{AeadXChaCha20Poly1305, LibsodiumXChaCha20Poly1305, XChaCha20Poly1305};

// TODO Does DefaultNonceSize need to be public? This is weird. Or at least rename it to AesDefaultNonceSize
pub use self::aesgcm::DefaultNonceSize;

// The [MAX_KEY_SIZE] constant is relied upon in our scrypt key generation because we always generate max size keys, even if we need
// only fewer bytes afterwards. If we change this constant, we need to make sure that scrypt still generates the same
// values even if it gets a different key size as input.
pub const MAX_KEY_SIZE: usize = 56;
const_assert!(AeadAes128Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(AeadAes256Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(Aes128Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(LibsodiumAes256GcmNonce12::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(OpensslAes128Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(OpensslAes256Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(AeadXChaCha20Poly1305::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(LibsodiumXChaCha20Poly1305::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(XChaCha20Poly1305::KEY_SIZE <= MAX_KEY_SIZE);
