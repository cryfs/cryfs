use anyhow::{bail, Result};
use async_trait::async_trait;

use cryfs_utils::crypto::symmetric::{
    Aes128Gcm, Aes256Gcm, Cipher, CipherDef, EncryptionKey, XChaCha20Poly1305,
};

// TODO combine AsyncCipherCallback and SyncCipherCallback into one implementation.
//      AsyncCipherCallback should be able to just call into a SyncCipherCallback that returns a Future.

// TODO This way of looking up ciphers at compile time probably bloats up our executable size a lot since it needs to create a whole separate stack of blockstores for each cipher. All for avoiding a dyn encrypt/decrypt call which probably doesn't even make a difference for perf.

// TODO Should we support the other ciphers that were supported by the C++ version?

pub const ALL_CIPHERS: &[&str] = &["xchacha20-poly1305", "aes-256-gcm", "aes-128-gcm"];

// offer a way to lookup ciphers at runtime while statically binding its type
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
    encryption_key: impl FnOnce(usize) -> Result<EncryptionKey>,
) -> Result<Box<dyn Cipher>> {
    struct DynCallback<K: FnOnce(usize) -> Result<EncryptionKey>> {
        encryption_key: K,
    }
    impl<K: FnOnce(usize) -> Result<EncryptionKey>> SyncCipherCallback for DynCallback<K> {
        type Result = Result<Box<dyn Cipher>>;
        fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result {
            let encryption_key = (self.encryption_key)(C::KEY_SIZE)?;
            Ok(Box::new(C::new(encryption_key)?))
        }
    }
    lookup_cipher_sync(cipher_name, DynCallback { encryption_key })?
}

pub fn cipher_is_supported(cipher_name: &str) -> bool {
    struct DummyCallback;
    impl SyncCipherCallback for DummyCallback {
        type Result = ();
        fn callback<C: CipherDef + Send + Sync + 'static>(self) {}
    }
    lookup_cipher_sync(cipher_name, DummyCallback).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use cryfs_utils::data::Data;
    // TODO Separate InfallibleUnwrap from lockable crate and remove lockable crate from our dependencies
    use lockable::InfallibleUnwrap;
    use rand::{rngs::StdRng, RngCore, SeedableRng};
    use std::marker::PhantomData;

    // Take a plaintext and make sure it has enough prefix bytes available to transform it into a ciphertext
    fn allocate_space_for_ciphertext<C: CipherDef>(plaintext: &[u8]) -> Data {
        let mut result = Data::allocate(
            C::CIPHERTEXT_OVERHEAD_PREFIX,
            plaintext.len(),
            C::CIPHERTEXT_OVERHEAD_SUFFIX,
        );
        result.as_mut().copy_from_slice(plaintext);
        result
    }

    fn key(num_bytes: usize, seed: u64) -> EncryptionKey {
        let mut rng = StdRng::seed_from_u64(seed);
        EncryptionKey::new(num_bytes, move |key_data| {
            rng.fill_bytes(key_data);
            Ok(())
        })
        .infallible_unwrap()
    }

    // TODO Test SyncCipherCallback
    // TODO Test lookup_cipher_dyn

    struct DummyCallback;
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
