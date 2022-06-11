#![cfg(test)]

use generic_array::ArrayLength;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use super::aesgcm::{
    Aes128Gcm, Aes256Gcm, Aes256Gcm_HardwareAccelerated, Aes256Gcm_SoftwareImplemented,
};
use super::XChaCha20Poly1305;
use super::{Cipher, EncryptionKey};
use crate::data::Data;

fn key<L: ArrayLength<u8>>(seed: u64) -> EncryptionKey<L> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; L::USIZE];
    rng.fill_bytes(&mut res);
    EncryptionKey::new(|key_data| {
        key_data.copy_from_slice(&res);
        Ok(())
    })
    .unwrap()
}

// Take a plaintext and make sure it has enough prefix bytes available to transform it into a ciphertext
fn allocate_space_for_ciphertext<C: Cipher>(plaintext: &[u8]) -> Data {
    let mut result = Data::from(vec![0; C::CIPHERTEXT_OVERHEAD + plaintext.len()]);
    result.shrink_to_subregion(C::CIPHERTEXT_OVERHEAD..);
    result.as_mut().copy_from_slice(plaintext);
    result
}

#[generic_tests::define]
mod enc_dec {
    use super::*;

    #[test]
    fn given_emptydata_when_encrypted_then_canbedecrypted<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(1));
        let plaintext = allocate_space_for_ciphertext::<Enc>(&[]);
        let ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into()).unwrap();
        assert_eq!(plaintext.as_ref(), decrypted_plaintext.as_ref());
    }

    #[test]
    fn given_somedata_when_encrypted_then_canbedecrypted<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(1));
        let plaintext = allocate_space_for_ciphertext::<Enc>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into()).unwrap();
        assert_eq!(plaintext.as_ref(), decrypted_plaintext.as_ref());
    }

    #[test]
    fn given_invalidciphertext_then_doesntdecrypt<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(1));
        let plaintext = allocate_space_for_ciphertext::<Enc>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let mut ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        ciphertext[20] ^= 1;
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into());
        assert!(decrypted_plaintext.is_err());
    }

    #[test]
    fn given_differentkey_then_doesntdecrypt<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(2));
        let plaintext = allocate_space_for_ciphertext::<Enc>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext = enc_cipher.encrypt(plaintext.clone().into()).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(ciphertext.into());
        assert!(decrypted_plaintext.is_err());
    }

    #[instantiate_tests(<XChaCha20Poly1305, XChaCha20Poly1305>)]
    mod xchacha20poly1305 {}

    #[instantiate_tests(<Aes128Gcm, Aes128Gcm>)]
    mod aes128gcm {}

    #[instantiate_tests(<Aes256Gcm_SoftwareImplemented, Aes256Gcm_SoftwareImplemented>)]
    mod aes256gcm_software {}

    #[instantiate_tests(<Aes256Gcm_HardwareAccelerated, Aes256Gcm_HardwareAccelerated>)]
    mod aes256gcm_hardware {}

    #[instantiate_tests(<Aes256Gcm, Aes256Gcm>)]
    mod aes256gcm {}

    // Test interoperability (i.e. encrypting with one and decrypting with the other works)
    #[instantiate_tests(<Aes256Gcm_HardwareAccelerated, Aes256Gcm_SoftwareImplemented>)]
    mod aes256gcm_hardware_software {}
    #[instantiate_tests(<Aes256Gcm_SoftwareImplemented, Aes256Gcm_HardwareAccelerated>)]
    mod aes256gcm_software_hardware {}
}

#[generic_tests::define]
mod basics {
    use super::*;

    #[test]
    fn given_emptydata_then_sizecalculationsarecorrect<C: Cipher>() {
        let cipher = C::new(key(1));
        let plaintext = allocate_space_for_ciphertext::<C>(&[]);
        let ciphertext = cipher.encrypt(plaintext.clone().into()).unwrap();
        assert_eq!(plaintext.len(), ciphertext.len() - C::CIPHERTEXT_OVERHEAD);
        assert_eq!(ciphertext.len(), plaintext.len() + C::CIPHERTEXT_OVERHEAD);
    }

    #[test]
    fn given_somedata_then_sizecalculationsarecorrect<C: Cipher>() {
        let cipher = C::new(key(1));
        let plaintext = allocate_space_for_ciphertext::<C>(&hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap());
        let ciphertext = cipher.encrypt(plaintext.clone().into()).unwrap();
        assert_eq!(plaintext.len(), ciphertext.len() - C::CIPHERTEXT_OVERHEAD);
        assert_eq!(ciphertext.len(), plaintext.len() + C::CIPHERTEXT_OVERHEAD);
    }

    #[instantiate_tests(<XChaCha20Poly1305>)]
    mod xchacha20poly1305 {}

    #[instantiate_tests(<Aes128Gcm>)]
    mod aes128gcm {}

    #[instantiate_tests(<Aes256Gcm_SoftwareImplemented>)]
    mod aes256gcm_software {}

    #[instantiate_tests(<Aes256Gcm_HardwareAccelerated>)]
    mod aes256gcm_hardware {}

    #[instantiate_tests(<Aes256Gcm>)]
    mod aes256gcm {}
}

mod xchacha20poly1305 {
    use super::*;

    #[test]
    fn test_backward_compatibility() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = XChaCha20Poly1305::new(key(1));
        let ciphertext = hex::decode("4879c427886b292b57b44cfbc5169ec3b6e87d6f47cd4987c34ad2ef6283c9176e6d4f0f812c96155793a67c2997557d031fbb").unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }
}

mod aes_128_gcm {
    use super::*;

    #[test]
    fn test_backward_compatibility() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes128Gcm::new(key(1));
        let ciphertext = hex::decode(
            "d85772885df1cb9150519483b6e0d4af2230203eeadde9e60e23bb5ee4922bf78295f35a0e0cfc",
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
    fn test_backward_compatibility_software() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes256Gcm_SoftwareImplemented::new(key(1));
        let ciphertext = hex::decode(
            "4f429e7932b58335e205fb2092c89fe3d026d3878453920d73885b053b0d4dd702c21aaf549aa3",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }

    #[test]
    fn test_backward_compatibility_hardware() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes256Gcm_HardwareAccelerated::new(key(1));
        let ciphertext = hex::decode(
            "4f429e7932b58335e205fb2092c89fe3d026d3878453920d73885b053b0d4dd702c21aaf549aa3",
        )
        .unwrap();
        assert_eq!(
            b"Hello World",
            &cipher.decrypt(ciphertext.into()).unwrap().as_ref()
        );
    }
}
