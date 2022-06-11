use anyhow::Result;
use generic_array::ArrayLength;

pub trait Cipher : Sized {
    type KeySize : ArrayLength<u8>;

    fn new(key: EncryptionKey<Self::KeySize>) -> Self;

    fn ciphertext_size(plaintext_size: usize) -> usize;
    fn plaintext_size(ciphertext_size: usize) -> usize;

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// TODO https://github.com/shadowsocks/crypto2 looks pretty fast, maybe we can use them for faster implementations?

mod aead_crate_wrapper;
mod aesgcm;
mod key;
mod cipher_crate_wrapper;

#[cfg(test)]
mod cipher_tests;

pub use key::EncryptionKey;

// export ciphers
pub use aesgcm::{Aes128Gcm, Aes256Gcm};
pub type XChaCha20Poly1305 = aead_crate_wrapper::AeadCipher<chacha20poly1305::XChaCha20Poly1305>;
