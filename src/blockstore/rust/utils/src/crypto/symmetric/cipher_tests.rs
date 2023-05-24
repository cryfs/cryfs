#![cfg(test)]

use rand::{rngs::StdRng, RngCore, SeedableRng};
// TODO Separate out infallible from lockable and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

use super::aesgcm::{
    AeadAes128Gcm, AeadAes256Gcm, Aes128Gcm, Aes256Gcm, LibsodiumAes256Gcm, OpensslAes128Gcm,
    OpensslAes256Gcm,
};
use super::xchacha20poly1305::{
    AeadXChaCha20Poly1305, LibsodiumXChaCha20Poly1305, XChaCha20Poly1305,
};
use super::{Cipher, CipherDef, EncryptionKey};
use crate::data::Data;

pub fn key(num_bytes: usize, seed: u64) -> EncryptionKey {
    let mut rng = StdRng::seed_from_u64(seed);
    EncryptionKey::new(num_bytes, move |key_data| {
        rng.fill_bytes(key_data);
        Ok(())
    })
    .infallible_unwrap()
}

// Take a plaintext and make sure it has enough prefix bytes available to transform it into a ciphertext
pub fn allocate_space_for_ciphertext<C: CipherDef>(plaintext: &[u8]) -> Data {
    let mut result = Data::from(vec![
        0;
        C::CIPHERTEXT_OVERHEAD_PREFIX
            + C::CIPHERTEXT_OVERHEAD_SUFFIX
            + plaintext.len()
    ]);
    result.shrink_to_subregion(
        C::CIPHERTEXT_OVERHEAD_PREFIX..(C::CIPHERTEXT_OVERHEAD_PREFIX + plaintext.len()),
    );
    result.as_mut().copy_from_slice(plaintext);
    result
}

#[generic_tests::define]
mod enc_dec {
    use super::*;

    #[test]
    fn given_emptydata_when_encrypted_then_canbedecrypted<Enc: CipherDef, Dec: CipherDef>() {
        let enc_cipher = Enc::new(key(Enc::KEY_SIZE, 1)).unwrap();
        let dec_cipher = Dec::new(key(Dec::KEY_SIZE, 1)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<Enc>(&[]);
        let ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into()).unwrap();
        assert_eq!(plaintext.as_ref(), decrypted_plaintext.as_ref());
    }

    #[test]
    fn given_somedata_when_encrypted_then_canbedecrypted<Enc: CipherDef, Dec: CipherDef>() {
        let enc_cipher = Enc::new(key(Enc::KEY_SIZE, 1)).unwrap();
        let dec_cipher = Dec::new(key(Dec::KEY_SIZE, 1)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<Enc>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into()).unwrap();
        assert_eq!(plaintext.as_ref(), decrypted_plaintext.as_ref());
    }

    #[test]
    fn given_invalidciphertext_then_doesntdecrypt<Enc: CipherDef, Dec: CipherDef>() {
        let enc_cipher = Enc::new(key(Enc::KEY_SIZE, 1)).unwrap();
        let dec_cipher = Dec::new(key(Dec::KEY_SIZE, 1)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<Enc>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let mut ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        ciphertext[20] ^= 1;
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into());
        assert!(decrypted_plaintext.is_err());
    }

    #[test]
    fn given_toosmallciphertext_then_doesntdecrypt<Enc: CipherDef, Dec: CipherDef>() {
        let enc_cipher = Enc::new(key(Enc::KEY_SIZE, 1)).unwrap();
        let dec_cipher = Dec::new(key(Dec::KEY_SIZE, 1)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<Enc>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        let ciphertext = &ciphertext[..(ciphertext.len() - 1)];
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.to_vec().into());
        assert!(decrypted_plaintext.is_err());
    }

    #[test]
    fn given_differentkey_then_doesntdecrypt<Enc: CipherDef, Dec: CipherDef>() {
        let enc_cipher = Enc::new(key(Enc::KEY_SIZE, 1)).unwrap();
        let dec_cipher = Dec::new(key(Dec::KEY_SIZE, 2)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<Enc>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into());
        assert!(decrypted_plaintext.is_err());
    }

    #[instantiate_tests(<XChaCha20Poly1305, XChaCha20Poly1305>)]
    mod xchacha20poly1305 {}

    #[instantiate_tests(<AeadXChaCha20Poly1305, AeadXChaCha20Poly1305>)]
    mod xchacha20poly1305_aead {}

    #[instantiate_tests(<LibsodiumXChaCha20Poly1305, LibsodiumXChaCha20Poly1305>)]
    mod xchacha20poly1305_libsodium {}

