#pragma once
#ifndef MESSMER_CPPUTILS_TEST_CRYPTO_SYMMETRIC_TESTUTILS_FAKEAUTHENTICATEDCIPHER_H_
#define MESSMER_CPPUTILS_TEST_CRYPTO_SYMMETRIC_TESTUTILS_FAKEAUTHENTICATEDCIPHER_H_

#include "cpp-utils/crypto/cryptopp_byte.h"
#include "cpp-utils/crypto/symmetric/Cipher.h"
#include "cpp-utils/data/FixedSizeData.h"
#include "cpp-utils/data/Data.h"
#include "cpp-utils/random/RandomGenerator.h"
#include <random>

namespace cpputils {

    struct FakeKey {
        static FakeKey FromBinary(const void *data) {
          return FakeKey{*(uint64_t *) data};
        }

        static constexpr unsigned int BINARY_LENGTH = sizeof(uint64_t);

        static FakeKey CreateKey(RandomGenerator &randomGenerator) {
            auto data = randomGenerator.getFixedSize<sizeof(uint64_t)>();
            return FakeKey{*((uint64_t *) data.data())};
        }

        uint64_t value;
    };

    // This is a fake cipher that uses an indeterministic xor chiffre and a 8-byte checksum for a simple authentication mechanism
    class FakeAuthenticatedCipher {
    public:
        BOOST_CONCEPT_ASSERT((CipherConcept<FakeAuthenticatedCipher>));

        using EncryptionKey = FakeKey;

        static EncryptionKey Key1() {
          return FakeKey{5};
        }

        static EncryptionKey Key2() {
          return FakeKey{63};
        }

        static constexpr unsigned int ciphertextSize(unsigned int plaintextBlockSize) {
          return plaintextBlockSize + sizeof(uint64_t) + sizeof(uint64_t);
        }

        static constexpr unsigned int plaintextSize(unsigned int ciphertextBlockSize) {
          return ciphertextBlockSize - sizeof(uint64_t) - sizeof(uint64_t);
        }

        static Data encrypt(const CryptoPP::byte *plaintext, unsigned int plaintextSize, const EncryptionKey &encKey) {
          Data result(ciphertextSize(plaintextSize));

          //Add a random IV
          uint64_t iv = std::uniform_int_distribution<uint64_t>()(random_);
          std::memcpy(result.data(), &iv, sizeof(uint64_t));

          //Use xor chiffre on plaintext
          _xor((CryptoPP::byte *) result.data() + sizeof(uint64_t), plaintext, plaintextSize, encKey.value ^ iv);

          //Add checksum information
          uint64_t checksum = _checksum((CryptoPP::byte *) result.data(), encKey, plaintextSize + sizeof(uint64_t));
          std::memcpy((CryptoPP::byte *) result.data() + plaintextSize + sizeof(uint64_t), &checksum, sizeof(uint64_t));

          return result;
        }

        static boost::optional <Data> decrypt(const CryptoPP::byte *ciphertext, unsigned int ciphertextSize,
                                              const EncryptionKey &encKey) {
          //We need at least 16 bytes (iv + checksum)
          if (ciphertextSize < 16) {
            return boost::none;
          }

          //Check checksum
          uint64_t expectedParity = _checksum(ciphertext, encKey, plaintextSize(ciphertextSize) + sizeof(uint64_t));
          uint64_t actualParity = *(uint64_t * )(ciphertext + plaintextSize(ciphertextSize) + sizeof(uint64_t));
          if (expectedParity != actualParity) {
            return boost::none;
          }

          //Decrypt xor chiffre from ciphertext
          uint64_t iv = *(uint64_t *) ciphertext;
          Data result(plaintextSize(ciphertextSize));
          _xor((CryptoPP::byte *) result.data(), ciphertext + sizeof(uint64_t), plaintextSize(ciphertextSize), encKey.value ^ iv);
          return std::move(result);
        }

        static constexpr const char *NAME = "FakeAuthenticatedCipher";

    private:
        static uint64_t _checksum(const CryptoPP::byte *data, FakeKey encKey, unsigned int size) {
          uint64_t checksum = 34343435 * encKey.value; // some init value
          const uint64_t *intData = reinterpret_cast<const uint64_t *>(data);
          unsigned int intSize = size / sizeof(uint64_t);
          for (unsigned int i = 0; i < intSize; ++i) {
            checksum = ((uint64_t)checksum) + intData[i];
          }
          unsigned int remainingBytes = size - sizeof(uint64_t) * intSize;
          for (unsigned int i = 0; i < remainingBytes; ++i) {
            checksum = ((uint64_t)checksum) + (data[8 * intSize + i] << (56 - 8 * i));
          }
          return checksum;
        }

        static void _xor(CryptoPP::byte *dst, const CryptoPP::byte *src, unsigned int size, uint64_t key) {
          const uint64_t *srcIntData = reinterpret_cast<const uint64_t *>(src);
          uint64_t *dstIntData = reinterpret_cast<uint64_t *>(dst);
          unsigned int intSize = size / sizeof(uint64_t);
          for (unsigned int i = 0; i < intSize; ++i) {
              dstIntData[i] = srcIntData[i] ^ key;
          }
          unsigned int remainingBytes = size - sizeof(uint64_t) * intSize;
          for (unsigned int i = 0; i < remainingBytes; ++i) {
              dst[8 * intSize + i] = src[8 * intSize + i] ^ ((key >> (56 - 8*i)) & 0xFF);
          }
        }

        static std::random_device random_;
    };

}

#endif
