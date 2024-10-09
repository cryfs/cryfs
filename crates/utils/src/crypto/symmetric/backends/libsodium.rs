//! AES-256-GCM implementation based on libsodium. This implementation is hardware accelerated but only works
//! on CPUs that are new enough to have that support. If the CPU doesn't support it, then `Aes256Gcm::new()`
//! will return an error.

use anyhow::{anyhow, ensure, Context, Result};
use sodiumoxide::crypto::aead::{
    aes256gcm as sodium_aes256gcm, xchacha20poly1305_ietf as sodium_xchachapoly1305,
};
use std::sync::Once;

use super::super::{Cipher, CipherDef, EncryptionKey};

use crate::{crypto::symmetric::InvalidKeySizeError, data::Data};

static INIT_LIBSODIUM: Once = Once::new();

fn init_libsodium() {
    INIT_LIBSODIUM.call_once(|| {
        sodiumoxide::init().expect("Failed to initialize libsodium");
    });
}

pub struct Aes256Gcm {
    cipher: sodium_aes256gcm::Aes256Gcm,
    encryption_key: EncryptionKey,
}

impl Aes256Gcm {
    /// Returns true, iff the hardware supports the instructions needed by this
    /// hardware-accelerated implementation of AES
    pub fn is_available() -> bool {
        init_libsodium();
        sodium_aes256gcm::is_available()
    }
}

impl CipherDef for Aes256Gcm {
    const KEY_SIZE: usize = sodium_aes256gcm::KEYBYTES;
    const CIPHERTEXT_OVERHEAD_PREFIX: usize = sodium_aes256gcm::NONCEBYTES;
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize = sodium_aes256gcm::TAGBYTES;

    fn new(encryption_key: EncryptionKey) -> Result<Self, InvalidKeySizeError> {
        if encryption_key.as_bytes().len() != Self::KEY_SIZE {
            return Err(InvalidKeySizeError {
                expected: Self::KEY_SIZE,
                got: encryption_key.as_bytes().len(),
            });
        }

        init_libsodium();

        let cipher = sodium_aes256gcm::Aes256Gcm::new().expect("Hardware doesn't support the instructions needed for this implementation. Please check is_available() before calling new().");
        Ok(Self {
            cipher,
            encryption_key,
        })
    }
}

impl Cipher for Aes256Gcm {
    fn ciphertext_overhead_prefix(&self) -> usize {
        Self::CIPHERTEXT_OVERHEAD_PREFIX
    }

    fn ciphertext_overhead_suffix(&self) -> usize {
        Self::CIPHERTEXT_OVERHEAD_SUFFIX
    }

    fn encrypt(&self, plaintext: Data) -> Result<Data> {
        _encrypt::<
            { Self::CIPHERTEXT_OVERHEAD_PREFIX },
            { Self::CIPHERTEXT_OVERHEAD_SUFFIX },
            sodium_aes256gcm::Nonce,
            sodium_aes256gcm::Key,
            sodium_aes256gcm::Tag,
        >(
            |m, ad, n, k| self.cipher.seal_detached(m, ad, n, k),
            || self.cipher.gen_initial_nonce(),
            // TODO Move convert_key call to constructor so we don't have to do it every time?
            //      Note that we have to somehow migrate the
            //      secret protection we get from our EncryptionKey class then.
            &_convert_key_aes256gcm(&self.encryption_key),
            plaintext,
        )
    }

    fn decrypt(&self, ciphertext: Data) -> Result<Data> {
        _decrypt::<
            { Self::CIPHERTEXT_OVERHEAD_PREFIX },
            { Self::CIPHERTEXT_OVERHEAD_SUFFIX },
            sodium_aes256gcm::Nonce,
            sodium_aes256gcm::Key,
            sodium_aes256gcm::Tag,
        >(
            |m, ad, t, n, k| self.cipher.open_detached(m, ad, t, n, k),
            sodium_aes256gcm::Nonce::from_slice,
            sodium_aes256gcm::Tag::from_slice,
            // TODO Move convert_key call to constructor so we don't have to do it every time?
            //      Note that we have to somehow migrate the
            //      secret protection we get from our EncryptionKey class then.
            &_convert_key_aes256gcm(&self.encryption_key),
            ciphertext,
        )
    }
}

pub struct XChaCha20Poly1305 {
    encryption_key: EncryptionKey,
}

impl CipherDef for XChaCha20Poly1305 {
    const KEY_SIZE: usize = sodium_xchachapoly1305::KEYBYTES;
    const CIPHERTEXT_OVERHEAD_PREFIX: usize = sodium_xchachapoly1305::NONCEBYTES;
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize = sodium_xchachapoly1305::TAGBYTES;

    fn new(encryption_key: EncryptionKey) -> Result<Self, InvalidKeySizeError> {
        if encryption_key.as_bytes().len() != Self::KEY_SIZE {
            return Err(InvalidKeySizeError {
                expected: Self::KEY_SIZE,
                got: encryption_key.as_bytes().len(),
            });
        }

        init_libsodium();

        Ok(Self { encryption_key })
    }
}

impl Cipher for XChaCha20Poly1305 {
    fn ciphertext_overhead_prefix(&self) -> usize {
        Self::CIPHERTEXT_OVERHEAD_PREFIX
    }

    fn ciphertext_overhead_suffix(&self) -> usize {
        Self::CIPHERTEXT_OVERHEAD_SUFFIX
    }

