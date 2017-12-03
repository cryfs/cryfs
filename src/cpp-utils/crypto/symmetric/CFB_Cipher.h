#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_CFBCIPHER_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_CFBCIPHER_H_

#include "cpp-utils/crypto/cryptopp_byte.h"
#include "../../data/FixedSizeData.h"
#include "../../data/Data.h"
#include "../../random/Random.h"
#include <boost/optional.hpp>
#include <cryptopp/modes.h>
#include "Cipher.h"
#include "EncryptionKey.h"

namespace cpputils {

template<typename BlockCipher, unsigned int KeySize>
class CFB_Cipher {
public:
  using EncryptionKey = cpputils::EncryptionKey<KeySize>;

  static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
    return plaintextBlockSize + IV_SIZE;
  }

  static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
    return ciphertextBlockSize - IV_SIZE;
  }

  static Data encrypt(const CryptoPP::byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey);
  static boost::optional<Data> decrypt(const CryptoPP::byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey);

private:
  static constexpr unsigned int IV_SIZE = BlockCipher::BLOCKSIZE;
};

template<typename BlockCipher, unsigned int KeySize>
Data CFB_Cipher<BlockCipher, KeySize>::encrypt(const CryptoPP::byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
  FixedSizeData<IV_SIZE> iv = Random::PseudoRandom().getFixedSize<IV_SIZE>();
  auto encryption = typename CryptoPP::CFB_Mode<BlockCipher>::Encryption(static_cast<const CryptoPP::byte*>(encKey.data()), encKey.BINARY_LENGTH, iv.data());
  Data ciphertext(ciphertextSize(plaintextSize));
  iv.ToBinary(ciphertext.data());
  encryption.ProcessData(static_cast<CryptoPP::byte*>(ciphertext.data()) + IV_SIZE, plaintext, plaintextSize);
  return ciphertext;
}

template<typename BlockCipher, unsigned int KeySize>
boost::optional<Data> CFB_Cipher<BlockCipher, KeySize>::decrypt(const CryptoPP::byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey) {
  if (ciphertextSize < IV_SIZE) {
    return boost::none;
  }

  const CryptoPP::byte *ciphertextIV = ciphertext;
  const CryptoPP::byte *ciphertextData = ciphertext + IV_SIZE;
  auto decryption = typename CryptoPP::CFB_Mode<BlockCipher>::Decryption(static_cast<const CryptoPP::byte*>(encKey.data()), encKey.BINARY_LENGTH, ciphertextIV);
  Data plaintext(plaintextSize(ciphertextSize));
  decryption.ProcessData(static_cast<CryptoPP::byte*>(plaintext.data()), ciphertextData, plaintext.size());
  return std::move(plaintext);
}

}

#endif
