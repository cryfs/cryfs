#include <cryfs/impl/config/CryPresetPasswordBasedKeyProvider.h>
#include <gmock/gmock.h>
#include "../../impl/testutils/MockConsole.h"
#include <cpp-utils/data/DataFixture.h>

using cpputils::make_unique_ref;
using cpputils::EncryptionKey;
using cpputils::PasswordBasedKDF;
using cpputils::Data;
using cpputils::DataFixture;
using std::string;
using cryfs::CryPresetPasswordBasedKeyProvider;
using testing::Invoke;
using testing::Eq;
using testing::StrEq;

namespace {

class MockKDF : public PasswordBasedKDF {
public:
    MOCK_METHOD(EncryptionKey, deriveExistingKey, (size_t keySize, const string& password, const Data& kdfParameters), (override));
    MOCK_METHOD(KeyResult, deriveNewKey, (size_t keySize, const string& password), (override));
};

TEST(CryPresetPasswordBasedKeyProviderTest, requestKeyForNewFilesystem) {
    constexpr size_t keySize = 512;
    constexpr const char* password = "mypassword";
    const EncryptionKey key = EncryptionKey::FromString(DataFixture::generate(keySize).ToString());
    auto kdf = make_unique_ref<MockKDF>();
    const Data kdfParameters = DataFixture::generate(100);

    EXPECT_CALL(*kdf, deriveNewKey(Eq(keySize), StrEq(password))).Times(1).WillOnce(Invoke([&] (auto, auto) {return PasswordBasedKDF::KeyResult{key, kdfParameters.copy()};}));

    CryPresetPasswordBasedKeyProvider keyProvider(password, std::move(kdf));
    auto returned_key = keyProvider.requestKeyForNewFilesystem(keySize);

    EXPECT_EQ(key.ToString(), returned_key.key.ToString());
    EXPECT_EQ(kdfParameters, returned_key.kdfParameters);
}

TEST(CryPresetPasswordBasedKeyProviderTest, requestKeyForExistingFilesystem) {
    constexpr size_t keySize = 512;
    constexpr const char* password = "mypassword";
    const EncryptionKey key = EncryptionKey::FromString(DataFixture::generate(keySize).ToString());
    auto kdf = make_unique_ref<MockKDF>();
    const Data kdfParameters = DataFixture::generate(100);

    EXPECT_CALL(*kdf, deriveExistingKey(Eq(keySize), StrEq(password), testing::_)).Times(1).WillOnce(Invoke([&] (auto, auto, const auto& kdfParams) {
        EXPECT_EQ(kdfParameters, kdfParams);
        return key;
    }));

    CryPresetPasswordBasedKeyProvider keyProvider(password, std::move(kdf));
    EncryptionKey returned_key = keyProvider.requestKeyForExistingFilesystem(keySize, kdfParameters);

    EXPECT_EQ(key.ToString(), returned_key.ToString());
}

}
