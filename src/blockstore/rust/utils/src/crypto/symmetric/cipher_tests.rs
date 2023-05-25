#![cfg(test)]

use rand::{rngs::StdRng, RngCore, SeedableRng};
// TODO Separate out infallible from lockable and don't depend on lockable from this crate
use generic_array::typenum::{U12, U16};
use lockable::InfallibleUnwrap;

use super::aesgcm::{
    AeadAes128Gcm, AeadAes256Gcm, Aes128Gcm, Aes256Gcm, LibsodiumAes256GcmNonce12,
    OpensslAes128Gcm, OpensslAes256Gcm,
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
    mod aes128gcm_noncedefault {}

    #[instantiate_tests(<Aes128Gcm<U12>, Aes128Gcm<U12>>)]
    mod aes128gcm_nonce12 {}

    #[instantiate_tests(<Aes128Gcm<U16>, Aes128Gcm<U16>>)]
    mod aes128gcm_nonce16 {}

    #[instantiate_tests(<AeadAes128Gcm, AeadAes128Gcm>)]
    mod aes128gcm_noncedefault_aead {}

    #[instantiate_tests(<AeadAes128Gcm<U12>, AeadAes128Gcm<U12>>)]
    mod aes128gcm_nonce12_aead {}

    #[instantiate_tests(<AeadAes128Gcm<U16>, AeadAes128Gcm<U16>>)]
    mod aes128gcm_nonce16_aead {}

    #[instantiate_tests(<OpensslAes128Gcm, OpensslAes128Gcm>)]
    mod aes128gcm_noncedefault_openssl {}

    #[instantiate_tests(<OpensslAes128Gcm<U12>, OpensslAes128Gcm<U12>>)]
    mod aes128gcm_nonce12_openssl {}

    #[instantiate_tests(<OpensslAes128Gcm<U16>, OpensslAes128Gcm<U16>>)]
    mod aes128gcm_nonce16_openssl {}

    #[instantiate_tests(<Aes256Gcm, Aes256Gcm>)]
    mod aes256gcm_noncedefault {}

    #[instantiate_tests(<Aes256Gcm<U12>, Aes256Gcm<U12>>)]
    mod aes256gcm_nonce12 {}

    #[instantiate_tests(<Aes256Gcm<U16>, Aes256Gcm<U16>>)]
    mod aes256gcm_nonce16 {}

    #[instantiate_tests(<AeadAes256Gcm, AeadAes256Gcm>)]
    mod aes256gcm_noncedefault_aead {}

    #[instantiate_tests(<AeadAes256Gcm<U12>, AeadAes256Gcm<U12>>)]
    mod aes256gcm_nonce12_aead {}

    #[instantiate_tests(<AeadAes256Gcm<U16>, AeadAes256Gcm<U16>>)]
    mod aes256gcm_nonce16_aead {}

    #[instantiate_tests(<OpensslAes256Gcm, OpensslAes256Gcm>)]
    mod aes256gcm_noncedefault_openssl {}

    #[instantiate_tests(<OpensslAes256Gcm<U12>, OpensslAes256Gcm<U12>>)]
    mod aes256gcm_nonce12_openssl {}

    #[instantiate_tests(<OpensslAes256Gcm<U16>, OpensslAes256Gcm<U16>>)]
    mod aes256gcm_nonce16_openssl {}

    #[instantiate_tests(<LibsodiumAes256GcmNonce12, LibsodiumAes256GcmNonce12>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_libsodium {}

    // Test interoperability for XChaCha20Poly1305 (i.e. encrypting with one and decrypting with the other works)
    #[instantiate_tests(<AeadXChaCha20Poly1305, LibsodiumXChaCha20Poly1305>)]
    mod xchacha20poly1305_aead_libsodium {}
    #[instantiate_tests(<LibsodiumXChaCha20Poly1305, AeadXChaCha20Poly1305>)]
    mod xchacha20poly1305_libsodium_aead {}

    // Test interoperability (i.e. encrypting with one and decrypting with the other works) for AES-128-GCM with default nonce
    #[instantiate_tests(<AeadAes128Gcm, OpensslAes128Gcm>)]
    mod aes128gcm_noncedefault_aead_openssl {}
    #[instantiate_tests(<OpensslAes128Gcm, AeadAes128Gcm>)]
    mod aes128gcm_noncedefault_openssl_aead {}
    // Test interoperability (i.e. encrypting with one and decrypting with the other works) for AES-128-GCM with nonce of 12 bytes
    #[instantiate_tests(<AeadAes128Gcm<U12>, OpensslAes128Gcm<U12>>)]
    mod aes128gcm_nonce12_aead_openssl {}
    #[instantiate_tests(<OpensslAes128Gcm<U12>, AeadAes128Gcm<U12>>)]
    mod aes128gcm_nonce12_openssl_aead {}
    // Test interoperability (i.e. encrypting with one and decrypting with the other works) for AES-128-GCM with nonce of 16 bytes
    #[instantiate_tests(<AeadAes128Gcm<U16>, OpensslAes128Gcm<U16>>)]
    mod aes128gcm_nonce16_aead_openssl {}
    #[instantiate_tests(<OpensslAes128Gcm<U16>, AeadAes128Gcm<U16>>)]
    mod aes128gcm_nonce16_openssl_aead {}

    // Test interoperability (i.e. encrypting with one and decrypting with the other works) for AES-256-GCM with default nonce size
    #[instantiate_tests(<AeadAes256Gcm, OpensslAes256Gcm>)]
    mod aes256gcm_noncedefault_aead_openssl {}
    #[instantiate_tests(<OpensslAes256Gcm, AeadAes256Gcm>)]
    mod aes256gcm_noncedefault_openssl_aead {}
    // Test interoperability (i.e. encrypting with one and decrypting with the other works) for AES-256-GCM with nonce of 12 bytes
    #[instantiate_tests(<AeadAes256Gcm<U12>, OpensslAes256Gcm<U12>>)]
    mod aes256gcm_nonce12_aead_openssl {}
    #[instantiate_tests(<OpensslAes256Gcm<U12>, AeadAes256Gcm<U12>>)]
    mod aes256gcm_nonce12_openssl_aead {}
    #[instantiate_tests(<AeadAes256Gcm<U12>, LibsodiumAes256GcmNonce12>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_nonce12_aead_libsodium {}
    #[instantiate_tests(<LibsodiumAes256GcmNonce12, AeadAes256Gcm<U12>>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_nonce12_libsodium_aead {}
    #[instantiate_tests(<OpensslAes256Gcm<U12>, LibsodiumAes256GcmNonce12>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_nonce12_openssl_libsodium {}
    #[instantiate_tests(<LibsodiumAes256GcmNonce12, OpensslAes256Gcm<U12>>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_nonce12_libsodium_openssl {}
    // Test interoperability (i.e. encrypting with one and decrypting with the other works) for AES-256-GCM with nonce of 16 bytes
    #[instantiate_tests(<AeadAes256Gcm<U16>, OpensslAes256Gcm<U16>>)]
    mod aes256gcm_nonce16_aead_openssl {}
    #[instantiate_tests(<OpensslAes256Gcm<U16>, AeadAes256Gcm<U16>>)]
    mod aes256gcm_nonce16_openssl_aead {}
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
    mod aes128gcm_noncedefault {}

    #[instantiate_tests(<Aes128Gcm<U12>>)]
    mod aes128gcm_nonce12 {}

    #[instantiate_tests(<Aes128Gcm<U16>>)]
    mod aes128gcm_nonce16 {}

    #[instantiate_tests(<AeadAes128Gcm>)]
    mod aes128gcm_noncedefault_aead {}

    #[instantiate_tests(<AeadAes128Gcm<U12>>)]
    mod aes128gcm_nonce12_aead {}

    #[instantiate_tests(<AeadAes128Gcm<U16>>)]
    mod aes128gcm_nonce16_aead {}

    #[instantiate_tests(<OpensslAes128Gcm>)]
    mod aes128gcm_noncedefault_openssl {}

    #[instantiate_tests(<OpensslAes128Gcm<U12>>)]
    mod aes128gcm_nonce12_openssl {}

    #[instantiate_tests(<OpensslAes128Gcm<U16>>)]
    mod aes128gcm_nonce16_openssl {}

    #[instantiate_tests(<Aes256Gcm>)]
    mod aes256gcm_noncedefault {}

    #[instantiate_tests(<Aes256Gcm<U12>>)]
    mod aes256gcm_nonce12 {}

    #[instantiate_tests(<Aes256Gcm<U16>>)]
    mod aes256gcm_nonce16 {}

    #[instantiate_tests(<AeadAes256Gcm>)]
    mod aes256gcm_noncedefault_aead {}

    #[instantiate_tests(<AeadAes256Gcm<U12>>)]
    mod aes256gcm_nonce12_aead {}

    #[instantiate_tests(<AeadAes256Gcm<U16>>)]
    mod aes256gcm_nonce16_aead {}

    #[instantiate_tests(<OpensslAes256Gcm>)]
    mod aes256gcm_noncedefault_openssl {}

    #[instantiate_tests(<OpensslAes256Gcm<U12>>)]
    mod aes256gcm_nonce12_openssl {}

    #[instantiate_tests(<OpensslAes256Gcm<U16>>)]
    mod aes256gcm_nonce16_openssl {}

    #[instantiate_tests(<LibsodiumAes256GcmNonce12>)]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    mod aes256gcm_libsodium {}
}

