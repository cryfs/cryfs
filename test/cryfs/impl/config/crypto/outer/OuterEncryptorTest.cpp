#include "cpp-utils/data/Data.h"
#include "cryfs/impl/config/crypto/outer/OuterConfig.h"
#include <boost/none.hpp>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/data/DataFixture.h>
#include <cryfs/impl/config/crypto/outer/OuterEncryptor.h>
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
using cpputils::serialize;
using cpputils::deserialize;
using namespace cryfs;

// This is needed for google test
namespace boost {
    inline ostream &operator<<(ostream &stream, const Data &) {
        return stream << "cpputils::Data()";
    }
}

class OuterEncryptorTest : public ::testing::Test {
public:
    Data kdfParameters() {
        return DataFixture::generate(128);
    }

    unique_ref<OuterEncryptor> makeOuterEncryptor() {
        auto key = OuterEncryptor::Cipher::EncryptionKey::FromString(
            DataFixture::generateFixedSize<OuterEncryptor::Cipher::KEYSIZE>().ToString()
        );
        return make_unique_ref<OuterEncryptor>(key, kdfParameters());
    }
};

TEST_F(OuterEncryptorTest, EncryptAndDecrypt) {
    auto encryptor = makeOuterEncryptor();
    const OuterConfig encrypted = encryptor->encrypt(DataFixture::generate(200));
    const Data decrypted = encryptor->decrypt(encrypted).value();
    EXPECT_EQ(DataFixture::generate(200), decrypted);
}

TEST_F(OuterEncryptorTest, EncryptAndDecrypt_EmptyData) {
    auto encryptor = makeOuterEncryptor();
    const OuterConfig encrypted = encryptor->encrypt(Data(0));
    const Data decrypted = encryptor->decrypt(encrypted).value();
    EXPECT_EQ(Data(0), decrypted);
}

TEST_F(OuterEncryptorTest, InvalidCiphertext) {
    auto encryptor = makeOuterEncryptor();
    OuterConfig encrypted = encryptor->encrypt(DataFixture::generate(200));
    serialize<uint8_t>(encrypted.encryptedInnerConfig.data(), deserialize<uint8_t>(encrypted.encryptedInnerConfig.data()) + 1); //Modify ciphertext
    auto decrypted = encryptor->decrypt(encrypted);
    EXPECT_EQ(none, decrypted);
}

TEST_F(OuterEncryptorTest, DoesntEncryptWhenTooLarge) {
    auto encryptor = makeOuterEncryptor();
    EXPECT_THROW(
        encryptor->encrypt(DataFixture::generate(2000)),
        std::runtime_error
    );
}

TEST_F(OuterEncryptorTest, EncryptionIsFixedSize) {
    auto encryptor = makeOuterEncryptor();
    const OuterConfig encrypted1 = encryptor->encrypt(DataFixture::generate(200));
    const OuterConfig encrypted2 = encryptor->encrypt(DataFixture::generate(700));
    const OuterConfig encrypted3 = encryptor->encrypt(Data(0));

    EXPECT_EQ(encrypted1.encryptedInnerConfig.size(), encrypted2.encryptedInnerConfig.size());
    EXPECT_EQ(encrypted1.encryptedInnerConfig.size(), encrypted3.encryptedInnerConfig.size());
}
