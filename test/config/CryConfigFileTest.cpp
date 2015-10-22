#include <google/gtest/gtest.h>

#include "../../src/config/CryConfigFile.h"
#include <messmer/cpp-utils/tempfile/TempFile.h>

using namespace cryfs;
using cpputils::TempFile;

class CryConfigFileTest: public ::testing::Test {
public:
    CryConfigFileTest(): file(false) {}

    TempFile file;

    CryConfigFile CreateAndLoadEmpty() {
        Create(CryConfig());
        return Load();
    }

    void Create(CryConfig cfg) {
        CryConfigFile::create(file.path(), std::move(cfg));
    }

    CryConfigFile Load() {
        return CryConfigFile::load(file.path()).value();
    }
};

TEST_F(CryConfigFileTest, RootBlob_Init) {
    CryConfigFile created = CreateAndLoadEmpty();
    EXPECT_EQ("", created.config()->RootBlob());
}

TEST_F(CryConfigFileTest, RootBlob_CreateAndLoad) {
    CryConfig cfg;
    cfg.SetRootBlob("rootblobid");
    Create(std::move(cfg));
    CryConfigFile loaded = Load();
    EXPECT_EQ("rootblobid", loaded.config()->RootBlob());
}

TEST_F(CryConfigFileTest, RootBlob_SaveAndLoad) {
    CryConfigFile created = CreateAndLoadEmpty();
    created.config()->SetRootBlob("rootblobid");
    created.save();
    CryConfigFile loaded = Load();
    EXPECT_EQ("rootblobid", loaded.config()->RootBlob());
}

TEST_F(CryConfigFileTest, EncryptionKey_Init) {
    CryConfigFile created = CreateAndLoadEmpty();
    EXPECT_EQ("", created.config()->EncryptionKey());
}

TEST_F(CryConfigFileTest, EncryptionKey_CreateAndLoad) {
    CryConfig cfg;
    cfg.SetEncryptionKey("encryptionkey");
    Create(std::move(cfg));
    CryConfigFile loaded = Load();
    EXPECT_EQ("encryptionkey", loaded.config()->EncryptionKey());
}

TEST_F(CryConfigFileTest, EncryptionKey_SaveAndLoad) {
    CryConfigFile created = CreateAndLoadEmpty();
    created.config()->SetEncryptionKey("encryptionkey");
    created.save();
    CryConfigFile loaded = Load();
    EXPECT_EQ("encryptionkey", loaded.config()->EncryptionKey());
}

TEST_F(CryConfigFileTest, Cipher_Init) {
    CryConfigFile created = CreateAndLoadEmpty();
    EXPECT_EQ("", created.config()->Cipher());
}

TEST_F(CryConfigFileTest, Cipher_CreateAndLoad) {
    CryConfig cfg;
    cfg.SetCipher("cipher");
    Create(std::move(cfg));
    CryConfigFile loaded = Load();
    EXPECT_EQ("cipher", loaded.config()->Cipher());
}

TEST_F(CryConfigFileTest, Cipher_SaveAndLoad) {
    CryConfigFile created = CreateAndLoadEmpty();
    created.config()->SetCipher("cipher");
    created.save();
    CryConfigFile loaded = Load();
    EXPECT_EQ("cipher", loaded.config()->Cipher());
}
