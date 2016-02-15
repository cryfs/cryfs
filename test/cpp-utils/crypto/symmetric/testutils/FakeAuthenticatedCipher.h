#pragma once
#ifndef MESSMER_CPPUTILS_TEST_CRYPTO_SYMMETRIC_TESTUTILS_FAKEAUTHENTICATEDCIPHER_H_
#define MESSMER_CPPUTILS_TEST_CRYPTO_SYMMETRIC_TESTUTILS_FAKEAUTHENTICATEDCIPHER_H_

#include "cpp-utils/crypto/symmetric/Cipher.h"
#include "cpp-utils/data/FixedSizeData.h"
#include "cpp-utils/data/Data.h"
#include "cpp-utils/random/RandomGenerator.h"

namespace cpputils {

    struct FakeKey {
        static FakeKey FromBinary(const void *data) {
          return FakeKey{*(uint8_t *) data};
        }

        static constexpr unsigned int BINARY_LENGTH = 1;

        uint8_t value;
    };

    // This is a fake cipher that uses an indeterministic caesar chiffre and a 4-byte parity for a simple authentication mechanism
    class FakeAuthenticatedCipher {
    public:
        BOOST_CONCEPT_ASSERT((CipherConcept<FakeAuthenticatedCipher>));

        using EncryptionKey = FakeKey;

        static EncryptionKey CreateKey(RandomGenerator &randomGenerator) {
          auto data = randomGenerator.getFixedSize<1>();
          return FakeKey{*((uint8_t *) data.data())};
        }

        static EncryptionKey Key1() {
          return FakeKey{5};
        }

        static EncryptionKey Key2() {
          return FakeKey{63};
        }

        static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
          return plaintextBlockSize + 5;
        }

        static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
          return ciphertextBlockSize - 5;
        }

        static Data encrypt(const byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
          Data result(ciphertextSize(plaintextSize));

          //Add a random IV
          uint8_t iv = rand();
          std::memcpy(result.data(), &iv, 1);

          //Use caesar chiffre on plaintext
          _caesar((byte *) result.data() + 1, plaintext, plaintextSize, encKey.value + iv);

          //Add parity information
          int32_t parity = _parity((byte *) result.data(), plaintextSize + 1);
          std::memcpy((byte *) result.data() + plaintextSize + 1, &parity, 4);

          return result;
        }

        static boost::optional <Data> decrypt(const byte *ciphertext, unsigned int ciphertextSize,
                                              const EncryptionKey &encKey) {
          //We need at least 5 bytes (iv + parity)
          if (ciphertextSize < 5) {
            return boost::none;
          }

          //Check parity
          int32_t expectedParity = _parity(ciphertext, plaintextSize(ciphertextSize) + 1);
          int32_t actualParity = *(int32_t * )(ciphertext + plaintextSize(ciphertextSize) + 1);
          if (expectedParity != actualParity) {
            return boost::none;
          }

          //Decrypt caesar chiffre from ciphertext
          int32_t iv = *(int32_t *) ciphertext;
          Data result(plaintextSize(ciphertextSize));
          _caesar((byte *) result.data(), ciphertext + 1, plaintextSize(ciphertextSize), -(encKey.value + iv));
          return std::move(result);
        }

        static constexpr const char *NAME = "FakeAuthenticatedCipher";

    private:
        static int32_t _parity(const byte *data, unsigned int size) {
          int32_t parity = 34343435; // some init value
          const int32_t *intData = reinterpret_cast<const int32_t *>(data);
          unsigned int intSize = size / sizeof(int32_t);
          for (unsigned int i = 0; i < intSize; ++i) {
            parity = ((int64_t)parity) + intData[i];
          }
          unsigned int remainingBytes = size - 4 * intSize;
          for (unsigned int i = 0; i < remainingBytes; ++i) {
            parity = ((int64_t)parity) + (data[4 * intSize + i] << (24 - 8 * i));
          }
          return parity;
        }

        static void _caesar(byte *dst, const byte *src, unsigned int size, uint8_t key) {
          for (unsigned int i = 0; i < size; ++i) {
            dst[i] = src[i] + key;
          }
        }
    };

}

#endif
