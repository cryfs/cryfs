#include <google/gtest/gtest.h>
#include "../../src/config/CryConfigLoader.h"
#include "../testutils/MockConsole.h"
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::TempFile;
using std::string;
using ::testing::Return;
using ::testing::_;

using namespace cryfs;

class CryConfigLoaderTest: public ::testing::Test, public TestWithMockConsole {
public:
    CryConfigLoaderTest(): loader(mockConsole(), cpputils::Random::PseudoRandom()), file(false) {}

    CryConfigFile Create() {
        EXPECT_FALSE(file.exists());
        return loader.loadOrCreate(file.path());
    }

    CryConfigFile Load() {
        EXPECT_TRUE(file.exists());
        return loader.loadOrCreate(file.path());
    }

    void CreateWithRootBlob(const string &rootBlob) {
        auto cfg = loader.loadOrCreate(file.path());
        cfg.config()->SetRootBlob(rootBlob);
        cfg.save();
    }

    void CreateWithCipher(const string &cipher) {
        auto cfg = loader.loadOrCreate(file.path());
        cfg.config()->SetCipher(cipher);
        cfg.save();
    }

    void CreateWithEncryptionKey(const string &encKey) {
        auto cfg = loader.loadOrCreate(file.path());
        cfg.config()->SetEncryptionKey(encKey);
        cfg.save();
    }

    CryConfigLoader loader;
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

TEST_F(CryConfigLoaderTest, RootBlob_Load) {
    CreateWithRootBlob("rootblobid");
    auto loaded = Load();
    EXPECT_EQ("rootblobid", loaded.config()->RootBlob());
}

TEST_F(CryConfigLoaderTest, RootBlob_Create) {
    auto created = Create();
    EXPECT_EQ("", created.config()->RootBlob());
}

TEST_F(CryConfigLoaderTest, EncryptionKey_Load) {
    CreateWithEncryptionKey("encryptionkey");
    auto loaded = Load();
    EXPECT_EQ("encryptionkey", loaded.config()->EncryptionKey());
}

TEST_F(CryConfigLoaderTest, EncryptionKey_Create) {
    auto created = Create();
    //aes-256-gcm is the default cipher chosen by mockConsole()
    blockstore::encrypted::AES256_GCM::EncryptionKey::FromString(created.config()->EncryptionKey()); // This crashes if key is invalid
}

TEST_F(CryConfigLoaderTest, Cipher_Load) {
    CreateWithCipher("ciphername");
    auto loaded = Load();
    EXPECT_EQ("ciphername", loaded.config()->Cipher());
}

TEST_F(CryConfigLoaderTest, Cipher_Create) {
    auto created = Create();
    //aes-256-gcm is the default cipher chosen by mockConsole()
    EXPECT_EQ("aes-256-gcm", created.config()->Cipher());
}
