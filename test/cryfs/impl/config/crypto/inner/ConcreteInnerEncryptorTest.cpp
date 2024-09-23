#include "cpp-utils/data/Data.h"
#include "cryfs/impl/config/crypto/inner/InnerConfig.h"
#include "cryfs/impl/config/crypto/inner/InnerEncryptor.h"
#include <boost/none.hpp>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/data/DataFixture.h>
#include <cryfs/impl/config/crypto/inner/ConcreteInnerEncryptor.h>
#include <cstdint>
#include <gtest/gtest.h>
#include <ostream>
#include <stdexcept>

using std::ostream;
using boost::none;
using cpputils::Data;
using cpputils::DataFixture;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::AES256_GCM;
using cpputils::AES256_CFB;
using cpputils::Twofish128_CFB;
using cpputils::serialize;
using cpputils::deserialize;
using namespace cryfs;

// This is needed for google test
namespace boost {
    inline ostream &operator<<(ostream &stream, const Data &) {
        return stream << "cpputils::Data()";
    }
}

class ConcreteInnerEncryptorTest : public ::testing::Test {
public:
    template<class Cipher>
    unique_ref<InnerEncryptor> makeInnerEncryptor() {
        auto key = Cipher::EncryptionKey::FromString(
            DataFixture::generateFixedSize<Cipher::KEYSIZE>().ToString()
        );
        return make_unique_ref<ConcreteInnerEncryptor<Cipher>>(key);
    }
};

TEST_F(ConcreteInnerEncryptorTest, EncryptAndDecrypt_AES) {
    auto encryptor = makeInnerEncryptor<AES256_GCM>();
    const InnerConfig encrypted = encryptor->encrypt(DataFixture::generate(200));
    const Data decrypted = encryptor->decrypt(encrypted).value();
    EXPECT_EQ(DataFixture::generate(200), decrypted);
}

TEST_F(ConcreteInnerEncryptorTest, EncryptAndDecrypt_Twofish) {
    auto encryptor = makeInnerEncryptor<Twofish128_CFB>();
    const InnerConfig encrypted = encryptor->encrypt(DataFixture::generate(200));
    const Data decrypted = encryptor->decrypt(encrypted).value();
    EXPECT_EQ(DataFixture::generate(200), decrypted);
}

TEST_F(ConcreteInnerEncryptorTest, EncryptAndDecrypt_EmptyData) {
    auto encryptor = makeInnerEncryptor<AES256_GCM>();
    const InnerConfig encrypted = encryptor->encrypt(Data(0));
    const Data decrypted = encryptor->decrypt(encrypted).value();
    EXPECT_EQ(Data(0), decrypted);
}

TEST_F(ConcreteInnerEncryptorTest, DoesntDecryptWithWrongCipherName) {
    auto encryptor = makeInnerEncryptor<Twofish128_CFB>();
    InnerConfig encrypted = encryptor->encrypt(Data(0));
    encrypted.cipherName = AES256_CFB::NAME;
    auto decrypted = encryptor->decrypt(encrypted);
    EXPECT_EQ(none, decrypted);
}

TEST_F(ConcreteInnerEncryptorTest, InvalidCiphertext) {
    auto encryptor = makeInnerEncryptor<AES256_GCM>();
    InnerConfig encrypted = encryptor->encrypt(DataFixture::generate(200));
    serialize<uint8_t>(encrypted.encryptedConfig.data(), deserialize<uint8_t>(encrypted.encryptedConfig.data()) + 1); //Modify ciphertext
    auto decrypted = encryptor->decrypt(encrypted);
    EXPECT_EQ(none, decrypted);
}

TEST_F(ConcreteInnerEncryptorTest, DoesntEncryptWhenTooLarge) {
    auto encryptor = makeInnerEncryptor<AES256_GCM>();
    EXPECT_THROW(
        encryptor->encrypt(DataFixture::generate(2000)),
        std::runtime_error
    );
}

TEST_F(ConcreteInnerEncryptorTest, EncryptionIsFixedSize) {
    auto encryptor = makeInnerEncryptor<AES256_GCM>();
    const InnerConfig encrypted1 = encryptor->encrypt(DataFixture::generate(100));
    const InnerConfig encrypted2 = encryptor->encrypt(DataFixture::generate(200));
    const InnerConfig encrypted3 = encryptor->encrypt(Data(0));

    EXPECT_EQ(encrypted1.encryptedConfig.size(), encrypted2.encryptedConfig.size());
    EXPECT_EQ(encrypted1.encryptedConfig.size(), encrypted3.encryptedConfig.size());
}
