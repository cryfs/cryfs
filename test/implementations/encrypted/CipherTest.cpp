#include <google/gtest/gtest.h>
#include "../../../implementations/encrypted/ciphers/AES256_CFB.h"
#include "../../../implementations/encrypted/ciphers/AES256_GCM.h"
#include "../../../implementations/encrypted/ciphers/Cipher.h"

#include <messmer/cpp-utils/data/DataFixture.h>
#include <messmer/cpp-utils/data/Data.h>

 #include <boost/optional/optional_io.hpp>

using namespace blockstore::encrypted;
using cpputils::Data;
using cpputils::DataFixture;

template<class Cipher>
class CipherTest: public ::testing::Test {
public:
  BOOST_CONCEPT_ASSERT((CipherConcept<Cipher>));
  typename Cipher::EncryptionKey encKey = createRandomKey();

  static typename Cipher::EncryptionKey createRandomKey(int seed = 0) {
    Data data = DataFixture::generate(Cipher::EncryptionKey::BINARY_LENGTH, seed);
    return Cipher::EncryptionKey::FromBinary(data.data());
  }

  void CheckEncryptThenDecryptIsIdentity(const Data &plaintext) {
    Data ciphertext = Encrypt(plaintext);
    Data decrypted = Decrypt(ciphertext);
    EXPECT_EQ(plaintext, decrypted);
  }

  void CheckEncryptIsIndeterministic(const Data &plaintext) {
    Data ciphertext = Encrypt(plaintext);
    Data ciphertext2 = Encrypt(plaintext);
    EXPECT_NE(ciphertext, ciphertext2);
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

  static Data CreateZeroes(unsigned int size) {
    return std::move(Data(size).FillWithZeroes());
  }

  static Data CreateData(unsigned int size, unsigned int seed = 0) {
    return DataFixture::generate(size, seed);
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

template<class Cipher>
class AuthenticatedCipherTest: public CipherTest<Cipher> {
public:
  void ExpectDoesntDecrypt(const Data &ciphertext) {
    auto decrypted = Cipher::decrypt((byte*)ciphertext.data(), ciphertext.size(), this->encKey);
    EXPECT_FALSE(decrypted);
  }

  Data zeroes1 = CipherTest<Cipher>::CreateZeroes(1);
  Data plaintext1 = CipherTest<Cipher>::CreateData(1);
  Data zeroes2 = CipherTest<Cipher>::CreateZeroes(100 * 1024);
  Data plaintext2 = CipherTest<Cipher>::CreateData(100 * 1024);
};

TYPED_TEST_CASE_P(AuthenticatedCipherTest);

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Zeroes_Size1) {
  Data ciphertext = this->Encrypt(this->zeroes1);
  *(byte*)ciphertext.data() = *(byte*)ciphertext.data() + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Data_Size1) {
  Data ciphertext = this->Encrypt(this->plaintext1);
  *(byte*)ciphertext.data() = *(byte*)ciphertext.data() + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Zeroes) {
  Data ciphertext = this->Encrypt(this->zeroes2);
  *(byte*)ciphertext.data() = *(byte*)ciphertext.data() + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Data) {
  Data ciphertext = this->Encrypt(this->plaintext2);
  *(byte*)ciphertext.data() = *(byte*)ciphertext.data() + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyLastByte_Zeroes) {
  Data ciphertext = this->Encrypt(this->zeroes2);
  ((byte*)ciphertext.data())[ciphertext.size() - 1] = ((byte*)ciphertext.data())[ciphertext.size() - 1] + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyLastByte_Data) {
  Data ciphertext = this->Encrypt(this->plaintext2);
  ((byte*)ciphertext.data())[ciphertext.size() - 1] = ((byte*)ciphertext.data())[ciphertext.size() - 1] + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyMiddleByte_Zeroes) {
  Data ciphertext = this->Encrypt(this->zeroes2);
  ((byte*)ciphertext.data())[ciphertext.size()/2] = ((byte*)ciphertext.data())[ciphertext.size()/2] + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyMiddleByte_Data) {
  Data ciphertext = this->Encrypt(this->plaintext2);
  ((byte*)ciphertext.data())[ciphertext.size()/2] = ((byte*)ciphertext.data())[ciphertext.size()/2] + 1;
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, TryDecryptZeroesData) {
  this->ExpectDoesntDecrypt(this->zeroes2);
}

TYPED_TEST_P(AuthenticatedCipherTest, TryDecryptRandomData) {
  this->ExpectDoesntDecrypt(this->plaintext2);
}

TYPED_TEST_P(AuthenticatedCipherTest, TryDecryptDataThatIsTooSmall) {
  Data tooSmallCiphertext(TypeParam::ciphertextSize(0) - 1);
  this->ExpectDoesntDecrypt(tooSmallCiphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, TryDecryptDataThatIsMuchTooSmall_0) {
  static_assert(TypeParam::ciphertextSize(0) > 0, "If this fails, the test case doesn't make sense.");
  Data tooSmallCiphertext(0);
  this->ExpectDoesntDecrypt(tooSmallCiphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, TryDecryptDataThatIsMuchTooSmall_1) {
  static_assert(TypeParam::ciphertextSize(0) > 1, "If this fails, the test case doesn't make sense.");
  Data tooSmallCiphertext(1);
  this->ExpectDoesntDecrypt(tooSmallCiphertext);
}

REGISTER_TYPED_TEST_CASE_P(AuthenticatedCipherTest,
  ModifyFirstByte_Zeroes_Size1,
  ModifyFirstByte_Zeroes,
  ModifyFirstByte_Data_Size1,
  ModifyFirstByte_Data,
  ModifyLastByte_Zeroes,
  ModifyLastByte_Data,
  ModifyMiddleByte_Zeroes,
  ModifyMiddleByte_Data,
  TryDecryptZeroesData,
  TryDecryptRandomData,
  TryDecryptDataThatIsTooSmall,
  TryDecryptDataThatIsMuchTooSmall_0,
  TryDecryptDataThatIsMuchTooSmall_1
);


INSTANTIATE_TYPED_TEST_CASE_P(AES256_CFB, CipherTest, AES256_CFB);
INSTANTIATE_TYPED_TEST_CASE_P(AES256_GCM, CipherTest, AES256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(AES256_GCM, AuthenticatedCipherTest, AES256_GCM);
