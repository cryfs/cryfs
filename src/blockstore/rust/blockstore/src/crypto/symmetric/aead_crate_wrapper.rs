//! Ciphers from the `aead` crate (and other crates following its traits, for example `aes_gcm`)

use aead::generic_array::typenum::Unsigned;
use aead::{
    generic_array::{ArrayLength, GenericArray},
    AeadInPlace, NewAead, Nonce, AeadCore,
};
use anyhow::{Context, Result};
use rand::{thread_rng, RngCore};
use std::marker::PhantomData;

use super::{Cipher, EncryptionKey};
use crate::data::Data;

// TODO The aes-gcm crate currently needs
// > RUSTFLAGS="-Ctarget-cpu=sandybridge -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"
// to build with hardware acceleration and we build without that, that's why we use it as the SoftwareImplemented version only.
// The announced to do runtime feature detection in the future though, we should then benchmark it against libsodium and possibly
// remove libsodium.
// TODO The chacha20-poly1305 crate needs
// > RUSTFLAGS="-Ctarget-feature=+avx2"
// or it won't use AVX2.

pub struct AeadCipher<C: NewAead + AeadInPlace> {
    encryption_key: EncryptionKey<C::KeySize>,
    _phantom: PhantomData<C>,
}

impl<C: NewAead + AeadInPlace> Cipher for AeadCipher<C> {
    type KeySize = C::KeySize;

    const CIPHERTEXT_OVERHEAD: usize = C::NonceSize::USIZE + C::TagSize::USIZE;

    fn new(encryption_key: EncryptionKey<Self::KeySize>) -> Self {
        Self {
            encryption_key,
            _phantom: PhantomData {},
        }
    }

    fn encrypt(&self, mut plaintext: Data) -> Result<Data> {
        // TODO Move C::new call to constructor so we don't have to do it every time?
        //      Is it actually expensive? Note that we have to somehow migrate the
        //      secret protection we get from our EncryptionKey class then.
        // TODO Is this data layout compatible with the C++ version of EncryptedBlockStore2?
        // TODO Use binary-layout crate here?
        let cipher = C::new(GenericArray::from_slice(self.encryption_key.as_bytes()));
        let ciphertext_size = plaintext.len() + Self::CIPHERTEXT_OVERHEAD;
        let nonce = random_nonce::<C>();
        let auth_tag = cipher
            .encrypt_in_place_detached(&nonce, &[], plaintext.as_mut())
            .context("Encrypting data failed")?;
        let mut ciphertext = plaintext.grow_region(Self::CIPHERTEXT_OVERHEAD, 0).context(
                "Tried to add prefix bytes so we can store ciphertext overhead in libsodium::Aes256Gcm::encrypt").unwrap();
        ciphertext[0..C::NonceSize::USIZE].copy_from_slice(nonce.as_ref());
        ciphertext[C::NonceSize::USIZE..(C::NonceSize::USIZE + C::TagSize::USIZE)]
            .copy_from_slice(auth_tag.as_ref());
        assert_eq!(ciphertext_size, ciphertext.len());
        Ok(ciphertext)
    }

    fn decrypt(&self, mut ciphertext: Data) -> Result<Data> {
        // TODO Move C::new call to constructor so we don't have to do it every time?
        //      Is it actually expensive? Note that we have to somehow migrate the
        //      secret protection we get from our EncryptionKey class then.
        let cipher = C::new(GenericArray::from_slice(self.encryption_key.as_bytes()));
        let ciphertext_len = ciphertext.len();
        let (nonce, rest) = ciphertext.as_mut().split_at_mut(C::NonceSize::USIZE);
        let nonce: &[u8] = nonce;
        let (auth_tag, cipherdata) = rest.split_at_mut(C::TagSize::USIZE);
        let auth_tag: &[u8] = auth_tag;
        cipher
            .decrypt_in_place_detached(nonce.into(), &[], cipherdata.as_mut(), auth_tag.into())
            .context("Decrypting data failed")?;
        let plaintext = ciphertext.into_subregion((C::NonceSize::USIZE + C::TagSize::USIZE)..);
        assert_eq!(
            ciphertext_len
                .checked_sub(Self::CIPHERTEXT_OVERHEAD)
                .unwrap(),
            plaintext.len()
        );
        Ok(plaintext)
    }
}

fn random_nonce<A: AeadCore>() -> Nonce<A> {
    let mut nonce = Nonce::<A>::default();
    let mut rng = thread_rng();
    rng.fill_bytes(&mut nonce);
    nonce
}

// Test cases are in cipher_tests.rs
