#include <gtest/gtest.h>
#include <cryfs/config/CryConfig.h>
#include <cpp-utils/data/DataFixture.h>

using namespace cryfs;
using cpputils::Data;
using cpputils::DataFixture;

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
    cfg.SetVersion("0.9.1");
    CryConfig moved = std::move(cfg);
    EXPECT_EQ("0.9.1", moved.Version());
}

TEST_F(CryConfigTest, Version_AfterSaveAndLoad) {
    cfg.SetVersion("0.9.2");
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ("0.9.2", loaded.Version());
}

TEST_F(CryConfigTest, CreatedWithVersion_Init) {
    EXPECT_EQ("", cfg.CreatedWithVersion());
}

TEST_F(CryConfigTest, CreatedWithVersion) {
    cfg.SetCreatedWithVersion("0.9.3");
    EXPECT_EQ("0.9.3", cfg.CreatedWithVersion());
}

TEST_F(CryConfigTest, CreatedWithVersion_AfterMove) {
    cfg.SetCreatedWithVersion("0.9.3");
    CryConfig moved = std::move(cfg);
    EXPECT_EQ("0.9.3", moved.CreatedWithVersion());
}

TEST_F(CryConfigTest, CreatedWithVersion_AfterSaveAndLoad) {
    cfg.SetCreatedWithVersion("0.9.3");
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ("0.9.3", loaded.CreatedWithVersion());
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

TEST_F(CryConfigTest, FilesystemID_Init) {
    EXPECT_EQ(CryConfig::FilesystemID::Null(), cfg.FilesystemId());
}

TEST_F(CryConfigTest, FilesystemID) {
    auto fixture = DataFixture::generateFixedSize<CryConfig::FilesystemID::BINARY_LENGTH>();
    cfg.SetFilesystemId(fixture);
    EXPECT_EQ(fixture, cfg.FilesystemId());
}

TEST_F(CryConfigTest, FilesystemID_AfterMove) {
    auto fixture = DataFixture::generateFixedSize<CryConfig::FilesystemID::BINARY_LENGTH>();
    cfg.SetFilesystemId(fixture);
    CryConfig moved = std::move(cfg);
    EXPECT_EQ(fixture, moved.FilesystemId());
}

TEST_F(CryConfigTest, FilesystemID_AfterSaveAndLoad) {
    auto fixture = DataFixture::generateFixedSize<CryConfig::FilesystemID::BINARY_LENGTH>();
    cfg.SetFilesystemId(fixture);
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ(fixture, loaded.FilesystemId());
}
