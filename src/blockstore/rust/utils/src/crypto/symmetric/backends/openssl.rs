use anyhow::{ensure, Context, Result};
use generic_array::{
    typenum::{Unsigned, U16},
    ArrayLength, GenericArray,
};
use openssl::symm::{decrypt_aead, encrypt_aead, Cipher as OpenSSLCipher};
use rand::{thread_rng, RngCore};
use std::marker::PhantomData;

use super::super::{Cipher, CipherDef, EncryptionKey};
use crate::data::Data;

#[allow(non_camel_case_types)]
pub trait CipherType {
    const KEY_SIZE: usize;
    type NONCE_SIZE: ArrayLength<u8>;
    type AUTH_TAG_SIZE: ArrayLength<u8>;

    fn instantiate() -> OpenSSLCipher;
}
pub struct Aes256Gcm<NonceSize: ArrayLength<u8>> {
    _n: PhantomData<NonceSize>,
}
impl<NonceSize: ArrayLength<u8>> CipherType for Aes256Gcm<NonceSize> {
    const KEY_SIZE: usize = 32;
    type NONCE_SIZE = NonceSize;
    type AUTH_TAG_SIZE = U16;

    fn instantiate() -> OpenSSLCipher {
        OpenSSLCipher::aes_256_gcm()
    }
}
pub struct Aes128Gcm<NonceSize: ArrayLength<u8>> {
    _n: PhantomData<NonceSize>,
}
impl<NonceSize: ArrayLength<u8>> CipherType for Aes128Gcm<NonceSize> {
    const KEY_SIZE: usize = 16;
    type NONCE_SIZE = NonceSize;
    type AUTH_TAG_SIZE = U16;

    fn instantiate() -> OpenSSLCipher {
        OpenSSLCipher::aes_128_gcm()
    }
}

pub struct AeadCipher<C: CipherType> {
    encryption_key: EncryptionKey,
    cipher: OpenSSLCipher,
    _c: PhantomData<C>,
}

impl<C: CipherType> CipherDef for AeadCipher<C> {
    const KEY_SIZE: usize = C::KEY_SIZE;
    const CIPHERTEXT_OVERHEAD_PREFIX: usize = C::NONCE_SIZE::USIZE;
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize = C::AUTH_TAG_SIZE::USIZE;

    fn new(encryption_key: EncryptionKey) -> Result<Self> {
        ensure!(
            encryption_key.as_bytes().len() == Self::KEY_SIZE,
            "Expected key size of {} bytes, but got {} bytes",
            Self::KEY_SIZE,
            encryption_key.as_bytes().len()
        );

        let cipher = C::instantiate();

        Ok(Self {
            encryption_key,
            cipher,
            _c: PhantomData,
        })
    }
}

impl<C: CipherType> Cipher for AeadCipher<C> {
    fn ciphertext_overhead_prefix(&self) -> usize {
        Self::CIPHERTEXT_OVERHEAD_PREFIX
    }

    fn ciphertext_overhead_suffix(&self) -> usize {
        Self::CIPHERTEXT_OVERHEAD_SUFFIX
    }

    fn encrypt(&self, plaintext: Data) -> Result<Data> {
        // TODO Use binary-layout here?
        let ciphertext_size =
            plaintext.len() + Self::CIPHERTEXT_OVERHEAD_PREFIX + Self::CIPHERTEXT_OVERHEAD_SUFFIX;
        let mut nonce = GenericArray::<u8, C::NONCE_SIZE>::default();
        thread_rng().fill_bytes(&mut nonce); // TODO Which rng?
        let mut auth_tag = GenericArray::<u8, C::AUTH_TAG_SIZE>::default();

        // TODO Does openssl allow a way to do in-place encryption?
        let ciphertext_output = encrypt_aead(
            self.cipher,
            self.encryption_key.as_bytes(),
            Some(&nonce),
            &[],
            plaintext.as_ref(),
            &mut auth_tag,
        )?;

        // Reuse the plaintext Data object because it has prefix/suffix bytes that we can use so we don't need a reallocation
        let mut ciphertext = plaintext;
        ciphertext.copy_from_slice(&ciphertext_output);

        ciphertext.grow_region_fail_if_reallocation_necessary(Self::CIPHERTEXT_OVERHEAD_PREFIX, Self::CIPHERTEXT_OVERHEAD_SUFFIX).context(
            "Tried to add prefix and suffix bytes so we can store ciphertext overhead in libsodium::Aes256Gcm::encrypt").unwrap();
        ciphertext[..Self::CIPHERTEXT_OVERHEAD_PREFIX].copy_from_slice(nonce.as_ref());
        ciphertext[(ciphertext_size - Self::CIPHERTEXT_OVERHEAD_SUFFIX)..]
            .copy_from_slice(auth_tag.as_ref());
        assert_eq!(ciphertext_size, ciphertext.len());
        Ok(ciphertext)
    }

    fn decrypt(&self, mut ciphertext: Data) -> Result<Data> {
        ensure!(ciphertext.len() >= Self::CIPHERTEXT_OVERHEAD_PREFIX + Self::CIPHERTEXT_OVERHEAD_SUFFIX, "Ciphertext is only {} bytes. That's too small to be decrypted, doesn't even have enough space for IV and Tag", ciphertext.len());
        let ciphertext_len = ciphertext.len();
        let (nonce, rest) = ciphertext
            .as_mut()
            .split_at_mut(Self::CIPHERTEXT_OVERHEAD_PREFIX);
        let (cipherdata, auth_tag) =
            rest.split_at_mut(rest.len() - Self::CIPHERTEXT_OVERHEAD_SUFFIX);

        // TODO Does openssl allow a way to do in-place decryption?
        let plaintext_output = decrypt_aead(
            self.cipher,
            self.encryption_key.as_bytes(),
            Some(&nonce),
            &[],
            cipherdata,
            auth_tag,
        )?;

        // Reuse the ciphertext Data object because it allows us to keep prefix/suffix bytes from it for later use
        let mut plaintext: Data = ciphertext;
        plaintext.shrink_to_subregion(
            Self::CIPHERTEXT_OVERHEAD_PREFIX..(plaintext.len() - Self::CIPHERTEXT_OVERHEAD_SUFFIX),
        );
        plaintext.copy_from_slice(&plaintext_output);
        assert_eq!(
            ciphertext_len
                .checked_sub(Self::CIPHERTEXT_OVERHEAD_PREFIX + Self::CIPHERTEXT_OVERHEAD_SUFFIX)
                .unwrap(),
            plaintext.len()
        );
        Ok(plaintext)
    }
}
