#include <cryptopp/cryptopp/modes.h>
#include "AES256_CFB.h"

using CryptoPP::CFB_Mode;
using CryptoPP::AES;
using cpputils::Data;
using cpputils::FixedSizeData;

namespace blockstore {
namespace encrypted {

constexpr unsigned int AES256_CFB::IV_SIZE;

Data AES256_CFB::encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
  FixedSizeData<IV_SIZE> iv = FixedSizeData<IV_SIZE>::CreateRandom();
  auto encryption = CFB_Mode<AES>::Encryption(encKey.data(), encKey.BINARY_LENGTH, iv.data());
  Data ciphertext(ciphertextSize(plaintextSize));
  std::memcpy(ciphertext.data(), iv.data(), IV_SIZE);
  encryption.ProcessData((byte*)ciphertext.data() + IV_SIZE, plaintext, plaintextSize);
  return ciphertext;
}

boost::optional<Data> AES256_CFB::decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey) {
  const byte *ciphertextIV = ciphertext;
  const byte *ciphertextData = ciphertext + IV_SIZE;
  auto decryption = CFB_Mode<AES>::Decryption((byte*)encKey.data(), encKey.BINARY_LENGTH, ciphertextIV);
  Data plaintext(plaintextSize(ciphertextSize));
  decryption.ProcessData((byte*)plaintext.data(), ciphertextData, plaintext.size());
  return std::move(plaintext);
}

}
}
