use crate::data::Data;
use anyhow::Result;
use async_trait::async_trait;
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
    Aes128Gcm, Aes256Gcm, Aes256GcmHardwareAccelerated, Aes256GcmSoftwareImplemented,
};
pub type XChaCha20Poly1305 = aead_crate_wrapper::AeadCipher<chacha20poly1305::XChaCha20Poly1305>;

// offer a way to lookup ciphers at runtime while statically binding its type
#[async_trait]
pub trait CipherCallback {
    type Result;

    async fn callback<C: Cipher + Send + Sync + 'static>(self) -> Self::Result;
}
pub async fn lookup_cipher<CB>(cipher_name: &str, callback: CB) -> CB::Result
where
    CB: CipherCallback,
{
    match cipher_name {
        "xchacha20-poly1305" => callback.callback::<XChaCha20Poly1305>().await,
        "aes-256-gcm" => callback.callback::<Aes256Gcm>().await,
        "aes-128-gcm" => callback.callback::<Aes128Gcm>().await,
        // TODO Add more ciphers
        _ => panic!("Unknown cipher: {}", cipher_name),
    }
}

#[cfg(test)]
mod tests {
    use super::cipher_tests::{allocate_space_for_ciphertext, key};
    use super::*;
    use async_trait::async_trait;
    use std::marker::PhantomData;

    struct DummyCallback;
    #[async_trait]
    impl CipherCallback for DummyCallback {
        type Result = ();
        async fn callback<C: Cipher + Send + Sync + 'static>(self) -> Self::Result {
            ()
        }
    }

    #[tokio::test]
    async fn finds_all_available_ciphers() {
        for cipher_name in ["xchacha20-poly1305", "aes-256-gcm", "aes-128-gcm"] {
            lookup_cipher(cipher_name, DummyCallback).await;
        }
    }

    struct CipherEqualityAssertion<ExpectedCipher: Cipher> {
        _p: PhantomData<ExpectedCipher>,
    }
    impl<ExpectedCipher: Cipher> CipherEqualityAssertion<ExpectedCipher> {
        pub fn new() -> Self {
            Self { _p: PhantomData }
        }
    }
    #[async_trait]
    impl<ExpectedCipher: Cipher + Send> CipherCallback for CipherEqualityAssertion<ExpectedCipher> {
        type Result = ();
        async fn callback<ActualCipher: Cipher + Send + Sync + 'static>(self) {
            let plaintext: Data = allocate_space_for_ciphertext::<ExpectedCipher>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
            let expected_cipher = ExpectedCipher::new(key(1));
            let actual_cipher = ActualCipher::new(key(1));
            let encrypted_with_expected = expected_cipher.encrypt(plaintext.clone()).unwrap();
            let encrypted_with_actual = actual_cipher.encrypt(plaintext.clone()).unwrap();
            assert_eq!(
                plaintext.clone(),
                actual_cipher.decrypt(encrypted_with_expected).unwrap()
            );
            assert_eq!(
                plaintext.clone(),
                expected_cipher.decrypt(encrypted_with_actual).unwrap()
            );
        }
    }

    #[tokio::test]
    #[should_panic(expected = "Unknown cipher: unknown-cipher")]
    async fn lookup_unknown_cipher() {
        lookup_cipher("unknown-cipher", DummyCallback).await;
    }

    #[tokio::test]
    async fn lookup_finds_correct_cipher() {
        lookup_cipher("aes-128-gcm", CipherEqualityAssertion::<Aes128Gcm>::new()).await;
        lookup_cipher("aes-256-gcm", CipherEqualityAssertion::<Aes256Gcm>::new()).await;
        lookup_cipher(
            "xchacha20-poly1305",
            CipherEqualityAssertion::<XChaCha20Poly1305>::new(),
        )
        .await;
    }
}