    #[instantiate_tests(<Aes128Gcm, Aes128Gcm>)]
    mod aes128gcm {}

    #[instantiate_tests(<AeadAes128Gcm, AeadAes128Gcm>)]
    mod aes128gcm_aead {}

    #[instantiate_tests(<OpensslAes128Gcm, OpensslAes128Gcm>)]
    mod aes128gcm_openssl {}

    #[instantiate_tests(<AeadAes256Gcm, AeadAes256Gcm>)]
    mod aes256gcm_aead {}

    #[instantiate_tests(<OpensslAes256Gcm, OpensslAes256Gcm>)]
    mod aes256gcm_openssl {}

    #[instantiate_tests(<LibsodiumAes256Gcm, LibsodiumAes256Gcm>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_libsodium {}

    #[instantiate_tests(<Aes256Gcm, Aes256Gcm>)]
    mod aes256gcm {}

    // Test interoperability for XChaCha20Poly1305 (i.e. encrypting with one and decrypting with the other works)
    #[instantiate_tests(<AeadXChaCha20Poly1305, LibsodiumXChaCha20Poly1305>)]
    mod xchacha20poly1305_aead_libsodium {}
    #[instantiate_tests(<LibsodiumXChaCha20Poly1305, AeadXChaCha20Poly1305>)]
    mod xchacha20poly1305_libsodium_aead {}

    // Test interoperability for AES-128-GCM (i.e. encrypting with one and decrypting with the other works)
    #[instantiate_tests(<AeadAes128Gcm, OpensslAes128Gcm>)]
    mod aes128gcm_aead_openssl {}
    #[instantiate_tests(<OpensslAes128Gcm, AeadAes128Gcm>)]
    mod aes128gcm_openssl_aead {}

    // Test interoperability for AES-256-GCM (i.e. encrypting with one and decrypting with the other works)
    #[instantiate_tests(<AeadAes256Gcm, OpensslAes256Gcm>)]
    mod aes256gcm_aead_openssl {}
    #[instantiate_tests(<OpensslAes256Gcm, AeadAes256Gcm>)]
    mod aes256gcm_openssl_aead {}
    #[instantiate_tests(<AeadAes256Gcm, LibsodiumAes256Gcm>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_aead_libsodium {}
    #[instantiate_tests(<LibsodiumAes256Gcm, AeadAes256Gcm>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_libsodium_aead {}
    #[instantiate_tests(<OpensslAes256Gcm, LibsodiumAes256Gcm>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_openssl_libsodium {}
    #[instantiate_tests(<LibsodiumAes256Gcm, OpensslAes256Gcm>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_libsodium_openssl {}
}

#[generic_tests::define]
mod basics {
    use super::*;

