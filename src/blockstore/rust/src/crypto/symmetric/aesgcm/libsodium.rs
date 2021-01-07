//! AES-256-GCM implementation based on libsodium. This implementation is hardware accelerated but only works
//! on CPUs that are new enough to have that support. If the CPU doesn't support it, then `Aes256Gcm::new()`
//! will return an error.

use anyhow::{anyhow, Result};
use generic_array::typenum::U32;
use sodiumoxide::crypto::aead::aes256gcm::{Aes256Gcm as _Aes256Gcm, Key, Nonce};
use std::sync::Once;

use super::super::{Cipher, EncryptionKey};

use super::NONCE_SIZE;

static INIT_LIBSODIUM: Once = Once::new();

fn init_libsodium() {
    INIT_LIBSODIUM.call_once(|| {
        sodiumoxide::init().expect("Failed to initialize libsodium");
    });
}

pub struct Aes256Gcm {
    cipher: _Aes256Gcm,
    encryption_key: EncryptionKey<U32>,
}

impl Aes256Gcm {
    /// Returns true, iff the hardware supports the instructions needed by this
    /// hardware-accelerated implementation of AES
    pub fn is_available() -> bool {
        init_libsodium();
        sodiumoxide::crypto::aead::aes256gcm::is_available()
    }
}

impl Cipher for Aes256Gcm {
    type KeySize = U32;

    fn new(encryption_key: EncryptionKey<Self::KeySize>) -> Self {
        init_libsodium();

        let cipher = _Aes256Gcm::new().expect("Hardware doesn't support the instructions needed for this implementation. Please check is_available() before calling new().");
        Self {
            cipher,
            encryption_key,
        }
    }

    fn ciphertext_size(plaintext_size: usize) -> usize {
        super::Aes256Gcm::ciphertext_size(plaintext_size)
    }

    fn plaintext_size(ciphertext_size: usize) -> usize {
        super::Aes256Gcm::plaintext_size(ciphertext_size)
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let ciphertext_size = Self::ciphertext_size(plaintext.len());
        let nonce = self.cipher.gen_initial_nonce();
        let cipherdata =
            self.cipher
                .seal(plaintext, None, &nonce, &convert_key(&self.encryption_key));
        let mut ciphertext = Vec::with_capacity(ciphertext_size);
        ciphertext.extend_from_slice(nonce.as_ref());
        ciphertext.extend(cipherdata); // TODO Is there a way to encrypt it without copying here? Or does it even matter?
        assert_eq!(ciphertext_size, ciphertext.len());
        Ok(ciphertext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let nonce = &ciphertext[..NONCE_SIZE];
        let cipherdata = &ciphertext[NONCE_SIZE..];
        let nonce = Nonce::from_slice(nonce).expect("Wrong nonce size");
        let plaintext = self
            .cipher
            .open(cipherdata, None, &nonce, &convert_key(&self.encryption_key))
            .map_err(|()| anyhow!("Decrypting data failed"))?;
        assert_eq!(Self::plaintext_size(ciphertext.len()), plaintext.len());
        Ok(plaintext)
    }
}

fn convert_key(key: &EncryptionKey<U32>) -> Key {
    // Panic on error is ok because key size is hard coded and not dependent on input here
    Key::from_slice(key.as_bytes()).expect("Invalid key size")
}

// Test cases are in cipher_tests.rs
