#include <google/gtest/gtest.h>
#include "../../../implementations/encrypted/ciphers/AES256_CFB.h"
#include "../../../implementations/encrypted/ciphers/AES256_GCM.h"
#include "../../../implementations/encrypted/ciphers/Cipher.h"

#include "../../testutils/DataBlockFixture.h"
#include "../../../utils/Data.h"

using namespace blockstore::encrypted;
using blockstore::Data;

template<class Cipher>
class CipherTest: public ::testing::Test {
public:
  BOOST_CONCEPT_ASSERT((CipherConcept<Cipher>));
  typename Cipher::EncryptionKey encKey = createRandomKey();

  static typename Cipher::EncryptionKey createRandomKey(int seed = 0) {
    DataBlockFixture data(Cipher::EncryptionKey::BINARY_LENGTH, seed);
    return Cipher::EncryptionKey::FromBinary(data.data());
  }

  void CheckEncryptThenDecryptIsIdentity(const Data &plaintext) {
    Data ciphertext = Encrypt(plaintext);
    Data decrypted = Decrypt(ciphertext);
    EXPECT_EQ(plaintext.size(), decrypted.size());
    EXPECT_EQ(0, std::memcmp(plaintext.data(), decrypted.data(), plaintext.size()));
  }

  void CheckEncryptIsIndeterministic(const Data &plaintext) {
    Data ciphertext = Encrypt(plaintext);
    Data ciphertext2 = Encrypt(plaintext);
    EXPECT_NE(0, std::memcmp(ciphertext.data(), ciphertext2.data(), ciphertext.size()));
  }

  void CheckEncryptedSize(const Data &plaintext) {
    Data ciphertext = Encrypt(plaintext);
    EXPECT_EQ(Cipher::ciphertextSize(plaintext.size()), ciphertext.size());
  }

  Data Encrypt(const Data &plaintext) {
    return Cipher::encrypt((byte*)plaintext.data(), plaintext.size(), this->encKey);
  }

  Data Decrypt(const Data &ciphertext) {
    return Cipher::decrypt((byte*)ciphertext.data(), ciphertext.size(), this->encKey).value();
  }

  Data CreateZeroes(unsigned int size) {
    Data zeroes(size);
    zeroes.FillWithZeroes();
    return zeroes;
  }

  Data CreateData(unsigned int size, unsigned int seed = 0) {
    DataBlockFixture data(size, seed);
    Data result(size);
    std::memcpy(result.data(), data.data(), size);
    return result;
  }
};

TYPED_TEST_CASE_P(CipherTest);

constexpr std::initializer_list<unsigned int> SIZES = {0, 1, 100, 1024, 5000, 1048576, 20971520};

TYPED_TEST_P(CipherTest, Size) {
  for (auto size: SIZES) {
    EXPECT_EQ(size, TypeParam::ciphertextSize(TypeParam::plaintextSize(size)));
    EXPECT_EQ(size, TypeParam::plaintextSize(TypeParam::ciphertextSize(size)));
  }
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Zeroes) {
  for (auto size: SIZES) {
    Data plaintext = this->CreateZeroes(size);
    this->CheckEncryptThenDecryptIsIdentity(plaintext);
  }
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Data) {
  for (auto size: SIZES) {
    Data plaintext = this->CreateData(size);
    this->CheckEncryptThenDecryptIsIdentity(plaintext);
  }
}

TYPED_TEST_P(CipherTest, EncryptIsIndeterministic_Zeroes) {
  for (auto size: SIZES) {
    Data plaintext = this->CreateZeroes(size);
    this->CheckEncryptIsIndeterministic(plaintext);
  }
}

TYPED_TEST_P(CipherTest, EncryptIsIndeterministic_Data) {
  for (auto size: SIZES) {
    Data plaintext = this->CreateData(size);
    this->CheckEncryptIsIndeterministic(plaintext);
  }
}

TYPED_TEST_P(CipherTest, EncryptedSize) {
  for (auto size: SIZES) {
    Data plaintext = this->CreateData(size);
    this->CheckEncryptedSize(plaintext);
  }
}

REGISTER_TYPED_TEST_CASE_P(CipherTest,
    Size,
    EncryptThenDecrypt_Zeroes,
    EncryptThenDecrypt_Data,
    EncryptIsIndeterministic_Zeroes,
    EncryptIsIndeterministic_Data,
    EncryptedSize
);

//TODO For authenticated ciphers, we need test cases checking that authentication fails on manipulations

INSTANTIATE_TYPED_TEST_CASE_P(AES256_CFB, CipherTest, AES256_CFB);
INSTANTIATE_TYPED_TEST_CASE_P(AES256_GCM, CipherTest, AES256_GCM);
