#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_GCM_CIPHER_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_GCM_CIPHER_H_

#include <messmer/cpp-utils/data/FixedSizeData.h>
#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/random/Random.h>
#include <cryptopp/cryptopp/gcm.h>
#include "Cipher.h"

namespace blockstore {
namespace encrypted {

template<typename BlockCipher, unsigned int KeySize>
class GCM_Cipher {
public:
    BOOST_CONCEPT_ASSERT((CipherConcept<GCM_Cipher<BlockCipher, KeySize>>));

    using EncryptionKey = cpputils::FixedSizeData<KeySize>;

    static EncryptionKey CreateKey() {
        return cpputils::Random::OSRandom().getFixedSize<EncryptionKey::BINARY_LENGTH>();
    }

    // Used in test cases for fast key creation
    static EncryptionKey CreatePseudoRandomKey() {
        return cpputils::Random::PseudoRandom().getFixedSize<EncryptionKey::BINARY_LENGTH>();
    }

    static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
        return plaintextBlockSize + IV_SIZE + TAG_SIZE;
    }

    static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
        return ciphertextBlockSize - IV_SIZE - TAG_SIZE;
    }

    static cpputils::Data encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey);
    static boost::optional<cpputils::Data> decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey);

private:
    static constexpr unsigned int IV_SIZE = BlockCipher::BLOCKSIZE;
    static constexpr unsigned int TAG_SIZE = 16;
};

template<typename BlockCipher, unsigned int KeySize>
cpputils::Data GCM_Cipher<BlockCipher, KeySize>::encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
    cpputils::FixedSizeData<IV_SIZE> iv = cpputils::Random::PseudoRandom().getFixedSize<IV_SIZE>();
    typename CryptoPP::GCM<BlockCipher, CryptoPP::GCM_64K_Tables>::Encryption encryption;
    encryption.SetKeyWithIV(encKey.data(), encKey.BINARY_LENGTH, iv.data(), IV_SIZE);
    cpputils::Data ciphertext(ciphertextSize(plaintextSize));

    std::memcpy(ciphertext.data(), iv.data(), IV_SIZE);
    CryptoPP::ArraySource(plaintext, plaintextSize, true,
      new CryptoPP::AuthenticatedEncryptionFilter(encryption,
        new CryptoPP::ArraySink((byte*)ciphertext.data() + IV_SIZE, ciphertext.size() - IV_SIZE),
        false, TAG_SIZE
      )
    );
    return ciphertext;
}

template<typename BlockCipher, unsigned int KeySize>
boost::optional<cpputils::Data> GCM_Cipher<BlockCipher, KeySize>::decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey) {
    if (ciphertextSize < IV_SIZE + TAG_SIZE) {
      return boost::none;
    }

    const byte *ciphertextIV = ciphertext;
    const byte *ciphertextData = ciphertext + IV_SIZE;
    typename CryptoPP::GCM<BlockCipher, CryptoPP::GCM_64K_Tables>::Decryption decryption;
    decryption.SetKeyWithIV((byte*)encKey.data(), encKey.BINARY_LENGTH, ciphertextIV, IV_SIZE);
    cpputils::Data plaintext(plaintextSize(ciphertextSize));

    try {
        CryptoPP::ArraySource((byte*)ciphertextData, ciphertextSize - IV_SIZE, true,
        new CryptoPP::AuthenticatedDecryptionFilter(decryption,
          new CryptoPP::ArraySink((byte*)plaintext.data(), plaintext.size()),
          CryptoPP::AuthenticatedDecryptionFilter::DEFAULT_FLAGS, TAG_SIZE
        )
      );
      return std::move(plaintext);
    } catch (const CryptoPP::HashVerificationFilter::HashVerificationFailed &e) {
      return boost::none;
    }
}

}
}

#endif
