#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_GCMCIPHER_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_GCMCIPHER_H_

#include "../../data/FixedSizeData.h"
#include "../../data/Data.h"
#include "../../random/Random.h"
#include <cryptopp/gcm.h>
#include "Cipher.h"

namespace cpputils {

template<typename BlockCipher, unsigned int KeySize>
class GCM_Cipher {
public:
    using EncryptionKey = FixedSizeData<KeySize>;

    static EncryptionKey CreateKey(RandomGenerator &randomGenerator) {
        return randomGenerator.getFixedSize<EncryptionKey::BINARY_LENGTH>();
    }

    static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
        return plaintextBlockSize + IV_SIZE + TAG_SIZE;
    }

    static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
        return ciphertextBlockSize - IV_SIZE - TAG_SIZE;
    }

    static Data encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey);
    static boost::optional<Data> decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey);

private:
    static constexpr unsigned int IV_SIZE = BlockCipher::BLOCKSIZE;
    static constexpr unsigned int TAG_SIZE = 16;
};

template<typename BlockCipher, unsigned int KeySize>
Data GCM_Cipher<BlockCipher, KeySize>::encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
    FixedSizeData<IV_SIZE> iv = Random::PseudoRandom().getFixedSize<IV_SIZE>();
    typename CryptoPP::GCM<BlockCipher, CryptoPP::GCM_64K_Tables>::Encryption encryption;
    encryption.SetKeyWithIV(encKey.data(), encKey.BINARY_LENGTH, iv.data(), IV_SIZE);
    Data ciphertext(ciphertextSize(plaintextSize));

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
boost::optional<Data> GCM_Cipher<BlockCipher, KeySize>::decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey) {
    if (ciphertextSize < IV_SIZE + TAG_SIZE) {
      return boost::none;
    }

    const byte *ciphertextIV = ciphertext;
    const byte *ciphertextData = ciphertext + IV_SIZE;
    typename CryptoPP::GCM<BlockCipher, CryptoPP::GCM_64K_Tables>::Decryption decryption;
    decryption.SetKeyWithIV((byte*)encKey.data(), encKey.BINARY_LENGTH, ciphertextIV, IV_SIZE);
    Data plaintext(plaintextSize(ciphertextSize));

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

#endif
