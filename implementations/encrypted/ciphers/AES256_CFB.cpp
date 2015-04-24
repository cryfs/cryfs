#include <cryptopp/cryptopp/modes.h>
#include "AES256_CFB.h"

using CryptoPP::CFB_Mode;
using CryptoPP::AES;

namespace blockstore {
namespace encrypted {

constexpr unsigned int AES256_CFB::IV_SIZE;

void AES256_CFB::encrypt(const byte *plaintext, unsigned int plaintextSize, byte *ciphertext, const EncryptionKey &encKey) {
  FixedSizeData<IV_SIZE> iv = FixedSizeData<IV_SIZE>::CreateRandom();
  auto encryption = CFB_Mode<AES>::Encryption(encKey.data(), encKey.BINARY_LENGTH, iv.data());
  std::memcpy(ciphertext, iv.data(), IV_SIZE);
  encryption.ProcessData(ciphertext + IV_SIZE, plaintext, plaintextSize);
}

void AES256_CFB::decrypt(const byte *ciphertext, byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
  const byte *iv = ciphertext;
  const byte *data = ciphertext + IV_SIZE;
  auto decryption = CFB_Mode<AES>::Decryption((byte*)encKey.data(), encKey.BINARY_LENGTH, iv);
  decryption.ProcessData(plaintext, data, plaintextSize);
}

}
}
