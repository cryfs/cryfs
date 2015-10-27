#include <google/gtest/gtest.h>
#include "../../src/config/CryConfigLoader.h"
#include "../testutils/MockConsole.h"
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>
#include <messmer/cpp-utils/test/crypto/testutils/SCryptTestSettings.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::TempFile;
using boost::optional;
using boost::none;
using std::string;
using ::testing::Return;
using ::testing::_;

using namespace cryfs;

class CryConfigLoaderTest: public ::testing::Test, public TestWithMockConsole {
public:
    CryConfigLoaderTest(): file(false) {}

    CryConfigLoader loader(const string &password) {
        return CryConfigLoader(mockConsole(), cpputils::Random::PseudoRandom(), [password] {return password;});
    }

    CryConfigFile Create(const string &password = "mypassword") {
        EXPECT_FALSE(file.exists());
        return loader(password).loadOrCreate<SCryptTestSettings>(file.path()).value();
    }

    optional<CryConfigFile> Load(const string &password = "mypassword") {
        EXPECT_TRUE(file.exists());
        return loader(password).loadOrCreate<SCryptTestSettings>(file.path());
    }

    void CreateWithRootBlob(const string &rootBlob, const string &password = "mypassword") {
        auto cfg = loader(password).loadOrCreate<SCryptTestSettings>(file.path()).value();
        cfg.config()->SetRootBlob(rootBlob);
        cfg.save();
    }

    void CreateWithCipher(const string &cipher, const string &password = "mypassword") {
        auto cfg = loader(password).loadOrCreate<SCryptTestSettings>(file.path()).value();
        cfg.config()->SetCipher(cipher);
        cfg.save();
    }

    void CreateWithEncryptionKey(const string &encKey, const string &password = "mypassword") {
        auto cfg = loader(password).loadOrCreate<SCryptTestSettings>(file.path()).value();
        cfg.config()->SetEncryptionKey(encKey);
        cfg.save();
    }

    TempFile file;
};

TEST_F(CryConfigLoaderTest, CreatesNewIfNotExisting) {
    EXPECT_FALSE(file.exists());
    Create();
    EXPECT_TRUE(file.exists());
}

TEST_F(CryConfigLoaderTest, DoesntCrashIfExisting) {
    Create();
    Load();
}

TEST_F(CryConfigLoaderTest, DoesntLoadIfWrongPassword) {
    Create("mypassword");
    auto loaded = Load("mypassword2");
    EXPECT_EQ(none, loaded);
}

TEST_F(CryConfigLoaderTest, RootBlob_Load) {
    CreateWithRootBlob("rootblobid");
    auto loaded = Load().value();
    EXPECT_EQ("rootblobid", loaded.config()->RootBlob());
}

TEST_F(CryConfigLoaderTest, RootBlob_Create) {
    auto created = Create();
    EXPECT_EQ("", created.config()->RootBlob());
}

TEST_F(CryConfigLoaderTest, EncryptionKey_Load) {
    CreateWithEncryptionKey("encryptionkey");
    auto loaded = Load().value();
    EXPECT_EQ("encryptionkey", loaded.config()->EncryptionKey());
}

TEST_F(CryConfigLoaderTest, EncryptionKey_Create) {
    auto created = Create();
    //aes-256-gcm is the default cipher chosen by mockConsole()
    blockstore::encrypted::AES256_GCM::EncryptionKey::FromString(created.config()->EncryptionKey()); // This crashes if key is invalid
}

TEST_F(CryConfigLoaderTest, Cipher_Load) {
    CreateWithCipher("ciphername");
    auto loaded = Load().value();
    EXPECT_EQ("ciphername", loaded.config()->Cipher());
}

TEST_F(CryConfigLoaderTest, Cipher_Create) {
    auto created = Create();
    //aes-256-gcm is the default cipher chosen by mockConsole()
    EXPECT_EQ("aes-256-gcm", created.config()->Cipher());
}