macro_rules! backward_compatibility_test {
    ($name:ident, $cipher:ty, $ciphertext:expr) => {
        #[test]
        fn $name() {
            // Test a preencrypted message to make sure we can still encrypt it
            let cipher = <$cipher>::new(key(<$cipher>::KEY_SIZE, 1)).unwrap();
            let ciphertext = hex::decode($ciphertext).unwrap();
            assert_eq!(
                b"Hello World",
                &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
            );
        }
    };
}

mod xchacha20poly1305 {
    use super::*;

    backward_compatibility_test!(
        test_backward_compatibility,
        XChaCha20Poly1305,
        "f75cbc1dfb19c7686a90deb76123d628b6ff74a38cdb3a899c9c1d4dc4558bfee4d9e9af7b289436999fe779b47b1a6b95b30f"
    );
    backward_compatibility_test!(
        test_backward_compatibility_aead,
        AeadXChaCha20Poly1305,
        "f75cbc1dfb19c7686a90deb76123d628b6ff74a38cdb3a899c9c1d4dc4558bfee4d9e9af7b289436999fe779b47b1a6b95b30f"
    );
    backward_compatibility_test!(
        test_backward_compatibility_libsodium,
        LibsodiumXChaCha20Poly1305,
        "f75cbc1dfb19c7686a90deb76123d628b6ff74a38cdb3a899c9c1d4dc4558bfee4d9e9af7b289436999fe779b47b1a6b95b30f"
    );
}

