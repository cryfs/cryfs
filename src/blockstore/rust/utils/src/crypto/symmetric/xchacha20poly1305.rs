pub type AeadXChaCha20Poly1305 =
    super::backends::aead::AeadCipher<chacha20poly1305::XChaCha20Poly1305>;
pub type LibsodiumXChaCha20Poly1305 = super::backends::libsodium::XChaCha20Poly1305;

/// Default implementation for XChaCha20Poly1305
pub type XChaCha20Poly1305 = LibsodiumXChaCha20Poly1305;
