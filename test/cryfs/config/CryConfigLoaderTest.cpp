#include <gtest/gtest.h>
#include <cryfs/config/CryConfigLoader.h>
#include "../testutils/MockConsole.h"
#include "../testutils/TestWithFakeHomeDirectory.h"
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <gitversion/gitversion.h>
#include <gitversion/VersionCompare.h>

using cpputils::TempFile;
using cpputils::SCrypt;
using cpputils::DataFixture;
using cpputils::Data;
using cpputils::NoninteractiveConsole;
using boost::optional;
using boost::none;
using std::string;
using std::ostream;
using std::make_shared;
using ::testing::Return;
using ::testing::HasSubstr;

using namespace cryfs;

// This is needed for google test
namespace boost {
    inline ostream &operator<<(ostream &stream, const CryConfigFile &) {
        return stream << "CryConfigFile()";
    }
}
namespace cryfs {
  inline ostream &operator<<(ostream &stream, const CryConfigLoader::ConfigLoadResult &) {
    return stream << "ConfigLoadResult()";
  }
}
#include <boost/optional/optional_io.hpp>

class FakeRandomGenerator final : public cpputils::RandomGenerator {
public:
  FakeRandomGenerator(Data output)
      : _output(std::move(output)) {}

  void _get(void *target, size_t bytes) override {
    ASSERT_EQ(_output.size(), bytes);
    std::memcpy(target, _output.data(), bytes);
  }

private:
  Data _output;
};

class CryConfigLoaderTest: public ::testing::Test, public TestWithMockConsole, TestWithFakeHomeDirectory {
public:
    CryConfigLoaderTest(): file(false) {
        console = mockConsole();
    }

    CryConfigLoader loader(const string &password, bool noninteractive, const optional<string> &cipher = none) {
        auto askPassword = [password] { return password;};
        if(noninteractive) {
            return CryConfigLoader(make_shared<NoninteractiveConsole>(console), cpputils::Random::PseudoRandom(), SCrypt::TestSettings, askPassword,
                                   askPassword, cipher, none, none);
        } else {
            return CryConfigLoader(console, cpputils::Random::PseudoRandom(), SCrypt::TestSettings, askPassword,
                                   askPassword, cipher, none, none);
        }
    }

    CryConfigFile Create(const string &password = "mypassword", const optional<string> &cipher = none, bool noninteractive = false) {
        EXPECT_FALSE(file.exists());
        return loader(password, noninteractive, cipher).loadOrCreate(file.path()).value().configFile;
    }

    optional<CryConfigFile> Load(const string &password = "mypassword", const optional<string> &cipher = none, bool noninteractive = false) {
        EXPECT_TRUE(file.exists());
        auto loadResult = loader(password, noninteractive, cipher).loadOrCreate(file.path());
        if (loadResult == none) {
            return none;
        }
        return std::move(loadResult->configFile);
    }

