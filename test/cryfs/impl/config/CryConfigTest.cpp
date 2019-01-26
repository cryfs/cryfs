#include <gtest/gtest.h>
#include <cryfs/impl/config/CryConfig.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using namespace cryfs;
using cpputils::Data;
using cpputils::DataFixture;
using boost::none;

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

TEST_F(CryConfigTest, RootBlob_AfterCopy) {
    cfg.SetRootBlob("rootblobid");
    CryConfig copy = cfg;
    EXPECT_EQ("rootblobid", copy.RootBlob());
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

TEST_F(CryConfigTest, EncryptionKey_AfterCopy) {
    cfg.SetEncryptionKey("enckey");
    CryConfig copy = cfg;
    EXPECT_EQ("enckey", copy.EncryptionKey());
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

TEST_F(CryConfigTest, Cipher_AfterCopy) {
    cfg.SetCipher("mycipher");
    CryConfig copy = cfg;
    EXPECT_EQ("mycipher", copy.Cipher());
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

TEST_F(CryConfigTest, Version_AfterCopy) {
    cfg.SetVersion("0.9.1");
    CryConfig copy = cfg;
    EXPECT_EQ("0.9.1", copy.Version());
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

TEST_F(CryConfigTest, CreatedWithVersion_AfterCopy) {
    cfg.SetCreatedWithVersion("0.9.3");
    CryConfig copy = cfg;
    EXPECT_EQ("0.9.3", copy.CreatedWithVersion());
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

TEST_F(CryConfigTest, BlocksizeBytes_AfterCopy) {
    cfg.SetBlocksizeBytes(32*1024);
    CryConfig copy = cfg;
    EXPECT_EQ(32*1024u, copy.BlocksizeBytes());
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

TEST_F(CryConfigTest, ExclusiveClientId_Init) {
    EXPECT_EQ(none, cfg.ExclusiveClientId());
}

TEST_F(CryConfigTest, ExclusiveClientId_Some) {
    cfg.SetExclusiveClientId(0x12345678u);
    EXPECT_EQ(0x12345678u, cfg.ExclusiveClientId().value());
}

TEST_F(CryConfigTest, ExclusiveClientId_None) {
    cfg.SetExclusiveClientId(0x12345678u);
    cfg.SetExclusiveClientId(none);
    EXPECT_EQ(none, cfg.ExclusiveClientId());
}

TEST_F(CryConfigTest, ExclusiveClientId_Some_AfterMove) {
    cfg.SetExclusiveClientId(0x12345678u);
    CryConfig moved = std::move(cfg);
    EXPECT_EQ(0x12345678u, moved.ExclusiveClientId().value());
}

TEST_F(CryConfigTest, ExclusiveClientId_None_AfterMove) {
    cfg.SetExclusiveClientId(0x12345678u);
    cfg.SetExclusiveClientId(none);
    CryConfig moved = std::move(cfg);
    EXPECT_EQ(none, moved.ExclusiveClientId());
}

TEST_F(CryConfigTest, ExclusiveClientId_Some_AfterSaveAndLoad) {
    cfg.SetExclusiveClientId(0x12345678u);
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ(0x12345678u, loaded.ExclusiveClientId().value());
}

TEST_F(CryConfigTest, ExclusiveClientId_None_AfterSaveAndLoad) {
    cfg.SetExclusiveClientId(none);
    CryConfig loaded = SaveAndLoad(std::move(cfg));
    EXPECT_EQ(none, loaded.ExclusiveClientId());
}
