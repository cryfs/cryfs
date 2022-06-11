//! AES-256-GCM implementation based on libsodium. This implementation is hardware accelerated but only works
//! on CPUs that are new enough to have that support. If the CPU doesn't support it, then `Aes256Gcm::new()`
//! will return an error.

use anyhow::{anyhow, Context, Result};
use generic_array::typenum::U32;
use sodiumoxide::crypto::aead::aes256gcm::{Aes256Gcm as _Aes256Gcm, Key, Nonce, Tag};
use std::sync::Once;

use super::super::{Cipher, EncryptionKey};
use super::{AUTH_TAG_SIZE, NONCE_SIZE};

use crate::data::Data;

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

    const CIPHERTEXT_OVERHEAD: usize = super::Aes256Gcm::CIPHERTEXT_OVERHEAD;

    fn new(encryption_key: EncryptionKey<Self::KeySize>) -> Self {
        init_libsodium();

        let cipher = _Aes256Gcm::new().expect("Hardware doesn't support the instructions needed for this implementation. Please check is_available() before calling new().");
        Self {
            cipher,
            encryption_key,
        }
    }

    fn encrypt(&self, mut plaintext: Data) -> Result<Data> {
        // TODO Is this data layout compatible with the C++ version of EncryptedBlockStore2?
        // TODO Use binary-layout here?
        let ciphertext_size = plaintext.len() + Self::CIPHERTEXT_OVERHEAD;
        let nonce = self.cipher.gen_initial_nonce();
        let auth_tag = self
            .cipher
            // TODO Move convert_key call to constructor so we don't have to do it every time?
            //      Note that we have to somehow migrate the
            //      secret protection we get from our EncryptionKey class then.
            .seal_detached(
                plaintext.as_mut(),
                None,
                &nonce,
                &convert_key(&self.encryption_key),
            );
        let mut ciphertext = plaintext.grow_region(Self::CIPHERTEXT_OVERHEAD, 0).context(
            "Tried to add prefix bytes so we can store ciphertext overhead in libsodium::Aes256Gcm::encrypt").unwrap();
        ciphertext[0..NONCE_SIZE].copy_from_slice(nonce.as_ref());
        ciphertext[NONCE_SIZE..(NONCE_SIZE + AUTH_TAG_SIZE)].copy_from_slice(auth_tag.as_ref());
        assert_eq!(ciphertext_size, ciphertext.len());
        Ok(ciphertext)
    }

    fn decrypt(&self, mut ciphertext: Data) -> Result<Data> {
        let ciphertext_len = ciphertext.len();
        let (nonce, rest) = ciphertext.as_mut().split_at_mut(NONCE_SIZE);
        let (auth_tag, cipherdata) = rest.split_at_mut(AUTH_TAG_SIZE);
        let nonce = Nonce::from_slice(nonce).expect("Wrong nonce size");
        let auth_tag = Tag::from_slice(auth_tag).expect("Wrong auth tag size");
        self.cipher
            // TODO Move convert_key call to constructor so we don't have to do it every time?
            //      Note that we have to somehow migrate the
            //      secret protection we get from our EncryptionKey class then.
            .open_detached(
                cipherdata.as_mut(),
                None,
                &auth_tag,
                &nonce,
                &convert_key(&self.encryption_key),
            )
            .map_err(|()| anyhow!("Decrypting data failed"))?;
        let plaintext = ciphertext.into_subregion((NONCE_SIZE + AUTH_TAG_SIZE)..);
        assert_eq!(
            ciphertext_len
                .checked_sub(Self::CIPHERTEXT_OVERHEAD)
                .unwrap(),
            plaintext.len()
        );
        Ok(plaintext)
    }
}

fn convert_key(key: &EncryptionKey<U32>) -> Key {
    // Panic on error is ok because key size is hard coded and not dependent on input here
    Key::from_slice(key.as_bytes()).expect("Invalid key size")
}

// Test cases are in cipher_tests.rs
