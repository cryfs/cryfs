#include "../../../impl/testutils/FakeCryKeyProvider.h"
#include "cpp-utils/data/Data.h"
#include "cryfs/impl/config/crypto/CryConfigEncryptor.h"
#include <boost/none.hpp>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/data/DataFixture.h>
#include <cryfs/impl/config/crypto/CryConfigEncryptorFactory.h>
#include <gtest/gtest.h>
#include <ostream>

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

class CryConfigEncryptorFactoryTest: public ::testing::Test {
public:
};

TEST_F(CryConfigEncryptorFactoryTest, EncryptAndDecrypt_SameEncryptor) {
    FakeCryKeyProvider keyProvider;
    auto encryptor = CryConfigEncryptorFactory::deriveNewKey(&keyProvider);
    const Data encrypted = encryptor->encrypt(DataFixture::generate(400), AES256_GCM::NAME);
    auto decrypted = encryptor->decrypt(encrypted).value();
    EXPECT_EQ(DataFixture::generate(400), decrypted.data);
}

TEST_F(CryConfigEncryptorFactoryTest, EncryptAndDecrypt_NewEncryptor) {
    FakeCryKeyProvider keyProvider1(1);
    auto encryptor = CryConfigEncryptorFactory::deriveNewKey(&keyProvider1);
    const Data encrypted = encryptor->encrypt(DataFixture::generate(400), AES256_GCM::NAME);

    FakeCryKeyProvider keyProvider2(1);
    auto loadedEncryptor = CryConfigEncryptorFactory::loadExistingKey(encrypted, &keyProvider2).value();
    auto decrypted = loadedEncryptor->decrypt(encrypted).value();
    EXPECT_EQ(DataFixture::generate(400), decrypted.data);
}

TEST_F(CryConfigEncryptorFactoryTest, DoesntDecryptWithWrongKey) {
    FakeCryKeyProvider keyProvider1(1);
    auto encryptor = CryConfigEncryptorFactory::deriveNewKey(&keyProvider1);
    const Data encrypted = encryptor->encrypt(DataFixture::generate(400), AES256_GCM::NAME);

    FakeCryKeyProvider keyProvider2(2);
    auto loadedEncryptor = CryConfigEncryptorFactory::loadExistingKey(encrypted, &keyProvider2).value();
    auto decrypted = loadedEncryptor->decrypt(encrypted);
    EXPECT_EQ(none, decrypted);
}

TEST_F(CryConfigEncryptorFactoryTest, DoesntDecryptWithWrongKey_EmptyData) {
    FakeCryKeyProvider keyProvider1(1);
    auto encryptor = CryConfigEncryptorFactory::deriveNewKey(&keyProvider1);
    const Data encrypted = encryptor->encrypt(Data(0), AES256_GCM::NAME);

    FakeCryKeyProvider keyProvider2(2);
    auto loadedEncryptor = CryConfigEncryptorFactory::loadExistingKey(encrypted, &keyProvider2).value();
    auto decrypted = loadedEncryptor->decrypt(encrypted);
    EXPECT_EQ(none, decrypted);
}

TEST_F(CryConfigEncryptorFactoryTest, DoesntDecryptInvalidData) {
    FakeCryKeyProvider keyProvider;
    auto loadedEncryptor = CryConfigEncryptorFactory::loadExistingKey(Data(0), &keyProvider);
    EXPECT_EQ(none, loadedEncryptor);
}