    #[test]
    fn given_emptydata_then_sizecalculationsarecorrect<C: CipherDef>() {
        let cipher = C::new(key(C::KEY_SIZE, 1)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<C>(&[]);
        let ciphertext = cipher.encrypt(plaintext.clone().into()).unwrap();
        assert_eq!(
            plaintext.len(),
            ciphertext.len() - C::CIPHERTEXT_OVERHEAD_PREFIX - C::CIPHERTEXT_OVERHEAD_SUFFIX
        );
        assert_eq!(
            ciphertext.len(),
            plaintext.len() + C::CIPHERTEXT_OVERHEAD_PREFIX + C::CIPHERTEXT_OVERHEAD_SUFFIX
        );
    }

    #[test]
    fn given_somedata_then_sizecalculationsarecorrect<C: CipherDef>() {
        let cipher = C::new(key(C::KEY_SIZE, 1)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<C>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext = cipher.encrypt(plaintext.clone().into()).unwrap();
        assert_eq!(
            plaintext.len(),
            ciphertext.len() - C::CIPHERTEXT_OVERHEAD_PREFIX - C::CIPHERTEXT_OVERHEAD_SUFFIX
        );
        assert_eq!(
            ciphertext.len(),
            plaintext.len() + C::CIPHERTEXT_OVERHEAD_PREFIX + C::CIPHERTEXT_OVERHEAD_SUFFIX
        );
    }

    #[test]
    fn given_zerosizeciphertext_then_doesntdecrypt<C: CipherDef>() {
        let cipher = C::new(key(C::KEY_SIZE, 1)).unwrap();
        let ciphertext = vec![];
        let decrypted_plaintext = cipher.decrypt(ciphertext.into());
        assert!(decrypted_plaintext.is_err());
    }

    #[test]
    fn given_toosmallciphertext_then_doesntdecrypt<C: CipherDef>() {
        let cipher = C::new(key(C::KEY_SIZE, 1)).unwrap();
        let ciphertext = vec![0xab, 0xcd];
        let decrypted_plaintext = cipher.decrypt(ciphertext.into());
        assert!(decrypted_plaintext.is_err());
    }

    #[test]
    fn test_encryption_is_indeterministic<C: CipherDef>() {
        let cipher = C::new(key(C::KEY_SIZE, 1)).unwrap();
        let plaintext = allocate_space_for_ciphertext::<C>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext1 = cipher.encrypt(plaintext.clone().into()).unwrap();
        let ciphertext2 = cipher.encrypt(plaintext.clone().into()).unwrap();
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[instantiate_tests(<XChaCha20Poly1305>)]
    mod xchacha20poly1305 {}

    #[instantiate_tests(<AeadXChaCha20Poly1305>)]
    mod xchacha20poly1305_aead {}

    #[instantiate_tests(<LibsodiumXChaCha20Poly1305>)]
    mod xchacha20poly1305_libsodium {}

    #[instantiate_tests(<Aes128Gcm>)]
    mod aes128gcm {}

    #[instantiate_tests(<AeadAes128Gcm>)]
    mod aes128gcm_aead {}

    #[instantiate_tests(<OpensslAes128Gcm>)]
    mod aes128gcm_openssl {}

    #[instantiate_tests(<AeadAes256Gcm>)]
    mod aes256gcm_aead {}

    #[instantiate_tests(<OpensslAes256Gcm>)]
    mod aes256gcm_openssl {}

    #[instantiate_tests(<LibsodiumAes256Gcm>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_libsodium {}

    #[instantiate_tests(<Aes256Gcm>)]
    mod aes256gcm {}
}

mod xchacha20poly1305 {
    use super::*;

    #[test]
    fn test_backward_compatibility_default() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = XChaCha20Poly1305::new(key(XChaCha20Poly1305::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode("f75cbc1dfb19c7686a90deb76123d628b6ff74a38cdb3a899c9c1d4dc4558bfee4d9e9af7b289436999fe779b47b1a6b95b30f").unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[test]
    fn test_backward_compatibility_aead() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = AeadXChaCha20Poly1305::new(key(AeadXChaCha20Poly1305::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode("f75cbc1dfb19c7686a90deb76123d628b6ff74a38cdb3a899c9c1d4dc4558bfee4d9e9af7b289436999fe779b47b1a6b95b30f").unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[test]
    fn test_backward_compatibility_libsodium() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher =
            LibsodiumXChaCha20Poly1305::new(key(LibsodiumXChaCha20Poly1305::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode("f75cbc1dfb19c7686a90deb76123d628b6ff74a38cdb3a899c9c1d4dc4558bfee4d9e9af7b289436999fe779b47b1a6b95b30f").unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }
}

mod aes_128_gcm {
    use super::*;

    #[test]
    fn test_backward_compatibility_default() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes128Gcm::new(key(Aes128Gcm::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode(
            "3d15d00e18d0bb55a5b7d37614e3621bef03f3758390b98be8d7b0e7a51b4fc07b5af9dc3e19bf",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[test]
    fn test_backward_compatibility_aead() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = AeadAes128Gcm::new(key(AeadAes128Gcm::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode(
            "3d15d00e18d0bb55a5b7d37614e3621bef03f3758390b98be8d7b0e7a51b4fc07b5af9dc3e19bf",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[test]
    fn test_backward_compatibility_openssl() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = OpensslAes128Gcm::new(key(OpensslAes128Gcm::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode(
            "3d15d00e18d0bb55a5b7d37614e3621bef03f3758390b98be8d7b0e7a51b4fc07b5af9dc3e19bf",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }
}

mod aes_256_gcm {
    use super::*;

    #[test]
    fn test_backward_compatibility_default() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes256Gcm::new(key(Aes256Gcm::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode(
            "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[test]
    fn test_backward_compatibility_aead() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = AeadAes256Gcm::new(key(AeadAes256Gcm::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode(
            "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[test]
    fn test_backward_compatibility_openssl() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = OpensslAes256Gcm::new(key(OpensslAes256Gcm::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode(
            "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    #[test]
    fn test_backward_compatibility_libsodium() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = LibsodiumAes256Gcm::new(key(LibsodiumAes256Gcm::KEY_SIZE, 1)).unwrap();
        let ciphertext = hex::decode(
            "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }
}
