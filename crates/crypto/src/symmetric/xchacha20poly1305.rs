//! XChaCha20-Poly1305 authenticated encryption cipher.
//!
//! XChaCha20-Poly1305 is a modern AEAD cipher that combines the ChaCha20 stream cipher
//! with the Poly1305 message authentication code. The "X" variant uses a 192-bit (24 byte)
//! nonce, which is large enough to be safely generated randomly without risk of collision.
//!
//! # When to Use
//!
//! XChaCha20-Poly1305 is recommended when:
//! - You need to encrypt a very large number of messages with the same key
//! - You want to safely use random nonces without collision risk
//! - Hardware AES acceleration is unavailable
//!
//! # Key Properties
//!
//! - Key size: 32 bytes (256 bits)
//! - Nonce size: 24 bytes (192 bits)
//! - Auth tag size: 16 bytes (128 bits)

/// XChaCha20-Poly1305 using the pure Rust `chacha20poly1305` crate.
///
/// This implementation is fully portable and doesn't require external libraries.
pub type AeadXChaCha20Poly1305 =
    super::backends::aead::AeadCipher<chacha20poly1305::XChaCha20Poly1305>;

/// XChaCha20-Poly1305 using libsodium.
///
/// This implementation uses libsodium for high-performance encryption.
pub type LibsodiumXChaCha20Poly1305 = super::backends::libsodium::XChaCha20Poly1305;

/// Default XChaCha20-Poly1305 implementation using libsodium.
///
/// This is the recommended XChaCha20-Poly1305 implementation as it provides
/// good performance across platforms.
pub type XChaCha20Poly1305 = LibsodiumXChaCha20Poly1305;
