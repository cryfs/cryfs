#include <gtest/gtest.h>
#include <cryfs/config/crypto/CryConfigEncryptorFactory.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/data/DataFixture.h>

using cpputils::SCrypt;
using cpputils::AES256_GCM;
using cpputils::Data;
using cpputils::DataFixture;
using boost::none;
using std::ostream;
using namespace cryfs;

// This is needed for google test
namespace boost {
    inline ostream &operator<<(ostream &stream, const CryConfigEncryptor::Decrypted &) {
        return stream << "CryConfigEncryptor::Decrypted()";
    }
}
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

class CryConfigEncryptorFactoryTest: public ::testing::Test {
public:
};

TEST_F(CryConfigEncryptorFactoryTest, EncryptAndDecrypt_SameEncryptor) {
    auto encryptor = CryConfigEncryptorFactory::deriveKey("mypassword", SCrypt::TestSettings);
    Data encrypted = encryptor->encrypt(DataFixture::generate(400), AES256_GCM::NAME);
    auto decrypted = encryptor->decrypt(encrypted).value();
    EXPECT_EQ(DataFixture::generate(400), decrypted.data);
}

TEST_F(CryConfigEncryptorFactoryTest, EncryptAndDecrypt_NewEncryptor) {
    auto encryptor = CryConfigEncryptorFactory::deriveKey("mypassword", SCrypt::TestSettings);
    Data encrypted = encryptor->encrypt(DataFixture::generate(400), AES256_GCM::NAME);

    auto loadedEncryptor = CryConfigEncryptorFactory::loadKey(encrypted, "mypassword").value();
    auto decrypted = loadedEncryptor->decrypt(encrypted).value();
    EXPECT_EQ(DataFixture::generate(400), decrypted.data);
}

TEST_F(CryConfigEncryptorFactoryTest, DoesntDecryptWithWrongPassword) {
    auto encryptor = CryConfigEncryptorFactory::deriveKey("mypassword", SCrypt::TestSettings);
    Data encrypted = encryptor->encrypt(DataFixture::generate(400), AES256_GCM::NAME);

    auto loadedEncryptor = CryConfigEncryptorFactory::loadKey(encrypted, "wrongpassword").value();
    auto decrypted = loadedEncryptor->decrypt(encrypted);
    EXPECT_EQ(none, decrypted);
}

TEST_F(CryConfigEncryptorFactoryTest, DoesntDecryptWithWrongPassword_EmptyData) {
    auto encryptor = CryConfigEncryptorFactory::deriveKey("mypassword", SCrypt::TestSettings);
    Data encrypted = encryptor->encrypt(Data(0), AES256_GCM::NAME);

    auto loadedEncryptor = CryConfigEncryptorFactory::loadKey(encrypted, "wrongpassword").value();
    auto decrypted = loadedEncryptor->decrypt(encrypted);
    EXPECT_EQ(none, decrypted);
}

TEST_F(CryConfigEncryptorFactoryTest, DoesntDecryptInvalidData) {
    auto loadedEncryptor = CryConfigEncryptorFactory::loadKey(Data(0), "mypassword");
    EXPECT_EQ(none, loadedEncryptor);
}