    void CreateWithRootBlob(const string &rootBlob, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path()).value().configFile;
        cfg.config()->SetRootBlob(rootBlob);
        cfg.save();
    }

    void CreateWithCipher(const string &cipher, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path()).value().configFile;
        cfg.config()->SetCipher(cipher);
        cfg.save();
    }

    void CreateWithEncryptionKey(const string &encKey, const string &password = "mypassword") {
        auto askPassword = [password] { return password;};
        FakeRandomGenerator generator(Data::FromString(encKey));
        auto loader = CryConfigLoader(console, generator, SCrypt::TestSettings, askPassword,
                                      askPassword, none, none, none);
        ASSERT_NE(boost::none, loader.loadOrCreate(file.path()));
    }

    void ChangeEncryptionKey(const string &encKey, const string& password = "mypassword") {
        auto cfg = CryConfigFile::load(file.path(), password).value();
        cfg.config()->SetEncryptionKey(encKey);
        cfg.save();
    }

    void CreateWithVersion(const string &version, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path()).value().configFile;
        cfg.config()->SetVersion(version);
        cfg.config()->SetCreatedWithVersion(version);
        cfg.save();
    }

    void CreateWithFilesystemID(const CryConfig::FilesystemID &filesystemId, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path()).value().configFile;
        cfg.config()->SetFilesystemId(filesystemId);
        cfg.save();
    }

    void ChangeFilesystemID(const CryConfig::FilesystemID &filesystemId, const string& password = "mypassword") {
      auto cfg = CryConfigFile::load(file.path(), password).value();
      cfg.config()->SetFilesystemId(filesystemId);
      cfg.save();
    }

    string olderVersion() {
        string olderVersion;
        if (std::stol(gitversion::MinorVersion()) > 0) {
            olderVersion = gitversion::MajorVersion() + "." + std::to_string(std::stol(gitversion::MinorVersion()) - 1);
        } else {
            olderVersion = std::to_string(std::stol(gitversion::MajorVersion()) - 1) + "." + gitversion::MinorVersion();
        }
        assert(gitversion::VersionCompare::isOlderThan(olderVersion, gitversion::VersionString()));
        return olderVersion;
    }

    string newerVersion() {
        string newerVersion = gitversion::MajorVersion()+"."+std::to_string(std::stol(gitversion::MinorVersion())+1);
        assert(gitversion::VersionCompare::isOlderThan(gitversion::VersionString(), newerVersion));
        return newerVersion;
    }

    std::shared_ptr<MockConsole> console;
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
    Create("mypassword", string("aes-256-gcm"), false);
    try {
        Load("mypassword", string("aes-256-cfb"), false);
        EXPECT_TRUE(false); // Should throw exception
    } catch (const std::runtime_error &e) {
        EXPECT_EQ(string("Filesystem uses aes-256-gcm cipher and not aes-256-cfb as specified."), e.what());
    }
}

TEST_F(CryConfigLoaderTest, DoesntLoadIfDifferentCipher_Noninteractive) {
    Create("mypassword", string("aes-256-gcm"), true);
    try {
        Load("mypassword", string("aes-256-cfb"), true);
        EXPECT_TRUE(false); // Should throw exception
    } catch (const std::runtime_error &e) {
        EXPECT_EQ(string("Filesystem uses aes-256-gcm cipher and not aes-256-cfb as specified."), e.what());
    }
}

TEST_F(CryConfigLoaderTest, DoesLoadIfSameCipher) {
    Create("mypassword", string("aes-256-gcm"));
    Load("mypassword", string("aes-256-gcm"));
}

TEST_F(CryConfigLoaderTest, DoesLoadIfSameCipher_Noninteractive) {
    Create("mypassword", string("aes-128-gcm"), true);
    Load("mypassword", string("aes-128-gcm"), true);
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
    CreateWithEncryptionKey("3B4682CF22F3CA199E385729B9F3CA19D325229E385729B9443CA19D325229E3");
    auto loaded = Load().value();
    EXPECT_EQ("3B4682CF22F3CA199E385729B9F3CA19D325229E385729B9443CA19D325229E3", loaded.config()->EncryptionKey());
}

