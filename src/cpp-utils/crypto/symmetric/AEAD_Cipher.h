#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_AEADCIPHER_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_AEADCIPHER_H_

#include "../../data/FixedSizeData.h"
#include "../../data/Data.h"
#include "../../random/Random.h"
#include "Cipher.h"
#include "EncryptionKey.h"

namespace cpputils {

template<class CryptoPPCipher, unsigned int KEYSIZE_, unsigned int IV_SIZE_, unsigned int TAG_SIZE_>
class AEADCipher {
public:
    using EncryptionKey = cpputils::EncryptionKey;

    static constexpr unsigned int KEYSIZE = KEYSIZE_;
    static constexpr unsigned int STRING_KEYSIZE = 2 * KEYSIZE;

    static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
        return plaintextBlockSize + IV_SIZE + TAG_SIZE;
    }

    static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
        return ciphertextBlockSize - IV_SIZE - TAG_SIZE;
    }

    static Data encrypt(const CryptoPP::byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey);
    static boost::optional<Data> decrypt(const CryptoPP::byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey);

private:
    static constexpr unsigned int IV_SIZE = IV_SIZE_;
    static constexpr unsigned int TAG_SIZE = TAG_SIZE_;
};

template<class CryptoPPCipher, unsigned int KEYSIZE_, unsigned int IV_SIZE_, unsigned int TAG_SIZE_>
constexpr unsigned int AEADCipher<CryptoPPCipher, KEYSIZE_, IV_SIZE_, TAG_SIZE_>::KEYSIZE;
template<class CryptoPPCipher, unsigned int KEYSIZE_, unsigned int IV_SIZE_, unsigned int TAG_SIZE_>
constexpr unsigned int AEADCipher<CryptoPPCipher, KEYSIZE_, IV_SIZE_, TAG_SIZE_>::STRING_KEYSIZE;

template<class CryptoPPCipher, unsigned int KEYSIZE_, unsigned int IV_SIZE_, unsigned int TAG_SIZE_>
Data AEADCipher<CryptoPPCipher, KEYSIZE_, IV_SIZE_, TAG_SIZE_>::encrypt(const CryptoPP::byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
    ASSERT(encKey.binaryLength() == AEADCipher::KEYSIZE, "Wrong key size");

    FixedSizeData<IV_SIZE> iv = Random::PseudoRandom().getFixedSize<IV_SIZE>();
    typename CryptoPPCipher::Encryption encryption;
    encryption.SetKeyWithIV(static_cast<const CryptoPP::byte*>(encKey.data()), encKey.binaryLength(), iv.data(), IV_SIZE);
    Data ciphertext(ciphertextSize(plaintextSize));

    iv.ToBinary(ciphertext.data());
    CryptoPP::ArraySource(plaintext, plaintextSize, true,
      new CryptoPP::AuthenticatedEncryptionFilter(encryption,
        new CryptoPP::ArraySink(static_cast<CryptoPP::byte*>(ciphertext.data()) + IV_SIZE, ciphertext.size() - IV_SIZE),
        false, TAG_SIZE
      )
    );
    return ciphertext;
}

template<class CryptoPPCipher, unsigned int KEYSIZE_, unsigned int IV_SIZE_, unsigned int TAG_SIZE_>
boost::optional<Data> AEADCipher<CryptoPPCipher, KEYSIZE_, IV_SIZE_, TAG_SIZE_>::decrypt(const CryptoPP::byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey) {
    ASSERT(encKey.binaryLength() == AEADCipher::KEYSIZE, "Wrong key size");

    if (ciphertextSize < IV_SIZE + TAG_SIZE) {
      return boost::none;
    }

    const CryptoPP::byte *ciphertextIV = ciphertext;
    const CryptoPP::byte *ciphertextData = ciphertext + IV_SIZE;
    typename CryptoPPCipher::Decryption decryption;
    decryption.SetKeyWithIV(static_cast<const CryptoPP::byte*>(encKey.data()), encKey.binaryLength(), ciphertextIV, IV_SIZE);
    Data plaintext(plaintextSize(ciphertextSize));

    try {
        CryptoPP::ArraySource(static_cast<const CryptoPP::byte*>(ciphertextData), ciphertextSize - IV_SIZE, true,
        new CryptoPP::AuthenticatedDecryptionFilter(decryption,
          new CryptoPP::ArraySink(static_cast<CryptoPP::byte*>(plaintext.data()), plaintext.size()),
          CryptoPP::AuthenticatedDecryptionFilter::DEFAULT_FLAGS, TAG_SIZE
        )
      );
      return plaintext;
    } catch (const CryptoPP::HashVerificationFilter::HashVerificationFailed &e) {
      return boost::none;
    }
}
    
}

#endif
