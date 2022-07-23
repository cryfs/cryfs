//! Ciphers from the `aead` crate (and other crates following its traits, for example `aes_gcm`)

use aead::generic_array::typenum::Unsigned;
use aead::{generic_array::GenericArray, AeadCore, AeadInPlace, NewAead, Nonce};
use anyhow::{ensure, Context, Result};
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

    type CiphertextOverheadPrefix = C::NonceSize;
    type CiphertextOverheadSuffix = C::TagSize;

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
        // TODO For compatibility with the C++ cryfs version, we append nonce in the beginning and tag in the end.
        //      But it is somewhat weird to grow the plaintext input into both directions. We should just grow it in one direction.
        // TODO Use binary-layout crate here?
        let cipher = C::new(GenericArray::from_slice(self.encryption_key.as_bytes()));
        let ciphertext_size = plaintext.len()
            + Self::CiphertextOverheadPrefix::USIZE
            + Self::CiphertextOverheadSuffix::USIZE;
        let nonce = random_nonce::<C>();
        let auth_tag = cipher
            .encrypt_in_place_detached(&nonce, &[], plaintext.as_mut())
            .context("Encrypting data failed")?;
        let mut ciphertext = plaintext;
        ciphertext
            .grow_region_fail_if_reallocation_necessary(Self::CiphertextOverheadPrefix::USIZE, Self::CiphertextOverheadSuffix::USIZE)
            .expect("Tried to add prefix and suffix bytes so we can store ciphertext overhead in libsodium::Aes256Gcm::encrypt");
        ciphertext[..Self::CiphertextOverheadPrefix::USIZE].copy_from_slice(nonce.as_ref());
        ciphertext[(ciphertext_size - Self::CiphertextOverheadSuffix::USIZE)..]
            .copy_from_slice(auth_tag.as_ref());
        assert_eq!(ciphertext_size, ciphertext.len());
        Ok(ciphertext)
    }

    fn decrypt(&self, mut ciphertext: Data) -> Result<Data> {
        ensure!(ciphertext.len() >= Self::CiphertextOverheadPrefix::USIZE + Self::CiphertextOverheadSuffix::USIZE, "Ciphertext is only {} bytes. That's too small to be decrypted, doesn't even have enough space for IV and Tag", ciphertext.len());
        // TODO Move C::new call to constructor so we don't have to do it every time?
        //      Is it actually expensive? Note that we have to somehow migrate the
        //      secret protection we get from our EncryptionKey class then.
        let cipher = C::new(GenericArray::from_slice(self.encryption_key.as_bytes()));
        let ciphertext_len = ciphertext.len();
        let (nonce, rest) = ciphertext
            .as_mut()
            .split_at_mut(Self::CiphertextOverheadPrefix::USIZE);
        let nonce: &[u8] = nonce;
        let (cipherdata, auth_tag) =
            rest.split_at_mut(rest.len() - Self::CiphertextOverheadSuffix::USIZE);
        let auth_tag: &[u8] = auth_tag;
        cipher
            .decrypt_in_place_detached(nonce.into(), &[], cipherdata.as_mut(), auth_tag.into())
            .context("Decrypting data failed")?;
        let mut plaintext = ciphertext;
        plaintext.shrink_to_subregion(
            Self::CiphertextOverheadPrefix::USIZE
                ..(plaintext.len() - Self::CiphertextOverheadSuffix::USIZE),
        );
        assert_eq!(
            ciphertext_len
                .checked_sub(
                    Self::CiphertextOverheadPrefix::USIZE + Self::CiphertextOverheadSuffix::USIZE
                )
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
