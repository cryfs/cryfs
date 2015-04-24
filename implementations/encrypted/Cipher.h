#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHER_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHER_H_

#include <cryptopp/cryptopp/modes.h>

namespace blockstore {
namespace encrypted {

class Cipher {
public:
  virtual ~Cipher() {}

  virtual unsigned int ciphertextBlockSize(unsigned int ciphertextBlockSize) const = 0;
  virtual unsigned int plaintextBlockSize(unsigned int plaintextBlockSize) const = 0;

  virtual void encrypt(const byte *plaintext, unsigned int plaintextSize, byte *ciphertext) const = 0;
  virtual void decrypt(const byte *ciphertext, byte *plaintext, unsigned int plaintextSize) const = 0;
};

}
}



#endif
