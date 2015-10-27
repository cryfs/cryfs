#include <google/gtest/gtest.h>

#include "../../src/config/CryConfigFile.h"
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include <boost/optional/optional_io.hpp>
#include <messmer/cpp-utils/test/crypto/testutils/SCryptTestSettings.h>

using namespace cryfs;
using cpputils::TempFile;
using std::string;
using boost::optional;
using boost::none;
namespace bf = boost::filesystem;

//gtest/boost::optional workaround for working with optional<CryConfigFile>
namespace boost {
    inline std::ostream &operator<<(std::ostream &out, const CryConfigFile &file) {
        UNUSED(file);
        out << "ConfigFile";
        return out;
    }
}

class CryConfigFileTest: public ::testing::Test {
public:
    CryConfigFileTest(): file(false) {}

    TempFile file;

    CryConfigFile CreateAndLoadEmpty(const string &password = "mypassword") {
        Create(CryConfig(), password);
        return Load().value();
    }

    void Create(CryConfig cfg, const string &password = "mypassword") {
        CryConfigFile::create<SCryptTestSettings>(file.path(), std::move(cfg), password);
    }

    optional<CryConfigFile> Load(const string &password = "mypassword") {
        return CryConfigFile::load(file.path(), password);
    }

    void CreateWithCipher(const string &cipher, const TempFile &tempFile) {
        CryConfig cfg;
        cfg.SetCipher(cipher);
        CryConfigFile::create<SCryptTestSettings>(tempFile.path(), std::move(cfg), "mypassword");
    }
};

TEST_F(CryConfigFileTest, DoesntLoadIfWrongPassword) {
    Create(CryConfig(), "mypassword");
    auto loaded = Load("mypassword2");
    EXPECT_EQ(none, loaded);
}

TEST_F(CryConfigFileTest, RootBlob_Init) {
    CryConfigFile created = CreateAndLoadEmpty();
    EXPECT_EQ("", created.config()->RootBlob());
}

TEST_F(CryConfigFileTest, RootBlob_CreateAndLoad) {
    CryConfig cfg;
    cfg.SetRootBlob("rootblobid");
    Create(std::move(cfg));
    CryConfigFile loaded = Load().value();
    EXPECT_EQ("rootblobid", loaded.config()->RootBlob());
}

TEST_F(CryConfigFileTest, RootBlob_SaveAndLoad) {
    CryConfigFile created = CreateAndLoadEmpty();
    created.config()->SetRootBlob("rootblobid");
    created.save();
    CryConfigFile loaded = Load().value();
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
    CryConfigFile loaded = Load().value();
    EXPECT_EQ("encryptionkey", loaded.config()->EncryptionKey());
}

TEST_F(CryConfigFileTest, EncryptionKey_SaveAndLoad) {
    CryConfigFile created = CreateAndLoadEmpty();
    created.config()->SetEncryptionKey("encryptionkey");
    created.save();
    CryConfigFile loaded = Load().value();
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
    CryConfigFile loaded = Load().value();
    EXPECT_EQ("cipher", loaded.config()->Cipher());
}

TEST_F(CryConfigFileTest, Cipher_SaveAndLoad) {
    CryConfigFile created = CreateAndLoadEmpty();
    created.config()->SetCipher("cipher");
    created.save();
    CryConfigFile loaded = Load().value();
    EXPECT_EQ("cipher", loaded.config()->Cipher());
}

//Test that the encrypted config file has the same size, no matter how big the plaintext config data.
TEST_F(CryConfigFileTest, ConfigFileHasFixedSize) {
    TempFile file1(false);
    TempFile file2(false);
    CreateWithCipher("short", file1);
    CreateWithCipher("long_cipher_name_that_causes_the_plaintext_config_data_to_be_larger", file2);
    EXPECT_EQ(bf::file_size(file1.path()), bf::file_size(file2.path()));
}
