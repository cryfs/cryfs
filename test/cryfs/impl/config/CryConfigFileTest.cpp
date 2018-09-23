#include <gtest/gtest.h>
#include <cryfs/impl/config/CryConfigFile.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using namespace cryfs;
using cpputils::TempFile;
using cpputils::unique_ref;
using std::string;
using boost::optional;
using boost::none;
using cpputils::SCrypt;
using cpputils::Data;
namespace bf = boost::filesystem;

//gtest/boost::optional workaround for working with optional<CryConfigFile>
namespace boost {
    inline std::ostream &operator<<(std::ostream &out, const CryConfigFile &file) {
        UNUSED(file);
        out << "ConfigFile()";
        return out;
    }
}
#include <boost/optional/optional_io.hpp>

class CryConfigFileTest: public ::testing::Test {
public:
    CryConfigFileTest(): file(false) {}

    TempFile file;

    CryConfig Config() {
        CryConfig result;
        result.SetCipher("aes-256-gcm");
        return result;
    }

    unique_ref<CryConfigFile> CreateAndLoadEmpty(const string &password = "mypassword") {
        Create(Config(), password);
        return Load().value();
    }

    void Create(CryConfig cfg, const string &password = "mypassword") {
        CryConfigFile::create(file.path(), std::move(cfg), password, SCrypt::TestSettings);
    }

    optional<unique_ref<CryConfigFile>> Load(const string &password = "mypassword") {
        return CryConfigFile::load(file.path(), password).right_opt();
    }

    void CreateWithCipher(const string &cipher) {
        return CreateWithCipher(cipher, file);
    }

    void CreateWithCipher(const string &cipher, const TempFile &tempFile) {
        CryConfig cfg;
        cfg.SetCipher(cipher);
        CryConfigFile::create(tempFile.path(), std::move(cfg), "mypassword", SCrypt::TestSettings);
    }
};

TEST_F(CryConfigFileTest, DoesntLoadIfWrongPassword) {
    Create(Config(), "mypassword");
    auto loaded = Load("mypassword2");
    EXPECT_EQ(none, loaded);
}

TEST_F(CryConfigFileTest, RootBlob_Init) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    EXPECT_EQ("", created->config()->RootBlob());
}

TEST_F(CryConfigFileTest, RootBlob_CreateAndLoad) {
    CryConfig cfg = Config();
    cfg.SetRootBlob("rootblobid");
    Create(std::move(cfg));
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("rootblobid", loaded->config()->RootBlob());
}

TEST_F(CryConfigFileTest, RootBlob_SaveAndLoad) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    created->config()->SetRootBlob("rootblobid");
    created->save();
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("rootblobid", loaded->config()->RootBlob());
}

TEST_F(CryConfigFileTest, EncryptionKey_Init) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    EXPECT_EQ("", created->config()->EncryptionKey());
}

TEST_F(CryConfigFileTest, EncryptionKey_CreateAndLoad) {
    CryConfig cfg = Config();
    cfg.SetEncryptionKey("encryptionkey");
    Create(std::move(cfg));
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("encryptionkey", loaded->config()->EncryptionKey());
}

TEST_F(CryConfigFileTest, EncryptionKey_SaveAndLoad) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    created->config()->SetEncryptionKey("encryptionkey");
    created->save();
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("encryptionkey", loaded->config()->EncryptionKey());
}

TEST_F(CryConfigFileTest, Cipher_Init) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    EXPECT_EQ("aes-256-gcm", created->config()->Cipher());
}

TEST_F(CryConfigFileTest, Cipher_CreateAndLoad) {
    CryConfig cfg = Config();
    cfg.SetCipher("twofish-128-cfb");
    Create(std::move(cfg));
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("twofish-128-cfb", loaded->config()->Cipher());
}

TEST_F(CryConfigFileTest, Cipher_SaveAndLoad) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    created->config()->SetCipher("twofish-128-cfb");
    created->save();
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("twofish-128-cfb", loaded->config()->Cipher());
}

TEST_F(CryConfigFileTest, Version_Init) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    EXPECT_EQ("", created->config()->Version());
}

TEST_F(CryConfigFileTest, Version_CreateAndLoad) {
    CryConfig cfg = Config();
    cfg.SetVersion("0.9.2");
    Create(std::move(cfg));
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("0.9.2", loaded->config()->Version());
}

TEST_F(CryConfigFileTest, Version_SaveAndLoad) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    created->config()->SetVersion("0.9.2");
    created->save();
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("0.9.2", loaded->config()->Version());
}

TEST_F(CryConfigFileTest, CreatedWithVersion_Init) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    EXPECT_EQ("", created->config()->Version());
}

TEST_F(CryConfigFileTest, CreatedWithVersion_CreateAndLoad) {
    CryConfig cfg = Config();
    cfg.SetCreatedWithVersion("0.9.2");
    Create(std::move(cfg));
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("0.9.2", loaded->config()->CreatedWithVersion());
}

TEST_F(CryConfigFileTest, CreatedWithVersion_SaveAndLoad) {
    unique_ref<CryConfigFile> created = CreateAndLoadEmpty();
    created->config()->SetCreatedWithVersion("0.9.2");
    created->save();
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("0.9.2", loaded->config()->CreatedWithVersion());
}

//Test that the encrypted config file has the same size, no matter how big the plaintext config data.
TEST_F(CryConfigFileTest, ConfigFileHasFixedSize) {
    TempFile file1(false);
    TempFile file2(false);
    //It is important to have different cipher name lengths here, because they're on the outer encryption level.
    //So this ensures that there also is a padding happening on the outer encryption level.
    CreateWithCipher("aes-128-gcm", file1); // Short cipher name and short key
    CreateWithCipher("twofish-256-cfb", file2); // Long cipher name and long key
    EXPECT_EQ(bf::file_size(file1.path()), bf::file_size(file2.path()));
}

TEST_F(CryConfigFileTest, CanSaveAndLoadModififedCipher) {
    CreateWithCipher("aes-256-gcm");
    unique_ref<CryConfigFile> created = Load().value();
    EXPECT_EQ("aes-256-gcm", created->config()->Cipher());
    created->config()->SetCipher("twofish-128-cfb");
    created->save();
    unique_ref<CryConfigFile> loaded = std::move(Load().value());
    EXPECT_EQ("twofish-128-cfb", loaded->config()->Cipher());
}

TEST_F(CryConfigFileTest, FailsIfConfigFileIsEncryptedWithACipherDifferentToTheOneSpecifiedByTheUser) {
    auto encryptor = CryConfigEncryptorFactory::deriveKey("mypassword", SCrypt::TestSettings);
    auto config = Config();
    config.SetCipher("aes-256-gcm");
    Data encrypted = encryptor->encrypt(config.save(), "aes-256-cfb");
    encrypted.StoreToFile(file.path());
    auto loaded = Load("mypassword");
    EXPECT_EQ(none, loaded);
}