mod aes_128_gcm {
    use super::*;

    backward_compatibility_test!(
        test_backward_compatibility_noncedefault,
        Aes128Gcm,
        "0c8ce256b3b23653f70aff6ca10df6dbb0846372b960ca05612a417fe54a65756e244a2ef169ded2ee78a0"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce16,
        Aes128Gcm<U16>,
        "0c8ce256b3b23653f70aff6ca10df6dbb0846372b960ca05612a417fe54a65756e244a2ef169ded2ee78a0"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce12,
        Aes128Gcm<U12>,
        "3d15d00e18d0bb55a5b7d37614e3621bef03f3758390b98be8d7b0e7a51b4fc07b5af9dc3e19bf"
    );

    backward_compatibility_test!(
        test_backward_compatibility_noncedefault_aead,
        AeadAes128Gcm,
        "0c8ce256b3b23653f70aff6ca10df6dbb0846372b960ca05612a417fe54a65756e244a2ef169ded2ee78a0"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce16_aead,
        AeadAes128Gcm<U16>,
        "0c8ce256b3b23653f70aff6ca10df6dbb0846372b960ca05612a417fe54a65756e244a2ef169ded2ee78a0"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce12_aead,
        AeadAes128Gcm<U12>,
        "3d15d00e18d0bb55a5b7d37614e3621bef03f3758390b98be8d7b0e7a51b4fc07b5af9dc3e19bf"
    );

    backward_compatibility_test!(
        test_backward_compatibility_noncedefault_openssl,
        OpensslAes128Gcm,
        "0c8ce256b3b23653f70aff6ca10df6dbb0846372b960ca05612a417fe54a65756e244a2ef169ded2ee78a0"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce16_openssl,
        OpensslAes128Gcm<U16>,
        "0c8ce256b3b23653f70aff6ca10df6dbb0846372b960ca05612a417fe54a65756e244a2ef169ded2ee78a0"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce12_openssl,
        OpensslAes128Gcm<U12>,
        "3d15d00e18d0bb55a5b7d37614e3621bef03f3758390b98be8d7b0e7a51b4fc07b5af9dc3e19bf"
    );
}

mod aes_256_gcm {
    use super::*;

    backward_compatibility_test!(
        test_backward_compatibility_noncedefault,
        Aes256Gcm,
        "aa9d7b584495665c74b447474d70c1dba27aeada5dd42a901c293bc15902a7395d376ac1bbe077a71464e7"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce16,
        Aes256Gcm<U16>,
        "aa9d7b584495665c74b447474d70c1dba27aeada5dd42a901c293bc15902a7395d376ac1bbe077a71464e7"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce12,
        Aes256Gcm<U12>,
        "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5"
    );

    backward_compatibility_test!(
        test_backward_compatibility_noncedefault_aead,
        AeadAes256Gcm,
        "aa9d7b584495665c74b447474d70c1dba27aeada5dd42a901c293bc15902a7395d376ac1bbe077a71464e7"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce16_aead,
        AeadAes256Gcm<U16>,
        "aa9d7b584495665c74b447474d70c1dba27aeada5dd42a901c293bc15902a7395d376ac1bbe077a71464e7"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce12_aead,
        AeadAes256Gcm<U12>,
        "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5"
    );

    backward_compatibility_test!(
        test_backward_compatibility_noncedefault_openssl,
        OpensslAes256Gcm,
        "aa9d7b584495665c74b447474d70c1dba27aeada5dd42a901c293bc15902a7395d376ac1bbe077a71464e7"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce16_openssl,
        OpensslAes256Gcm<U16>,
        "aa9d7b584495665c74b447474d70c1dba27aeada5dd42a901c293bc15902a7395d376ac1bbe077a71464e7"
    );
    backward_compatibility_test!(
        test_backward_compatibility_nonce12_openssl,
        OpensslAes256Gcm<U12>,
        "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5"
    );

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] // TODO Better aes-ni feature detection
    backward_compatibility_test!(
        test_backward_compatibility_libsodium,
        LibsodiumAes256GcmNonce12,
        "b42e5713993597c702dd8f691402b3f43c65462fb478aca9791d53ea90bdc70e390064be2b94c5"
    );
}
