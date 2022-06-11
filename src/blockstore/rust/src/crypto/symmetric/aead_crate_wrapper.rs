//! Ciphers from the `aead` crate (and other crates following its traits, for example `aes_gcm`)

use aead::generic_array::typenum::Unsigned;
use aead::{
    generic_array::{ArrayLength, GenericArray},
    Aead, NewAead, Nonce,
};
use anyhow::{Context, Result};
use rand::{thread_rng, RngCore};
use std::marker::PhantomData;

use super::{Cipher, EncryptionKey};

// TODO The aes-gcm crate currently needs
// > RUSTFLAGS="-Ctarget-cpu=sandybridge -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"
// to build with hardware acceleration and we build without that, that's why we use it as the SoftwareImplemented version only.
// The announced to do runtime feature detection in the future though, we should then benchmark it against libsodium and possibly
// remove libsodium.
// TODO The chacha20-poly1305 crate needs
// > RUSTFLAGS="-Ctarget-feature=+avx2"
// or it won't use AVX2.

pub struct AeadCipher<C: NewAead + Aead> {
    encryption_key: EncryptionKey<C::KeySize>,
    _phantom: PhantomData<C>,
}

impl<C: NewAead + Aead> Cipher for AeadCipher<C> {
    type KeySize = C::KeySize;

    fn new(encryption_key: EncryptionKey<Self::KeySize>) -> Self {
        Self {
            encryption_key,
            _phantom: PhantomData {},
        }
    }

    fn ciphertext_size(plaintext_size: usize) -> usize {
        plaintext_size + C::NonceSize::USIZE + C::TagSize::USIZE
    }

    fn plaintext_size(ciphertext_size: usize) -> usize {
        assert!(
            ciphertext_size >= C::NonceSize::USIZE + C::TagSize::USIZE,
            "Invalid ciphertext size"
        );
        ciphertext_size - C::NonceSize::USIZE - C::TagSize::USIZE
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = C::new(GenericArray::from_slice(self.encryption_key.as_bytes()));
        let ciphertext_size = Self::ciphertext_size(plaintext.len());
        let nonce = random_nonce();
        let cipherdata = cipher
            .encrypt(&nonce, plaintext)
            .context("Encrypting data failed")?;
        let mut ciphertext = Vec::with_capacity(ciphertext_size);
        ciphertext.extend_from_slice(&nonce);
        ciphertext.extend(cipherdata); // TODO Is there a way to encrypt it without copying here? Or does it even matter?
        assert_eq!(ciphertext_size, ciphertext.len());
        Ok(ciphertext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let cipher = C::new(GenericArray::from_slice(self.encryption_key.as_bytes()));
        let nonce = &ciphertext[..C::NonceSize::USIZE];
        let cipherdata = &ciphertext[C::NonceSize::USIZE..];
        let plaintext = cipher
            .decrypt(nonce.into(), cipherdata)
            .context("Decrypting data failed")?;
        assert_eq!(Self::plaintext_size(ciphertext.len()), plaintext.len());
        Ok(plaintext)
    }
}

fn random_nonce<Size: ArrayLength<u8>>() -> Nonce<Size> {
    let mut nonce = Nonce::<Size>::default();
    let mut rng = thread_rng();
    rng.fill_bytes(&mut nonce);
    nonce
}

// Test cases are in cipher_tests.rs