TEST_F(CryConfigLoaderTest, EncryptionKey_Load_whenKeyChanged_thenFails) {
  CreateWithEncryptionKey("3B4682CF22F3CA199E385729B9F3CA19D325229E385729B9443CA19D325229E3");
  ChangeEncryptionKey("3B4682CF22F3CA199E385729B9F3CA19D325229E385729B9443CA19D325229E4");
  EXPECT_THROW(
      Load(),
      std::runtime_error
  );
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

TEST_F(CryConfigLoaderTest, Version_Load) {
    CreateWithVersion("0.9.2");
    auto loaded = Load().value();
    EXPECT_EQ(gitversion::VersionString(), loaded.config()->Version());
    EXPECT_EQ("0.9.2", loaded.config()->CreatedWithVersion());
}

TEST_F(CryConfigLoaderTest, Version_Load_IsStoredAndNotOnlyOverwrittenInMemoryOnLoad) {
    CreateWithVersion("0.9.2", "mypassword");
    Load().value();
    auto configFile = CryConfigFile::load(file.path(), "mypassword").value();
    EXPECT_EQ(gitversion::VersionString(), configFile.config()->Version());
    EXPECT_EQ("0.9.2", configFile.config()->CreatedWithVersion());
}

TEST_F(CryConfigLoaderTest, Version_Create) {
    auto created = Create();
    EXPECT_EQ(gitversion::VersionString(), created.config()->Version());
    EXPECT_EQ(gitversion::VersionString(), created.config()->CreatedWithVersion());
}

TEST_F(CryConfigLoaderTest, FilesystemID_Load) {
    auto fixture = DataFixture::generateFixedSize<CryConfig::FilesystemID::BINARY_LENGTH>();
    CreateWithFilesystemID(fixture);
    auto loaded = Load().value();
    EXPECT_EQ(fixture, loaded.config()->FilesystemId());
}

TEST_F(CryConfigLoaderTest, FilesystemID_Create) {
    auto created = Create();
    EXPECT_NE(CryConfig::FilesystemID::Null(), created.config()->FilesystemId());
}

TEST_F(CryConfigLoaderTest, AsksWhenLoadingNewerFilesystem_AnswerYes) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("should not be opened with older versions"), false)).Times(1).WillOnce(Return(true));

    string version = newerVersion();
    CreateWithVersion(version);
    EXPECT_NE(boost::none, Load());
}

TEST_F(CryConfigLoaderTest, AsksWhenLoadingNewerFilesystem_AnswerNo) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("should not be opened with older versions"), false)).Times(1).WillOnce(Return(false));

    string version = newerVersion();
    CreateWithVersion(version);
    try {
        Load();
        EXPECT_TRUE(false); // expect throw
    } catch (const std::runtime_error &e) {
        EXPECT_THAT(e.what(), HasSubstr("Please update your CryFS version."));
    }
}

TEST_F(CryConfigLoaderTest, AsksWhenMigratingOlderFilesystem) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("Do you want to migrate it?"), false)).Times(1).WillOnce(Return(true));

    string version = olderVersion();
    CreateWithVersion(version);
    EXPECT_NE(boost::none, Load());
}

TEST_F(CryConfigLoaderTest, DoesNotAskForMigrationWhenCorrectVersion) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("Do you want to migrate it?"), false)).Times(0);

    CreateWithVersion(gitversion::VersionString());
    EXPECT_NE(boost::none, Load());
}

TEST_F(CryConfigLoaderTest, DontMigrateWhenAnsweredNo) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("Do you want to migrate it?"), false)).Times(1).WillOnce(Return(false));

    string version = olderVersion();
    CreateWithVersion(version);
    try {
        Load();
        EXPECT_TRUE(false); // expect throw
    } catch (const std::runtime_error &e) {
        EXPECT_THAT(e.what(), HasSubstr("It has to be migrated."));
    }
}

TEST_F(CryConfigLoaderTest, MyClientIdIsIndeterministic) {
    TempFile file1(false);
    TempFile file2(false);
    uint32_t myClientId = loader("mypassword", true).loadOrCreate(file1.path()).value().myClientId;
    EXPECT_NE(myClientId, loader("mypassword", true).loadOrCreate(file2.path()).value().myClientId);
}

TEST_F(CryConfigLoaderTest, MyClientIdIsLoadedCorrectly) {
    TempFile file(false);
    uint32_t myClientId = loader("mypassword", true).loadOrCreate(file.path()).value().myClientId;
    EXPECT_EQ(myClientId, loader("mypassword", true).loadOrCreate(file.path()).value().myClientId);
}
