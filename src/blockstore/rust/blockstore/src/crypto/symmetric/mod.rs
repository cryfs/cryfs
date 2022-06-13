use crate::data::Data;
use anyhow::Result;
use generic_array::ArrayLength;

pub trait Cipher: Sized {
    type KeySize: ArrayLength<u8>;

    fn new(key: EncryptionKey<Self::KeySize>) -> Self;

    // How many bytes is a ciphertext larger than a plaintext?
    const CIPHERTEXT_OVERHEAD_PREFIX: usize;
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize;

    // TODO Can we make this API safer? It requires the data block passed in to have at least CIPHERTEXT_OVERHEAD prefix bytes available.
    fn encrypt(&self, data: Data) -> Result<Data>;

    fn decrypt(&self, data: Data) -> Result<Data>;
}

// TODO https://github.com/shadowsocks/crypto2 looks pretty fast, maybe we can use them for faster implementations?
// TODO Ring also looks pretty fast, see https://kerkour.com/rust-symmetric-encryption-aead-benchmark

mod aead_crate_wrapper;
mod aesgcm;
mod cipher_crate_wrapper;
mod key;

#[cfg(test)]
mod cipher_tests;

pub use key::EncryptionKey;

// export ciphers
pub use aesgcm::{
    Aes128Gcm, Aes256Gcm, Aes256Gcm_HardwareAccelerated, Aes256Gcm_SoftwareImplemented,
};
pub type XChaCha20Poly1305 = aead_crate_wrapper::AeadCipher<chacha20poly1305::XChaCha20Poly1305>;

// offer a way to lookup ciphers at runtime while statically binding its type
pub trait CipherCallback {
    type Result;

    fn callback<C: Cipher + Send + Sync + 'static>(self) -> Self::Result;
}
pub fn lookup_cipher<CB>(cipher_name: &str, callback: CB) -> CB::Result
where
    CB: CipherCallback,
{
    match cipher_name {
        "xchacha20-poly1305" => callback.callback::<XChaCha20Poly1305>(),
        "aes-256-gcm" => callback.callback::<Aes256Gcm>(),
        "aes-128-gcm" => callback.callback::<Aes128Gcm>(),
        // TODO Add more ciphers
        _ => panic!("Unknown cipher: {}", cipher_name),
    }
}
