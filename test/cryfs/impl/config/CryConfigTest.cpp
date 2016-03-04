#include <gtest/gtest.h>
#include <cryfs/impl/config/CryConfig.h>

using namespace cryfs;
using cpputils::Data;

class CryConfigTest: public ::testing::Test {
public:
    CryConfig cfg;

    CryConfig SaveAndLoad(CryConfig cfg) {
        Data configData = cfg.save();
        return CryConfig::load(configData);
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

TEST_F(CryConfigTest, Version_Init) {
    EXPECT_EQ("", cfg.Version());
}

TEST_F(CryConfigTest, Version) {
    cfg.SetVersion("0.9.1");
    EXPECT_EQ("0.9.1", cfg.Version());
}

TEST_F(CryConfigTest, Version_AfterMove) {
    cfg.SetCipher("0.9.1");
    CryConfig moved = std::move(cfg);
    EXPECT_EQ("0.9.1", moved.Cipher());
}

TEST_F(CryConfigTest, Version_AfterSaveAndLoad) {
    cfg.SetCipher("0.9.2");
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ("0.9.2", loaded.Cipher());
}

TEST_F(CryConfigTest, BlocksizeBytes_Init) {
    EXPECT_EQ(0u, cfg.BlocksizeBytes());
}

TEST_F(CryConfigTest, BlocksizeBytes) {
    cfg.SetBlocksizeBytes(4*1024*1024);
    EXPECT_EQ(4*1024*1024u, cfg.BlocksizeBytes());
}

TEST_F(CryConfigTest, BlocksizeBytes_AfterMove) {
    cfg.SetBlocksizeBytes(32*1024);
    CryConfig moved = std::move(cfg);
    EXPECT_EQ(32*1024u, moved.BlocksizeBytes());
}

TEST_F(CryConfigTest, BlocksizeBytes_AfterSaveAndLoad) {
    cfg.SetBlocksizeBytes(10*1024);
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ(10*1024u, loaded.BlocksizeBytes());
}
