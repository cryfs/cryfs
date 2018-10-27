#include "cpp-utils/crypto/cryptopp_byte.h"
#include <gtest/gtest.h>
#include "cpp-utils/crypto/symmetric/Cipher.h"
#include "cpp-utils/crypto/symmetric/ciphers.h"
#include "cpp-utils/crypto/symmetric/testutils/FakeAuthenticatedCipher.h"

#include "cpp-utils/data/DataFixture.h"
#include "cpp-utils/data/Data.h"
#include <boost/optional/optional_io.hpp>

using namespace cpputils;
using std::string;

template<class Cipher>
class CipherTest: public ::testing::Test {
public:
  BOOST_CONCEPT_ASSERT((CipherConcept<Cipher>));
  typename Cipher::EncryptionKey encKey = createKeyFixture();

  static typename Cipher::EncryptionKey createKeyFixture(int seed = 0) {
    Data data = DataFixture::generate(Cipher::KEYSIZE, seed);
    return Cipher::EncryptionKey::FromString(data.ToString());
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

  void ExpectDoesntDecrypt(const Data &ciphertext) {
    auto decrypted = Cipher::decrypt(static_cast<const CryptoPP::byte*>(ciphertext.data()), ciphertext.size(), this->encKey);
    EXPECT_FALSE(decrypted);
  }

  Data Encrypt(const Data &plaintext) {
    return Cipher::encrypt(static_cast<const CryptoPP::byte*>(plaintext.data()), plaintext.size(), this->encKey);
  }

  Data Decrypt(const Data &ciphertext) {
    return Cipher::decrypt(static_cast<const CryptoPP::byte*>(ciphertext.data()), ciphertext.size(), this->encKey).value();
  }

  static Data CreateZeroes(unsigned int size) {
    return Data(size).FillWithZeroes();
  }

  static Data CreateData(unsigned int size, unsigned int seed = 0) {
    return DataFixture::generate(size, seed);
  }
};

TYPED_TEST_CASE_P(CipherTest);

constexpr std::array<unsigned int, 7> SIZES = {{0, 1, 100, 1024, 5000, 1048576, 20971520}};

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

TYPED_TEST_P(CipherTest, TryDecryptDataThatIsTooSmall) {
  Data tooSmallCiphertext(TypeParam::ciphertextSize(0) - 1);
  this->ExpectDoesntDecrypt(tooSmallCiphertext);
}

TYPED_TEST_P(CipherTest, TryDecryptDataThatIsMuchTooSmall_0) {
  static_assert(TypeParam::ciphertextSize(0) > 0, "If this fails, the test case doesn't make sense.");
  Data tooSmallCiphertext(0);
  this->ExpectDoesntDecrypt(tooSmallCiphertext);
}

TYPED_TEST_P(CipherTest, TryDecryptDataThatIsMuchTooSmall_1) {
  static_assert(TypeParam::ciphertextSize(0) > 1, "If this fails, the test case doesn't make sense.");
  Data tooSmallCiphertext(1);
  this->ExpectDoesntDecrypt(tooSmallCiphertext);
}

REGISTER_TYPED_TEST_CASE_P(CipherTest,
    Size,
    EncryptThenDecrypt_Zeroes,
    EncryptThenDecrypt_Data,
    EncryptIsIndeterministic_Zeroes,
    EncryptIsIndeterministic_Data,
    EncryptedSize,
    TryDecryptDataThatIsTooSmall,
    TryDecryptDataThatIsMuchTooSmall_0,
    TryDecryptDataThatIsMuchTooSmall_1
);

template<class Cipher>
class AuthenticatedCipherTest: public CipherTest<Cipher> {
public:
  Data zeroes1 = CipherTest<Cipher>::CreateZeroes(1);
  Data plaintext1 = CipherTest<Cipher>::CreateData(1);
  Data zeroes2 = CipherTest<Cipher>::CreateZeroes(100 * 1024);
  Data plaintext2 = CipherTest<Cipher>::CreateData(100 * 1024);
};

TYPED_TEST_CASE_P(AuthenticatedCipherTest);

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Zeroes_Size1) {
  Data ciphertext = this->Encrypt(this->zeroes1);
  void* firstByte = ciphertext.data();
  serialize<CryptoPP::byte>(firstByte, deserialize<CryptoPP::byte>(firstByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Data_Size1) {
  Data ciphertext = this->Encrypt(this->plaintext1);
  void* firstByte = ciphertext.data();
  serialize<CryptoPP::byte>(firstByte, deserialize<CryptoPP::byte>(firstByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Zeroes) {
  Data ciphertext = this->Encrypt(this->zeroes2);
  void* firstByte = ciphertext.data();
  serialize<CryptoPP::byte>(firstByte, deserialize<CryptoPP::byte>(firstByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyFirstByte_Data) {
  Data ciphertext = this->Encrypt(this->plaintext2);
  void* firstByte = ciphertext.data();
  serialize<CryptoPP::byte>(firstByte, deserialize<CryptoPP::byte>(firstByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyLastByte_Zeroes) {
  Data ciphertext = this->Encrypt(this->zeroes2);
  void* lastByte = ciphertext.dataOffset(ciphertext.size() - 1);
  serialize<CryptoPP::byte>(lastByte, deserialize<CryptoPP::byte>(lastByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyLastByte_Data) {
  Data ciphertext = this->Encrypt(this->plaintext2);
  void* lastByte = ciphertext.dataOffset(ciphertext.size() - 1);
  serialize<CryptoPP::byte>(lastByte, deserialize<CryptoPP::byte>(lastByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyMiddleByte_Zeroes) {
  Data ciphertext = this->Encrypt(this->zeroes2);
  void* middleByte = ciphertext.dataOffset(ciphertext.size()/2);
  serialize<CryptoPP::byte>(middleByte, deserialize<CryptoPP::byte>(middleByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, ModifyMiddleByte_Data) {
  Data ciphertext = this->Encrypt(this->plaintext2);
  void* middleByte = ciphertext.dataOffset(ciphertext.size()/2);
  serialize<CryptoPP::byte>(middleByte, deserialize<CryptoPP::byte>(middleByte) + 1);
  this->ExpectDoesntDecrypt(ciphertext);
}

TYPED_TEST_P(AuthenticatedCipherTest, TryDecryptZeroesData) {
  this->ExpectDoesntDecrypt(this->zeroes2);
}

TYPED_TEST_P(AuthenticatedCipherTest, TryDecryptRandomData) {
  this->ExpectDoesntDecrypt(this->plaintext2);
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
  TryDecryptRandomData
);


INSTANTIATE_TYPED_TEST_CASE_P(Fake, CipherTest, FakeAuthenticatedCipher);
INSTANTIATE_TYPED_TEST_CASE_P(Fake, AuthenticatedCipherTest, FakeAuthenticatedCipher);

INSTANTIATE_TYPED_TEST_CASE_P(AES256_CFB, CipherTest, AES256_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(AES256_GCM, CipherTest, AES256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(AES256_GCM, AuthenticatedCipherTest, AES256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(AES128_CFB, CipherTest, AES128_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(AES128_GCM, CipherTest, AES128_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(AES128_GCM, AuthenticatedCipherTest, AES128_GCM);

INSTANTIATE_TYPED_TEST_CASE_P(Twofish256_CFB, CipherTest, Twofish256_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Twofish256_GCM, CipherTest, Twofish256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Twofish256_GCM, AuthenticatedCipherTest, Twofish256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Twofish128_CFB, CipherTest, Twofish128_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Twofish128_GCM, CipherTest, Twofish128_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Twofish128_GCM, AuthenticatedCipherTest, Twofish128_GCM);

INSTANTIATE_TYPED_TEST_CASE_P(Serpent256_CFB, CipherTest, Serpent256_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Serpent256_GCM, CipherTest, Serpent256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Serpent256_GCM, AuthenticatedCipherTest, Serpent256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Serpent128_CFB, CipherTest, Serpent128_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Serpent128_GCM, CipherTest, Serpent128_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Serpent128_GCM, AuthenticatedCipherTest, Serpent128_GCM);

INSTANTIATE_TYPED_TEST_CASE_P(Cast256_CFB, CipherTest, Cast256_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Cast256_GCM, CipherTest, Cast256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Cast256_GCM, AuthenticatedCipherTest, Cast256_GCM);

#if CRYPTOPP_VERSION != 564
INSTANTIATE_TYPED_TEST_CASE_P(Mars448_CFB, CipherTest, Mars448_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Mars448_GCM, CipherTest, Mars448_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Mars448_GCM, AuthenticatedCipherTest, Mars448_GCM);
#endif
INSTANTIATE_TYPED_TEST_CASE_P(Mars256_CFB, CipherTest, Mars256_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Mars256_GCM, CipherTest, Mars256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Mars256_GCM, AuthenticatedCipherTest, Mars256_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Mars128_CFB, CipherTest, Mars128_CFB); //CFB mode is not authenticated
INSTANTIATE_TYPED_TEST_CASE_P(Mars128_GCM, CipherTest, Mars128_GCM);
INSTANTIATE_TYPED_TEST_CASE_P(Mars128_GCM, AuthenticatedCipherTest, Mars128_GCM);


// Test cipher names
TEST(CipherNameTest, TestCipherNames) {
  EXPECT_EQ("aes-256-gcm", string(AES256_GCM::NAME));
  EXPECT_EQ("aes-256-cfb", string(AES256_CFB::NAME));
  EXPECT_EQ("aes-128-gcm", string(AES128_GCM::NAME));
  EXPECT_EQ("aes-128-cfb", string(AES128_CFB::NAME));

  EXPECT_EQ("twofish-256-gcm", string(Twofish256_GCM::NAME));
  EXPECT_EQ("twofish-256-cfb", string(Twofish256_CFB::NAME));
  EXPECT_EQ("twofish-128-gcm", string(Twofish128_GCM::NAME));
  EXPECT_EQ("twofish-128-cfb", string(Twofish128_CFB::NAME));

  EXPECT_EQ("serpent-256-gcm", string(Serpent256_GCM::NAME));
  EXPECT_EQ("serpent-256-cfb", string(Serpent256_CFB::NAME));
  EXPECT_EQ("serpent-128-gcm", string(Serpent128_GCM::NAME));
  EXPECT_EQ("serpent-128-cfb", string(Serpent128_CFB::NAME));

  EXPECT_EQ("cast-256-gcm", string(Cast256_GCM::NAME));
  EXPECT_EQ("cast-256-cfb", string(Cast256_CFB::NAME));

#if CRYPTOPP_VERSION != 564
  EXPECT_EQ("mars-448-gcm", string(Mars448_GCM::NAME));
  EXPECT_EQ("mars-448-cfb", string(Mars448_CFB::NAME));
#endif
  EXPECT_EQ("mars-256-gcm", string(Mars256_GCM::NAME));
  EXPECT_EQ("mars-256-cfb", string(Mars256_CFB::NAME));
  EXPECT_EQ("mars-128-gcm", string(Mars128_GCM::NAME));
  EXPECT_EQ("mars-128-cfb", string(Mars128_CFB::NAME));
}
