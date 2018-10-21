#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_GCMCIPHER_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_GCMCIPHER_H_

#include "cpp-utils/crypto/cryptopp_byte.h"
#include "../../data/FixedSizeData.h"
#include "../../data/Data.h"
#include "../../random/Random.h"
#include <vendor_cryptopp/gcm.h>
#include "Cipher.h"
#include "EncryptionKey.h"

namespace cpputils {

template<typename BlockCipher, unsigned int KeySize>
class GCM_Cipher {
public:
    using EncryptionKey = cpputils::EncryptionKey;

    static constexpr unsigned int KEYSIZE = KeySize;
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
    static constexpr unsigned int IV_SIZE = BlockCipher::BLOCKSIZE;
    static constexpr unsigned int TAG_SIZE = 16;
};

template<class BlockCipher, unsigned int KeySize>
constexpr unsigned int GCM_Cipher<BlockCipher, KeySize>::KEYSIZE;
template<class BlockCipher, unsigned int KeySize>
constexpr unsigned int GCM_Cipher<BlockCipher, KeySize>::STRING_KEYSIZE;

template<typename BlockCipher, unsigned int KeySize>
Data GCM_Cipher<BlockCipher, KeySize>::encrypt(const CryptoPP::byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
    ASSERT(encKey.binaryLength() == KeySize, "Wrong key size");

    FixedSizeData<IV_SIZE> iv = Random::PseudoRandom().getFixedSize<IV_SIZE>();
    typename CryptoPP::GCM<BlockCipher, CryptoPP::GCM_64K_Tables>::Encryption encryption;
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

template<typename BlockCipher, unsigned int KeySize>
boost::optional<Data> GCM_Cipher<BlockCipher, KeySize>::decrypt(const CryptoPP::byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey) {
    ASSERT(encKey.binaryLength() == KeySize, "Wrong key size");

    if (ciphertextSize < IV_SIZE + TAG_SIZE) {
      return boost::none;
    }

    const CryptoPP::byte *ciphertextIV = ciphertext;
    const CryptoPP::byte *ciphertextData = ciphertext + IV_SIZE;
    typename CryptoPP::GCM<BlockCipher, CryptoPP::GCM_64K_Tables>::Decryption decryption;
    decryption.SetKeyWithIV(static_cast<const CryptoPP::byte*>(encKey.data()), encKey.binaryLength(), ciphertextIV, IV_SIZE);
    Data plaintext(plaintextSize(ciphertextSize));

    try {
        CryptoPP::ArraySource(static_cast<const CryptoPP::byte*>(ciphertextData), ciphertextSize - IV_SIZE, true,
        new CryptoPP::AuthenticatedDecryptionFilter(decryption,
          new CryptoPP::ArraySink(static_cast<CryptoPP::byte*>(plaintext.data()), plaintext.size()),
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
