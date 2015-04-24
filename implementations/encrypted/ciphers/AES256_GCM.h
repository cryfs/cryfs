#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_AES256_GCM_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_AES256_GCM_H_

#include "../../../utils/FixedSizeData.h"
#include "../../../utils/Data.h"
#include <cryptopp/cryptopp/aes.h>
#include <boost/optional.hpp>
#include "Cipher.h"

namespace blockstore {
namespace encrypted {

class AES256_GCM {
public:
  BOOST_CONCEPT_ASSERT((CipherConcept<AES256_GCM>));

  using EncryptionKey = FixedSizeData<32>;
  static_assert(32 == CryptoPP::AES::MAX_KEYLENGTH, "If AES offered larger keys, we should offer a variant with it");

  static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
    return plaintextBlockSize + IV_SIZE + TAG_SIZE;
  }

  static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
    return ciphertextBlockSize - IV_SIZE - TAG_SIZE;
  }

  static Data encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey);
  static boost::optional<Data> decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey);

private:
  static constexpr unsigned int IV_SIZE = CryptoPP::AES::BLOCKSIZE;
  static constexpr unsigned int TAG_SIZE = 16;
};

}
}

#endif