    fn encrypt(&self, plaintext: Data) -> Result<Data> {
        _encrypt::<
            { Self::CIPHERTEXT_OVERHEAD_PREFIX },
            { Self::CIPHERTEXT_OVERHEAD_SUFFIX },
            sodium_xchachapoly1305::Nonce,
            sodium_xchachapoly1305::Key,
            sodium_xchachapoly1305::Tag,
        >(
            sodium_xchachapoly1305::seal_detached,
            sodium_xchachapoly1305::gen_nonce,
            // TODO Move convert_key call to constructor so we don't have to do it every time?
            //      Note that we have to somehow migrate the
            //      secret protection we get from our EncryptionKey class then.
            &_convert_key_xchachapoly1305(&self.encryption_key),
            plaintext,
        )
    }

    fn decrypt(&self, ciphertext: Data) -> Result<Data> {
        _decrypt::<
            { Self::CIPHERTEXT_OVERHEAD_PREFIX },
            { Self::CIPHERTEXT_OVERHEAD_SUFFIX },
            sodium_xchachapoly1305::Nonce,
            sodium_xchachapoly1305::Key,
            sodium_xchachapoly1305::Tag,
        >(
            sodium_xchachapoly1305::open_detached,
            sodium_xchachapoly1305::Nonce::from_slice,
            sodium_xchachapoly1305::Tag::from_slice,
            // TODO Move convert_key call to constructor so we don't have to do it every time?
            //      Note that we have to somehow migrate the
            //      secret protection we get from our EncryptionKey class then.
            &_convert_key_xchachapoly1305(&self.encryption_key),
            ciphertext,
        )
    }
}

fn _encrypt<
    const CIPHERTEXT_OVERHEAD_PREFIX: usize,
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize,
    Nonce: AsRef<[u8]>,
    Key,
    Tag: AsRef<[u8]>,
>(
    seal_fn: impl FnOnce(&mut [u8], Option<&[u8]>, &Nonce, &Key) -> Tag,
    gen_initial_nonce_fn: impl FnOnce() -> Nonce,
    encryption_key: &Key,
    mut plaintext: Data,
) -> Result<Data> {
    // TODO Use binary-layout here?
    let ciphertext_size = plaintext.len() + CIPHERTEXT_OVERHEAD_PREFIX + CIPHERTEXT_OVERHEAD_SUFFIX;
    let nonce = gen_initial_nonce_fn();
    let auth_tag = seal_fn(plaintext.as_mut(), None, &nonce, encryption_key);
    let mut ciphertext = plaintext;
    ciphertext.grow_region_fail_if_reallocation_necessary(CIPHERTEXT_OVERHEAD_PREFIX, CIPHERTEXT_OVERHEAD_SUFFIX).context(
        "Tried to add prefix and suffix bytes so we can store ciphertext overhead in libsodium::encrypt").unwrap();
    ciphertext[..CIPHERTEXT_OVERHEAD_PREFIX].copy_from_slice(nonce.as_ref());
    ciphertext[(ciphertext_size - CIPHERTEXT_OVERHEAD_SUFFIX)..].copy_from_slice(auth_tag.as_ref());
    assert_eq!(ciphertext_size, ciphertext.len());
    Ok(ciphertext)
}

fn _decrypt<
    const CIPHERTEXT_OVERHEAD_PREFIX: usize,
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize,
    Nonce: AsRef<[u8]>,
    Key,
    Tag: AsRef<[u8]>,
>(
    open_fn: impl FnOnce(&mut [u8], Option<&[u8]>, &Tag, &Nonce, &Key) -> Result<(), ()>,
    nonce_from_slice_fn: impl FnOnce(&[u8]) -> Option<Nonce>,
    auth_tag_from_slice_fn: impl FnOnce(&[u8]) -> Option<Tag>,
    encryption_key: &Key,
    mut ciphertext: Data,
) -> Result<Data> {
    ensure!(ciphertext.len() >= CIPHERTEXT_OVERHEAD_PREFIX + CIPHERTEXT_OVERHEAD_SUFFIX, "Ciphertext is only {} bytes. That's too small to be decrypted, doesn't even have enough space for IV and Tag", ciphertext.len());
    let ciphertext_len = ciphertext.len();
    let (nonce, rest) = ciphertext.as_mut().split_at_mut(CIPHERTEXT_OVERHEAD_PREFIX);
    let (cipherdata, auth_tag) = rest.split_at_mut(rest.len() - CIPHERTEXT_OVERHEAD_SUFFIX);
    let nonce = nonce_from_slice_fn(nonce).expect("Wrong nonce size");
    let auth_tag = auth_tag_from_slice_fn(auth_tag).expect("Wrong auth tag size");
    open_fn(
        cipherdata.as_mut(),
        None,
        &auth_tag,
        &nonce,
        &encryption_key,
    )
    .map_err(|()| anyhow!("Decrypting data failed"))?;
    let mut plaintext = ciphertext;
    plaintext.shrink_to_subregion(
        CIPHERTEXT_OVERHEAD_PREFIX..(plaintext.len() - CIPHERTEXT_OVERHEAD_SUFFIX),
    );
    assert_eq!(
        ciphertext_len
            .checked_sub(CIPHERTEXT_OVERHEAD_PREFIX + CIPHERTEXT_OVERHEAD_SUFFIX)
            .unwrap(),
        plaintext.len()
    );
    Ok(plaintext)
}

fn _convert_key_aes256gcm(key: &EncryptionKey) -> sodium_aes256gcm::Key {
    // Panic on error is ok because key size is hard coded and not dependent on input here
    sodium_aes256gcm::Key::from_slice(key.as_bytes()).expect("Invalid key size")
}

fn _convert_key_xchachapoly1305(key: &EncryptionKey) -> sodium_xchachapoly1305::Key {
    // Panic on error is ok because key size is hard coded and not dependent on input here
    sodium_xchachapoly1305::Key::from_slice(key.as_bytes()).expect("Invalid key size")
}

// Test cases are in cipher_tests.rs
