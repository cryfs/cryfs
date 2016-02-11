#include <gtest/gtest.h>
#include <cryfs/config/CryConfigLoader.h>
#include "../testutils/MockConsole.h"
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <boost/optional/optional_io.hpp>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::TempFile;
using cpputils::SCrypt;
using boost::optional;
using boost::none;
using std::string;
using std::ostream;
using ::testing::Return;
using ::testing::_;

using namespace cryfs;

// This is needed for google test
namespace boost {
    inline ostream &operator<<(ostream &stream, const CryConfigFile &) {
        return stream << "CryConfigFile()";
    }
}

//TODO Test loading with same/different --cipher argument

class CryConfigLoaderTest: public ::testing::Test, public TestWithMockConsole {
public:
    CryConfigLoaderTest(): file(false) {}

    CryConfigLoader loader(const string &password, const optional<string> &cipher = none) {
        auto askPassword = [password] {return password;};
        return CryConfigLoader(mockConsole(), cpputils::Random::PseudoRandom(), SCrypt::TestSettings, askPassword, askPassword, cipher);
    }

    CryConfigFile Create(const string &password = "mypassword", const optional<string> &cipher = none) {
        EXPECT_FALSE(file.exists());
        return loader(password, cipher).loadOrCreate(file.path()).value();
    }

    optional<CryConfigFile> Load(const string &password = "mypassword", const optional<string> &cipher = none) {
        EXPECT_TRUE(file.exists());
        return loader(password, cipher).loadOrCreate(file.path());
    }

    void CreateWithRootBlob(const string &rootBlob, const string &password = "mypassword") {
        auto cfg = loader(password).loadOrCreate(file.path()).value();
        cfg.config()->SetRootBlob(rootBlob);
        cfg.save();
    }

    void CreateWithCipher(const string &cipher, const string &password = "mypassword") {
        auto cfg = loader(password).loadOrCreate(file.path()).value();
        cfg.config()->SetCipher(cipher);
        cfg.save();
    }

    void CreateWithEncryptionKey(const string &encKey, const string &password = "mypassword") {
        auto cfg = loader(password).loadOrCreate(file.path()).value();
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

TEST_F(CryConfigLoaderTest, DoesntLoadIfDifferentCipher) {
    Create("mypassword", string("aes-256-gcm"));
    try {
        Load("mypassword", string("aes-256-cfb"));
        EXPECT_TRUE(false); // Should throw exception
    } catch (const std::runtime_error &e) {
        EXPECT_EQ(string("Filesystem uses aes-256-gcm cipher and not aes-256-cfb as specified."), e.what());
    }
}

TEST_F(CryConfigLoaderTest, DoesLoadIfSameCipher) {
    Create("mypassword", string("aes-256-gcm"));
    Load("mypassword", string("aes-256-gcm"));
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
    cpputils::AES256_GCM::EncryptionKey::FromString(created.config()->EncryptionKey()); // This crashes if key is invalid
}

TEST_F(CryConfigLoaderTest, Cipher_Load) {
    CreateWithCipher("twofish-128-cfb");
    auto loaded = Load().value();
    EXPECT_EQ("twofish-128-cfb", loaded.config()->Cipher());
}

TEST_F(CryConfigLoaderTest, Cipher_Create) {
    auto created = Create();
    //aes-256-gcm is the default cipher chosen by mockConsole()
    EXPECT_EQ("aes-256-gcm", created.config()->Cipher());
}
