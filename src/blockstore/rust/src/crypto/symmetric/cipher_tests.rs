#![cfg(test)]

use generic_array::ArrayLength;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use super::aesgcm::{Aes256Gcm_SoftwareImplemented, Aes256Gcm_HardwareAccelerated, Aes256Gcm, Aes128Gcm};
use super::XChaCha20Poly1305;
use super::{Cipher, EncryptionKey};

fn key<L: ArrayLength<u8>>(seed: u64) -> EncryptionKey<L> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; L::USIZE];
    rng.fill_bytes(&mut res);
    EncryptionKey::new(|key_data| {
        key_data.copy_from_slice(&res);
        Ok(())
    }).unwrap()
}

#[generic_tests::define]
mod enc_dec {
    use super::*;

    #[test]
    fn given_emptydata_when_encrypted_then_canbedecrypted<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(1));
        let plaintext = vec![];
        let ciphertext = enc_cipher.encrypt(&plaintext).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext, decrypted_plaintext);
    }

    #[test]
    fn given_somedata_when_encrypted_then_canbedecrypted<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(1));
        let plaintext = hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap();
        let ciphertext = enc_cipher.encrypt(&plaintext).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext, decrypted_plaintext);
    }

    #[test]
    fn given_invalidciphertext_then_doesntdecrypt<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(1));
        let plaintext = hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap();
        let mut ciphertext = enc_cipher.encrypt(&plaintext).unwrap();
        ciphertext[20] ^= 1;
        let decrypted_plaintext = dec_cipher.decrypt(&ciphertext);
        assert!(decrypted_plaintext.is_err());
    }
    
    #[test]
    fn given_differentkey_then_doesntdecrypt<Enc: Cipher, Dec: Cipher>() {
        let enc_cipher = Enc::new(key(1));
        let dec_cipher = Dec::new(key(2));
        let plaintext = hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap();
        let ciphertext = enc_cipher.encrypt(&plaintext).unwrap();
        let decrypted_plaintext = dec_cipher.decrypt(&ciphertext);
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
        let plaintext = vec![];
        let ciphertext = cipher.encrypt(&plaintext).unwrap();
        assert_eq!(plaintext.len(), C::plaintext_size(ciphertext.len()));
        assert_eq!(
            ciphertext.len(),
            C::ciphertext_size(plaintext.len())
        );
    }
    
    #[test]
    fn given_somedata_then_sizecalculationsarecorrect<C: Cipher>() {
        let cipher = C::new(key(1));
        let plaintext = hex::decode("0ffc9a43e15ccfbef1b0880167df335677c9005948eeadb31f89b06b90a364ad03c6b0859652dca960f8fa60c75747c4f0a67f50f5b85b800468559ea1a816173c0abaf5df8f02978a54b250bc57c7c6a55d4d245014722c0b1764718a6d5ca654976370").unwrap();
        let ciphertext = cipher.encrypt(&plaintext).unwrap();
        assert_eq!(plaintext.len(), C::plaintext_size(ciphertext.len()));
        assert_eq!(
            ciphertext.len(),
            C::ciphertext_size(plaintext.len())
        );
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
        let ciphertext = hex::decode("4bbf95c3a8a08d5726c3a9cbf8d49c9b83d6214de41264ede9865f354a2ebc869dbcf937d6c854c8e9f1e670e0874aa8d3e357").unwrap();
        assert_eq!(b"Hello World", &cipher.decrypt(&ciphertext).unwrap().as_ref());
    }    
}

mod aes_128_gcm {
    use super::*;

    #[test]
    fn test_backward_compatibility() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes128Gcm::new(key(1));
        let ciphertext = hex::decode("a42cd01044008c5cc8aa77e8abd6e4ec2b7574bba3b542919b1cb7f6e3c6c41c79e627525364d4").unwrap();
        assert_eq!(b"Hello World", &cipher.decrypt(&ciphertext).unwrap().as_ref());
    }
}

mod aes_256_gcm {
    use super::*;

    #[test]
    fn test_backward_compatibility_software() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes256Gcm_SoftwareImplemented::new(key(1));
        let ciphertext = hex::decode("4821ee76a61a51db1dca87a4450924787d989c3730d2353e9a4697cbb644bef9f5f7ada578a5c2").unwrap();
        assert_eq!(b"Hello World", &cipher.decrypt(&ciphertext).unwrap().as_ref());
    }

    #[test]
    fn test_backward_compatibility_hardware() {
        // Test a preencrypted message to make sure we can still encrypt it
        let cipher = Aes256Gcm_HardwareAccelerated::new(key(1));
        let ciphertext = hex::decode("4821ee76a61a51db1dca87a4450924787d989c3730d2353e9a4697cbb644bef9f5f7ada578a5c2").unwrap();
        assert_eq!(b"Hello World", &cipher.decrypt(&ciphertext).unwrap().as_ref());
    }
}
