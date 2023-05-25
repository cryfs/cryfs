use generic_array::typenum::U16;

// TODO AES-GCM-SIV might be better than AES-GCM
// TODO Add 128bit fixed string to the message and verify it, see https://libsodium.gitbook.io/doc/secret-key_cryptography/aead#robustness
// TODO Use different keys for different blocks to reduce chance of nonce collisions

/// We're using an unusual nonce size of 16 bytes because:
/// * Using GCM with 12 bytes means the algorithm's IV becomes the 12 bytes nonce plus a 4 bytes counter starting at 1.
/// * Using GCM with 16 bytes means the algorithm's IV becomes a 16 bytes hash of that nonce.
/// * So the most secure approach would be to use a deterministic guaranteed-unique (e.g. incremental) 12 byte nonce without any hashing.
///   In that approach, a 16 byte nonce would be bad because the hashing could reintroduce collisions.
/// * But we don't have a global counter, so we need to use random nonces.
/// * Now with random nonces, using a 16 bytes nonce, even if hashed, is better than using a 12 bytes nonce because it reduces the chance of collisions.
pub type DefaultNonceSize = U16;

pub type LibsodiumAes256GcmNonce12 = super::backends::libsodium::Aes256Gcm;
pub type AeadAes256Gcm<NonceSize = DefaultNonceSize> =
    super::backends::aead::AeadCipher<aes_gcm::AesGcm<aes_gcm::aes::Aes256, NonceSize>>;
pub type OpensslAes256Gcm<NonceSize = DefaultNonceSize> =
    super::backends::openssl::AeadCipher<super::backends::openssl::Aes256Gcm<NonceSize>>;

/// Default aes-256-gcm implementation
pub type Aes256Gcm<NonceSize = DefaultNonceSize> = OpensslAes256Gcm<NonceSize>;

pub type OpensslAes128Gcm<NonceSize = DefaultNonceSize> =
    super::backends::openssl::AeadCipher<super::backends::openssl::Aes128Gcm<NonceSize>>;
pub type AeadAes128Gcm<NonceSize = DefaultNonceSize> =
    super::backends::aead::AeadCipher<aes_gcm::AesGcm<aes_gcm::aes::Aes128, NonceSize>>;

/// Default aes-128-gcm implementation
pub type Aes128Gcm<NonceSize = DefaultNonceSize> = OpensslAes128Gcm<NonceSize>;
