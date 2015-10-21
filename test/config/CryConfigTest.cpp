#include <google/gtest/gtest.h>

#include "../../src/config/CryConfig.h"

using namespace cryfs;

class CryConfigTest: public ::testing::Test {
public:
    CryConfig cfg;

    CryConfig SaveAndLoad(CryConfig cfg) {
        std::stringstream stream;
        cfg.save(stream);
        CryConfig loaded;
        loaded.load(stream);
        return loaded;
    }
};

TEST_F(CryConfigTest, RootBlob_Init) {
    EXPECT_EQ("", cfg.RootBlob());
}

TEST_F(CryConfigTest, RootBlob) {
    cfg.SetRootBlob("rootblobid");
    EXPECT_EQ("rootblobid", cfg.RootBlob());
}

TEST_F(CryConfigTest, RootBlob_AfterMove) {
    cfg.SetRootBlob("rootblobid");
    CryConfig moved = std::move(cfg);
    EXPECT_EQ("rootblobid", moved.RootBlob());
}

TEST_F(CryConfigTest, RootBlob_AfterSaveAndLoad) {
    cfg.SetRootBlob("rootblobid");
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ("rootblobid", loaded.RootBlob());
}

TEST_F(CryConfigTest, EncryptionKey_Init) {
    EXPECT_EQ("", cfg.EncryptionKey());
}

TEST_F(CryConfigTest, EncryptionKey) {
    cfg.SetEncryptionKey("enckey");
    EXPECT_EQ("enckey", cfg.EncryptionKey());
}

TEST_F(CryConfigTest, EncryptionKey_AfterMove) {
    cfg.SetEncryptionKey("enckey");
    CryConfig moved = std::move(cfg);
    EXPECT_EQ("enckey", moved.EncryptionKey());
}

TEST_F(CryConfigTest, EncryptionKey_AfterSaveAndLoad) {
    cfg.SetEncryptionKey("enckey");
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ("enckey", loaded.EncryptionKey());
}

TEST_F(CryConfigTest, Cipher_Init) {
    EXPECT_EQ("", cfg.Cipher());
}

TEST_F(CryConfigTest, Cipher) {
    cfg.SetCipher("mycipher");
    EXPECT_EQ("mycipher", cfg.Cipher());
}

TEST_F(CryConfigTest, Cipher_AfterMove) {
    cfg.SetCipher("mycipher");
    CryConfig moved = std::move(cfg);
    EXPECT_EQ("mycipher", moved.Cipher());
}

TEST_F(CryConfigTest, Cipher_AfterSaveAndLoad) {
    cfg.SetCipher("mycipher");
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ("mycipher", loaded.Cipher());
}
