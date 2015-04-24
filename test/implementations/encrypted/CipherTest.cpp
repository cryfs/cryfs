#include <google/gtest/gtest.h>
#include "../../../implementations/encrypted/ciphers/AES256_CFB.h"

#include "../../testutils/DataBlockFixture.h"
#include "../../../utils/Data.h"

using namespace blockstore::encrypted;
using blockstore::Data;

template<class Cipher>
class CipherTest: public ::testing::Test {
public:
  typename Cipher::EncryptionKey encKey = createRandomKey();

  static typename Cipher::EncryptionKey createRandomKey(int seed = 0) {
    DataBlockFixture data(Cipher::EncryptionKey::BINARY_LENGTH, seed);
    return Cipher::EncryptionKey::FromBinary(data.data());
  }

  void CheckEncryptThenDecryptIsIdentity(const Data &plaintext) {
    Data ciphertext(Cipher::ciphertextSize(plaintext.size()));
    Data decrypted(plaintext.size());
    Cipher::encrypt((byte*)plaintext.data(), plaintext.size(), (byte*)ciphertext.data(), this->encKey);
    Cipher::decrypt((byte*)ciphertext.data(), (byte*) decrypted.data(), decrypted.size(), this->encKey);
    EXPECT_EQ(0, std::memcmp(plaintext.data(), decrypted.data(), plaintext.size()));
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

TYPED_TEST_P(CipherTest, Size_0) {
  EXPECT_EQ(0, TypeParam::ciphertextSize(TypeParam::plaintextSize(0)));
}

TYPED_TEST_P(CipherTest, Size_1) {
  EXPECT_EQ(1, TypeParam::ciphertextSize(TypeParam::plaintextSize(1)));
}

TYPED_TEST_P(CipherTest, Size_1024) {
  EXPECT_EQ(1024, TypeParam::ciphertextSize(TypeParam::plaintextSize(1024)));
  EXPECT_EQ(1024, TypeParam::plaintextSize(TypeParam::ciphertextSize(1024)));
}

TYPED_TEST_P(CipherTest, Size_4096) {
  EXPECT_EQ(4096, TypeParam::ciphertextSize(TypeParam::plaintextSize(4096)));
  EXPECT_EQ(4096, TypeParam::plaintextSize(TypeParam::ciphertextSize(4096)));
}

TYPED_TEST_P(CipherTest, Size_1048576) {
  EXPECT_EQ(1048576, TypeParam::ciphertextSize(TypeParam::plaintextSize(1048576)));
  EXPECT_EQ(1048576, TypeParam::plaintextSize(TypeParam::ciphertextSize(1048576)));
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Empty) {
  Data plaintext = this->CreateZeroes(0);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Zeroes_1) {
  Data plaintext = this->CreateZeroes(1);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Data_1) {
  Data plaintext = this->CreateData(1);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Zeroes_100) {
  Data plaintext = this->CreateZeroes(100);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Data_100) {
  Data plaintext = this->CreateData(100);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Zeroes_1024) {
  Data plaintext = this->CreateZeroes(1024);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Data_1024) {
  Data plaintext = this->CreateData(1024);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Zeroes_5000) {
  Data plaintext = this->CreateZeroes(5000);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Data_5000) {
  Data plaintext = this->CreateData(5000);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Zeroes_1MB) {
  Data plaintext = this->CreateZeroes(1048576);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Data_1MB) {
  Data plaintext = this->CreateData(1048576);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Zeroes_50MB) {
  Data plaintext = this->CreateZeroes(52428800);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

TYPED_TEST_P(CipherTest, EncryptThenDecrypt_Data_50MB) {
  Data plaintext = this->CreateData(52428800);
  this->CheckEncryptThenDecryptIsIdentity(plaintext);
}

REGISTER_TYPED_TEST_CASE_P(CipherTest,
    Size_0,
    Size_1,
    Size_1024,
    Size_4096,
    Size_1048576,
    EncryptThenDecrypt_Empty,
    EncryptThenDecrypt_Zeroes_1,
    EncryptThenDecrypt_Data_1,
    EncryptThenDecrypt_Zeroes_100,
    EncryptThenDecrypt_Data_100,
    EncryptThenDecrypt_Zeroes_1024,
    EncryptThenDecrypt_Data_1024,
    EncryptThenDecrypt_Zeroes_5000,
    EncryptThenDecrypt_Data_5000,
    EncryptThenDecrypt_Zeroes_1MB,
    EncryptThenDecrypt_Data_1MB,
    EncryptThenDecrypt_Zeroes_50MB,
    EncryptThenDecrypt_Data_50MB
);

//TODO For authenticated ciphers, we need test cases checking that authentication fails on manipulations

INSTANTIATE_TYPED_TEST_CASE_P(AES256_CFB, CipherTest, AES256_CFB);
