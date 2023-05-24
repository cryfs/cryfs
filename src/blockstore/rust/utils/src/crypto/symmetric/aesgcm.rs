use generic_array::typenum::{Unsigned, U12};

// TODO AES-GCM-SIV or XChaCha20-Poly1305 (XChaCha20-Poly1305-ietf, chacha20poly1305_ietf, chacha20poly1305) might be better than AES-GCM
// TODO Add 128bit fixed string to the message and verify it, see https://libsodium.gitbook.io/doc/secret-key_cryptography/aead#robustness

use super::CipherDef;

#[allow(non_camel_case_types)]
type NONCE_SIZE = U12;

pub type LibsodiumAes256Gcm = super::backends::libsodium::Aes256Gcm;
static_assertions::const_assert_eq!(
    <LibsodiumAes256Gcm as CipherDef>::CIPHERTEXT_OVERHEAD_PREFIX,
    NONCE_SIZE::USIZE
);

pub type AeadAes256Gcm =
    super::backends::aead::AeadCipher<aes_gcm::AesGcm<aes_gcm::aes::Aes256, NONCE_SIZE>>;
pub type OpensslAes256Gcm =
    super::backends::openssl::AeadCipher<super::backends::openssl::Aes256Gcm<NONCE_SIZE>>;

/// Default aes-256-gcm implementation
pub type Aes256Gcm = OpensslAes256Gcm;

pub type OpensslAes128Gcm =
    super::backends::openssl::AeadCipher<super::backends::openssl::Aes128Gcm<NONCE_SIZE>>;
pub type AeadAes128Gcm =
    super::backends::aead::AeadCipher<aes_gcm::AesGcm<aes_gcm::aes::Aes128, NONCE_SIZE>>;
// TODO Libsodium probably has aes128gcm as well, add it.

/// Default aes-128-gcm implementation
pub type Aes128Gcm = OpensslAes128Gcm;
