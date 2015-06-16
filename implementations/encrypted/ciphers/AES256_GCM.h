#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_AES256_GCM_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_AES256_GCM_H_

#include <messmer/cpp-utils/data/FixedSizeData.h>
#include <messmer/cpp-utils/data/Data.h>
#include <cryptopp/cryptopp/aes.h>
#include "Cipher.h"

namespace blockstore {
namespace encrypted {

class AES256_GCM {
public:
  BOOST_CONCEPT_ASSERT((CipherConcept<AES256_GCM>));

  //TODO Does EncryptionKey::GenerateRandom() use a PseudoRandomGenerator? Would be better to use real randomness. This is true for all ciphers - we should offer a CreateKey() method in Ciphers.
  using EncryptionKey = cpputils::FixedSizeData<32>;
  static_assert(32 == CryptoPP::AES::MAX_KEYLENGTH, "If AES offered larger keys, we should offer a variant with it");

  static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
    return plaintextBlockSize + IV_SIZE + TAG_SIZE;
  }

  static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
    return ciphertextBlockSize - IV_SIZE - TAG_SIZE;
  }

  static cpputils::Data encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey);
  static boost::optional<cpputils::Data> decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey);

private:
  static constexpr unsigned int IV_SIZE = CryptoPP::AES::BLOCKSIZE;
  static constexpr unsigned int TAG_SIZE = 16;
};

}
}

#endif
