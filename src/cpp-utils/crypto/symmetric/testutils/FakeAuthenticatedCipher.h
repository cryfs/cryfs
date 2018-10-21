#pragma once
#ifndef MESSMER_CPPUTILS_TEST_CRYPTO_SYMMETRIC_TESTUTILS_FAKEAUTHENTICATEDCIPHER_H_
#define MESSMER_CPPUTILS_TEST_CRYPTO_SYMMETRIC_TESTUTILS_FAKEAUTHENTICATEDCIPHER_H_

#include "cpp-utils/crypto/cryptopp_byte.h"
#include "cpp-utils/crypto/symmetric/Cipher.h"
#include "cpp-utils/data/FixedSizeData.h"
#include "cpp-utils/data/Data.h"
#include "cpp-utils/random/RandomGenerator.h"
#include <random>
#include <cpp-utils/data/SerializationHelper.h>

namespace cpputils {

    struct FakeKey {
        static FakeKey FromString(const std::string& keyData) {
          return FakeKey{static_cast<uint64_t>(std::strtol(keyData.c_str(), nullptr, 10))};
        }

        static constexpr unsigned int BINARY_LENGTH = sizeof(uint64_t);

        static FakeKey CreateKey(RandomGenerator &randomGenerator) {
            auto data = randomGenerator.getFixedSize<sizeof(uint64_t)>();
            return FakeKey {*reinterpret_cast<uint64_t*>(data.data())};
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
          serialize<uint64_t>(result.data(), iv);

          //Use xor chiffre on plaintext
          _xor(static_cast<CryptoPP::byte*>(result.dataOffset(sizeof(uint64_t))), plaintext, plaintextSize, encKey.value ^ iv);

          //Add checksum information
          uint64_t checksum = _checksum(static_cast<const CryptoPP::byte*>(result.data()), encKey, plaintextSize + sizeof(uint64_t));
          serialize<uint64_t>(result.dataOffset(plaintextSize + sizeof(uint64_t)), checksum);

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
          uint64_t actualParity = deserialize<uint64_t>(ciphertext + plaintextSize(ciphertextSize) + sizeof(uint64_t));
          if (expectedParity != actualParity) {
            return boost::none;
          }

          //Decrypt xor chiffre from ciphertext
          uint64_t iv = deserialize<uint64_t>(ciphertext);
          Data result(plaintextSize(ciphertextSize));
          _xor(static_cast<CryptoPP::byte *>(result.data()), ciphertext + sizeof(uint64_t), plaintextSize(ciphertextSize), encKey.value ^ iv);

          return std::move(result);
        }

        static constexpr const char *NAME = "FakeAuthenticatedCipher";

    private:
        static uint64_t _checksum(const CryptoPP::byte *data, FakeKey encKey, std::size_t size) {
          uint64_t checksum = 34343435 * encKey.value; // some init value

          for (unsigned int i = 0; i < size; ++i) {
            checksum ^= (static_cast<uint64_t>(data[i]) << (56 - 8 * (i%8)));
          }

          return checksum;
        }

        static void _xor(CryptoPP::byte *dst, const CryptoPP::byte *src, unsigned int size, uint64_t key) {
          for (unsigned int i = 0; i < size; ++i) {
            dst[i] = src[i] ^ ((key >> (56 - 8*(i%8))) & 0xFF);
          }
        }

        static std::random_device random_;
    };

}

#endif
