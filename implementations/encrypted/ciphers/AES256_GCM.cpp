#include <cryptopp/cryptopp/gcm.h>
#include "AES256_GCM.h"

using CryptoPP::GCM;
using CryptoPP::AES;
using CryptoPP::AuthenticatedEncryptionFilter;
using CryptoPP::AuthenticatedDecryptionFilter;
using CryptoPP::ArraySource;
using CryptoPP::ArraySink;
using CryptoPP::GCM_64K_Tables;
using CryptoPP::HashVerificationFilter;
using cpputils::Data;
using cpputils::FixedSizeData;

namespace blockstore {
namespace encrypted {

constexpr unsigned int AES256_GCM::IV_SIZE;

Data AES256_GCM::encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
  FixedSizeData<IV_SIZE> iv = FixedSizeData<IV_SIZE>::CreateRandom();
  GCM<AES, GCM_64K_Tables>::Encryption encryption;
  encryption.SetKeyWithIV(encKey.data(), encKey.BINARY_LENGTH, iv.data(), IV_SIZE);
  Data ciphertext(ciphertextSize(plaintextSize));

  std::memcpy(ciphertext.data(), iv.data(), IV_SIZE);
  ArraySource(plaintext, plaintextSize, true,
    new AuthenticatedEncryptionFilter(encryption,
      new ArraySink((byte*)ciphertext.data() + IV_SIZE, ciphertext.size() - IV_SIZE),
      false, TAG_SIZE
    )
  );
  return ciphertext;
}

boost::optional<Data> AES256_GCM::decrypt(const byte *ciphertext, unsigned int ciphertextSize, const EncryptionKey &encKey) {
  if (ciphertextSize < IV_SIZE + TAG_SIZE) {
    return boost::none;
  }

  const byte *ciphertextIV = ciphertext;
  const byte *ciphertextData = ciphertext + IV_SIZE;
  GCM<AES, GCM_64K_Tables>::Decryption decryption;
  decryption.SetKeyWithIV((byte*)encKey.data(), encKey.BINARY_LENGTH, ciphertextIV, IV_SIZE);
  Data plaintext(plaintextSize(ciphertextSize));

  try {
    ArraySource((byte*)ciphertextData, ciphertextSize - IV_SIZE, true,
      new AuthenticatedDecryptionFilter(decryption,
        new ArraySink((byte*)plaintext.data(), plaintext.size())
      )
    );
    return std::move(plaintext);
  } catch (const HashVerificationFilter::HashVerificationFailed &e) {
    return boost::none;
  }
}

}
}
