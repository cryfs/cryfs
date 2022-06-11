use anyhow::Result;
use generic_array::typenum::U32;
use log::warn;

// TODO AES-GCM-SIV or XChaCha20-Poly1305 (XChaCha20-Poly1305-ietf, chacha20poly1305_ietf, chacha20poly1305) might be better than AES-GCM
// TODO Add 128bit fixed string to the message and verify it, see https://libsodium.gitbook.io/doc/secret-key_cryptography/aead#robustness

mod libsodium;

use super::{Cipher, EncryptionKey};
use crate::data::Data;

const NONCE_SIZE: usize = 12;
const AUTH_TAG_SIZE: usize = 16;

pub type Aes256Gcm_HardwareAccelerated = libsodium::Aes256Gcm;
pub type Aes256Gcm_SoftwareImplemented = super::aead_crate_wrapper::AeadCipher<aes_gcm::Aes256Gcm>;

/// An implementation of the AES-256-GCM cipher. This does runtime CPU feature detection.
/// If the CPU supports a hardware accelerated implementation, that one will be used, oherwise we fall back
/// to a slow software implementation.
enum Aes256GcmImpl {
    HardwareAccelerated(Aes256Gcm_HardwareAccelerated),
    SoftwareImplementation(Aes256Gcm_SoftwareImplemented),
}

pub struct Aes256Gcm(Aes256GcmImpl);

impl Cipher for Aes256Gcm {
    type KeySize = U32;

    const CIPHERTEXT_OVERHEAD_PREFIX: usize = NONCE_SIZE;
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize = AUTH_TAG_SIZE;

    fn new(encryption_key: EncryptionKey<Self::KeySize>) -> Self {
        let hardware_acceleration_available = Aes256Gcm_HardwareAccelerated::is_available();
        if hardware_acceleration_available {
            Self(Aes256GcmImpl::HardwareAccelerated(
                Aes256Gcm_HardwareAccelerated::new(encryption_key),
            ))
        } else {
            warn!("Your CPU doesn't offer hardware acceleration for AES. Doing cryptography will be very slow.");
            Self(Aes256GcmImpl::SoftwareImplementation(
                Aes256Gcm_SoftwareImplemented::new(encryption_key),
            ))
        }
    }

    fn encrypt(&self, plaintext: Data) -> Result<Data> {
        match &self.0 {
            Aes256GcmImpl::HardwareAccelerated(i) => i.encrypt(plaintext),
            Aes256GcmImpl::SoftwareImplementation(i) => i.encrypt(plaintext),
        }
    }

    fn decrypt(&self, ciphertext: Data) -> Result<Data> {
        match &self.0 {
            Aes256GcmImpl::HardwareAccelerated(i) => i.decrypt(ciphertext),
            Aes256GcmImpl::SoftwareImplementation(i) => i.decrypt(ciphertext),
        }
    }
}

// We don't have a hardware accelerated implementation for Aes-128-gcm, so let's just use the aes_gcm software one
pub type Aes128Gcm = super::aead_crate_wrapper::AeadCipher<aes_gcm::Aes128Gcm>;
