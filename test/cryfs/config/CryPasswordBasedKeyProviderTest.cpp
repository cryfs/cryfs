#include <cryfs/config/CryPasswordBasedKeyProvider.h>
#include <gmock/gmock.h>
#include "../testutils/MockConsole.h"
#include <cpp-utils/data/DataFixture.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::EncryptionKey;
using cpputils::PasswordBasedKDF;
using cpputils::Data;
using cpputils::DataFixture;
using std::shared_ptr;
using std::make_shared;
using std::string;
using cryfs::CryPasswordBasedKeyProvider;
using testing::Return;
using testing::Invoke;
using testing::Eq;
using testing::StrEq;
using testing::NiceMock;
using testing::_;

class MockCallable {
public:
  MOCK_METHOD0(call, std::string());
};

class MockKDF : public PasswordBasedKDF {
public:
  MOCK_METHOD3(deriveExistingKey, EncryptionKey(size_t keySize, const string& password, const Data& kdfParameters));
  MOCK_METHOD2(deriveNewKey, KeyResult(size_t keySize, const string& password));
};

class CryPasswordBasedKeyProviderTest : public ::testing::Test {
public:
  CryPasswordBasedKeyProviderTest()
  : mockConsole(make_shared<NiceMock<MockConsole>>())
  , askPasswordForNewFilesystem()
  , askPasswordForExistingFilesystem()
  , kdf_(make_unique_ref<MockKDF>())
  , kdf(kdf_.get())
  , keyProvider(mockConsole, [this] () {return askPasswordForExistingFilesystem.call();}, [this] () {return askPasswordForNewFilesystem.call(); }, std::move(kdf_)) {}

  shared_ptr<NiceMock<MockConsole>> mockConsole;
  MockCallable askPasswordForNewFilesystem;
  MockCallable askPasswordForExistingFilesystem;
  unique_ref<MockKDF> kdf_;
  MockKDF* kdf;

  CryPasswordBasedKeyProvider keyProvider;
};

TEST_F(CryPasswordBasedKeyProviderTest, requestKeyForNewFilesystem) {
  constexpr size_t keySize = 512;
  constexpr const char* password = "mypassword";
  const EncryptionKey key = EncryptionKey::FromString(DataFixture::generate(keySize).ToString());
  const Data kdfParameters = DataFixture::generate(100);

  EXPECT_CALL(askPasswordForNewFilesystem, call()).Times(1).WillOnce(Return(password));
  EXPECT_CALL(askPasswordForExistingFilesystem, call()).Times(0);
  EXPECT_CALL(*kdf, deriveNewKey(Eq(keySize), StrEq(password))).Times(1).WillOnce(Invoke([&] (auto, auto) {return PasswordBasedKDF::KeyResult{key, kdfParameters.copy()};}));

  auto returned_key = keyProvider.requestKeyForNewFilesystem(keySize);

  EXPECT_EQ(key.ToString(), returned_key.key.ToString());
  EXPECT_EQ(kdfParameters, returned_key.kdfParameters);
}

TEST_F(CryPasswordBasedKeyProviderTest, requestKeyForExistingFilesystem) {
  constexpr size_t keySize = 512;
  constexpr const char* password = "mypassword";
  const EncryptionKey key = EncryptionKey::FromString(DataFixture::generate(keySize).ToString());
  const Data kdfParameters = DataFixture::generate(100);

  EXPECT_CALL(askPasswordForNewFilesystem, call()).Times(0);
  EXPECT_CALL(askPasswordForExistingFilesystem, call()).Times(1).WillOnce(Return(password));
  EXPECT_CALL(*kdf, deriveExistingKey(Eq(keySize), StrEq(password), _)).Times(1).WillOnce(Invoke([&] (auto, auto, const auto& kdfParams) {
    EXPECT_EQ(kdfParameters, kdfParams);
    return key;
  }));

  EncryptionKey returned_key = keyProvider.requestKeyForExistingFilesystem(keySize, kdfParameters);

  EXPECT_EQ(key.ToString(), returned_key.ToString());
}
