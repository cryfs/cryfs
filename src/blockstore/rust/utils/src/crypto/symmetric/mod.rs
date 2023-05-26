use crate::data::Data;
use anyhow::{bail, Result};
use async_trait::async_trait;
use static_assertions::const_assert;

pub trait Cipher {
    // TODO Can we make this API safer? It requires the data block passed in to have at least CIPHERTEXT_OVERHEAD prefix bytes available.
    fn encrypt(&self, data: Data) -> Result<Data>;

    fn decrypt(&self, data: Data) -> Result<Data>;

    fn ciphertext_overhead_prefix(&self) -> usize;
    fn ciphertext_overhead_suffix(&self) -> usize;
}

pub trait CipherDef: Cipher + Sized {
    fn new(key: EncryptionKey) -> Result<Self>;

    const KEY_SIZE: usize;

    // How many bytes is a ciphertext larger than a plaintext?
    const CIPHERTEXT_OVERHEAD_PREFIX: usize;
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize;
}

// TODO https://github.com/shadowsocks/crypto2 looks pretty fast, maybe we can use them for faster implementations?
// TODO Ring also looks pretty fast, see https://kerkour.com/rust-symmetric-encryption-aead-benchmark

mod backends;
mod key;

#[cfg(test)]
mod cipher_tests;

pub use key::EncryptionKey;

// export ciphers
mod aesgcm;
pub use aesgcm::{
    AeadAes128Gcm, AeadAes256Gcm, Aes128Gcm, Aes256Gcm, LibsodiumAes256GcmNonce12,
    OpensslAes128Gcm, OpensslAes256Gcm,
};
// TODO Is there an openssl implementation for XChaCha20Poly1305?
mod xchacha20poly1305;
pub use xchacha20poly1305::{AeadXChaCha20Poly1305, LibsodiumXChaCha20Poly1305, XChaCha20Poly1305};

// The [MAX_KEY_SIZE] constant is relied upon in our scrypt key generation because we always generate max size keys, even if we need
// only fewer bytes afterwards. If we change this constant, we need to make sure that scrypt still generates the same
// values even if it gets a different key size as input.
pub const MAX_KEY_SIZE: usize = 56;
const_assert!(XChaCha20Poly1305::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(Aes256Gcm::<aesgcm::DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(Aes128Gcm::<aesgcm::DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);

// TODO combine AsyncCipherCallback and SyncCipherCallback into one implementation.
//      AsyncCipherCallback should be able to just call into a SyncCipherCallback that returns a Future.

// offer a way to lookup ciphers at runtime while statically binding its type
#[async_trait]
pub trait AsyncCipherCallback {
    type Result;

    async fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result;
}
pub trait SyncCipherCallback {
    type Result;

    fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result;
}
pub fn lookup_cipher_sync<CB>(cipher_name: &str, callback: CB) -> Result<CB::Result>
where
    CB: SyncCipherCallback,
{
    match cipher_name {
        "xchacha20-poly1305" => Ok(callback.callback::<XChaCha20Poly1305>()),
        "aes-256-gcm" => Ok(callback.callback::<Aes256Gcm>()),
        "aes-128-gcm" => Ok(callback.callback::<Aes128Gcm>()),
        // TODO Add more ciphers
        _ => bail!("Unknown cipher: {}", cipher_name),
    }
}
pub async fn lookup_cipher_async<CB>(cipher_name: &str, callback: CB) -> Result<CB::Result>
where
    CB: AsyncCipherCallback,
{
    match cipher_name {
        "xchacha20-poly1305" => Ok(callback.callback::<XChaCha20Poly1305>().await),
        "aes-256-gcm" => Ok(callback.callback::<Aes256Gcm>().await),
        "aes-128-gcm" => Ok(callback.callback::<Aes128Gcm>().await),
        // TODO Add more ciphers
        _ => bail!("Unknown cipher: {}", cipher_name),
    }
}
pub fn lookup_cipher_dyn(
    cipher_name: &str,
    encryption_key: impl FnOnce(usize) -> EncryptionKey,
) -> Result<Box<dyn Cipher>> {
    struct DynCallback<K: FnOnce(usize) -> EncryptionKey> {
        encryption_key: K,
    }
    impl<K: FnOnce(usize) -> EncryptionKey> SyncCipherCallback for DynCallback<K> {
        type Result = Result<Box<dyn Cipher>>;
        fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result {
            let encryption_key = (self.encryption_key)(C::KEY_SIZE);
            Ok(Box::new(C::new(encryption_key)?))
        }
    }
    lookup_cipher_sync(cipher_name, DynCallback { encryption_key })?
}

#[cfg(test)]
mod tests {
    use super::cipher_tests::{allocate_space_for_ciphertext, key};
    use super::*;
    use async_trait::async_trait;
    use std::marker::PhantomData;

    // TODO Test SyncCipherCallback
    // TODO Test lookup_cipher_dyn

    struct DummyCallback;
    #[async_trait]
    impl AsyncCipherCallback for DummyCallback {
        type Result = ();
        async fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result {
            ()
        }
    }

    #[tokio::test]
    async fn finds_all_available_ciphers() {
        for cipher_name in ["xchacha20-poly1305", "aes-256-gcm", "aes-128-gcm"] {
            lookup_cipher_async(cipher_name, DummyCallback)
                .await
                .unwrap();
        }
    }

    struct CipherEqualityAssertion<ExpectedCipher: CipherDef> {
        _p: PhantomData<ExpectedCipher>,
    }
    impl<ExpectedCipher: CipherDef> CipherEqualityAssertion<ExpectedCipher> {
        pub fn new() -> Self {
            Self { _p: PhantomData }
        }
    }
    #[async_trait]
    impl<ExpectedCipher: CipherDef + Send> AsyncCipherCallback
        for CipherEqualityAssertion<ExpectedCipher>
    {
        type Result = ();
        async fn callback<ActualCipher: CipherDef + Send + Sync + 'static>(self) {
            let plaintext: Data = allocate_space_for_ciphertext::<ExpectedCipher>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
            let expected_cipher = ExpectedCipher::new(key(ExpectedCipher::KEY_SIZE, 1)).unwrap();
            let actual_cipher = ActualCipher::new(key(ActualCipher::KEY_SIZE, 1)).unwrap();
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
    async fn lookup_unknown_cipher() {
        let err = lookup_cipher_async("unknown-cipher", DummyCallback)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), "Unknown cipher: unknown-cipher");
    }

    #[tokio::test]
    async fn lookup_finds_correct_cipher() {
        lookup_cipher_async("aes-128-gcm", CipherEqualityAssertion::<Aes128Gcm>::new())
            .await
            .unwrap();
        lookup_cipher_async("aes-256-gcm", CipherEqualityAssertion::<Aes256Gcm>::new())
            .await
            .unwrap();
        lookup_cipher_async(
            "xchacha20-poly1305",
            CipherEqualityAssertion::<XChaCha20Poly1305>::new(),
        )
        .await
        .unwrap();
    }
}
