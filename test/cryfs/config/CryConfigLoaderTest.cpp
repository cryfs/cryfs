#include <gtest/gtest.h>
#include <cryfs/config/CryConfigLoader.h>
#include <cryfs/config/CryPresetPasswordBasedKeyProvider.h>
#include "../testutils/MockConsole.h"
#include "../testutils/TestWithFakeHomeDirectory.h"
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <gitversion/gitversion.h>
#include <gitversion/parser.h>
#include <gitversion/VersionCompare.h>

using cpputils::TempFile;
using cpputils::SCrypt;
using cpputils::DataFixture;
using cpputils::Data;
using cpputils::NoninteractiveConsole;
using cpputils::make_unique_ref;
using cpputils::Console;
using cpputils::unique_ref;
using cryfs::CryPresetPasswordBasedKeyProvider;
using boost::optional;
using boost::none;
using std::string;
using std::ostream;
using std::shared_ptr;
using std::make_shared;
using ::testing::Return;
using ::testing::HasSubstr;
using ::testing::_;

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
    unique_ref<CryKeyProvider> keyProvider(const string& password) {
      return make_unique_ref<CryPresetPasswordBasedKeyProvider>(password, make_unique_ref<SCrypt>(SCrypt::TestSettings));
    }

    CryConfigLoaderTest(): file(false), tempLocalStateDir(), localStateDir(tempLocalStateDir.path()) {
        console = mockConsole();
    }

    CryConfigLoader loader(const string &password, bool noninteractive, const optional<string> &cipher = none) {
        auto _console = noninteractive ? shared_ptr<Console>(make_shared<NoninteractiveConsole>(console)) : shared_ptr<Console>(console);
        return CryConfigLoader(_console, cpputils::Random::PseudoRandom(), keyProvider(password), localStateDir, cipher, none, none);
    }

    CryConfigFile Create(const string &password = "mypassword", const optional<string> &cipher = none, bool noninteractive = false) {
        EXPECT_FALSE(file.exists());
        return loader(password, noninteractive, cipher).loadOrCreate(file.path(), false, false).value().configFile;
    }

    optional<CryConfigFile> Load(const string &password = "mypassword", const optional<string> &cipher = none, bool noninteractive = false, bool allowFilesystemUpgrade = false) {
        EXPECT_TRUE(file.exists());
        auto loadResult = loader(password, noninteractive, cipher).loadOrCreate(file.path(), allowFilesystemUpgrade, false);
        if (loadResult == none) {
            return none;
        }
        return std::move(loadResult->configFile);
    }

    void CreateWithRootBlob(const string &rootBlob, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path(), false, false).value().configFile;
        cfg.config()->SetRootBlob(rootBlob);
        cfg.save();
    }

    void CreateWithCipher(const string &cipher, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path(), false, false).value().configFile;
        cfg.config()->SetCipher(cipher);
        cfg.save();
    }

    void CreateWithEncryptionKey(const string &encKey, const string &password = "mypassword") {
        FakeRandomGenerator generator(Data::FromString(encKey));
        auto loader = CryConfigLoader(console, generator, keyProvider(password), localStateDir, none, none, none);
        ASSERT_NE(boost::none, loader.loadOrCreate(file.path(), false, false));
    }

    void ChangeEncryptionKey(const string &encKey, const string& password = "mypassword") {
        auto cfg = CryConfigFile::load(file.path(), keyProvider(password).get()).value();
        cfg.config()->SetEncryptionKey(encKey);
        cfg.save();
    }

    void CreateWithVersion(const string &version, const string& formatVersion, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path(), false, false).value().configFile;
        cfg.config()->SetVersion(formatVersion);
        cfg.config()->SetLastOpenedWithVersion(version);
        cfg.config()->SetCreatedWithVersion(version);
        cfg.save();
    }
  
    void CreateWithFilesystemID(const CryConfig::FilesystemID &filesystemId, const string &password = "mypassword") {
        auto cfg = loader(password, false).loadOrCreate(file.path(), false, false).value().configFile;
        cfg.config()->SetFilesystemId(filesystemId);
        cfg.save();
    }

    void ChangeFilesystemID(const CryConfig::FilesystemID &filesystemId, const string& password = "mypassword") {
      auto cfg = CryConfigFile::load(file.path(), keyProvider(password).get()).value();
      cfg.config()->SetFilesystemId(filesystemId);
      cfg.save();
    }

    string olderVersion() {
        auto versionInfo = gitversion::Parser::parse(CryConfig::FilesystemFormatVersion);
        string olderVersion;
        if (std::stol(versionInfo.minorVersion) > 0) {
            olderVersion = versionInfo.majorVersion + "." + std::to_string(std::stol(versionInfo.minorVersion) - 1);
        } else {
            olderVersion = std::to_string(std::stol(versionInfo.majorVersion) - 1) + "." + versionInfo.minorVersion;
        }
        assert(gitversion::VersionCompare::isOlderThan(olderVersion, CryConfig::FilesystemFormatVersion));
        return olderVersion;
    }

    string newerVersion() {
        string newerVersion = gitversion::MajorVersion()+"."+std::to_string(std::stol(gitversion::MinorVersion())+2);
        EXPECT_TRUE(gitversion::VersionCompare::isOlderThan(CryConfig::FilesystemFormatVersion, newerVersion))
            << "Format Version " << CryConfig::FilesystemFormatVersion << " should be older than Git Version " << newerVersion;
        return newerVersion;
    }

    std::shared_ptr<MockConsole> console;
    TempFile file;
    cpputils::TempDir tempLocalStateDir;
    LocalStateDir localStateDir;
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
    CreateWithVersion("0.9.2", "0.9.2");
    auto loaded = Load().value();
    EXPECT_EQ(CryConfig::FilesystemFormatVersion, loaded.config()->Version());
    EXPECT_EQ(gitversion::VersionString(), loaded.config()->LastOpenedWithVersion());
    EXPECT_EQ("0.9.2", loaded.config()->CreatedWithVersion());
}

TEST_F(CryConfigLoaderTest, Version_Load_IsStoredAndNotOnlyOverwrittenInMemoryOnLoad) {
    CreateWithVersion("0.9.2", "0.9.2", "mypassword");
    Load().value();
    auto configFile = CryConfigFile::load(file.path(), keyProvider("mypassword").get()).value();
    EXPECT_EQ(CryConfig::FilesystemFormatVersion, configFile.config()->Version());
    EXPECT_EQ(gitversion::VersionString(), configFile.config()->LastOpenedWithVersion());
    EXPECT_EQ("0.9.2", configFile.config()->CreatedWithVersion());
}

TEST_F(CryConfigLoaderTest, Version_Create) {
    auto created = Create();
    EXPECT_EQ(CryConfig::FilesystemFormatVersion, created.config()->Version());
    EXPECT_EQ(gitversion::VersionString(), created.config()->LastOpenedWithVersion());
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
    CreateWithVersion(version, version);
    EXPECT_NE(boost::none, Load());
}

TEST_F(CryConfigLoaderTest, AsksWhenLoadingNewerFilesystem_AnswerNo) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("should not be opened with older versions"), false)).Times(1).WillOnce(Return(false));

    string version = newerVersion();
    CreateWithVersion(version, version);
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
    CreateWithVersion(version, version);
    EXPECT_NE(boost::none, Load());
}

TEST_F(CryConfigLoaderTest, DoesNotAskForMigrationWhenCorrectVersion) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("Do you want to migrate it?"), _)).Times(0);

    CreateWithVersion(gitversion::VersionString(), CryConfig::FilesystemFormatVersion);
    EXPECT_NE(boost::none, Load());
}

TEST_F(CryConfigLoaderTest, DontMigrateWhenAnsweredNo) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("Do you want to migrate it?"), false)).Times(1).WillOnce(Return(false));

    string version = olderVersion();
    CreateWithVersion(version, version);
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
    uint32_t myClientId = loader("mypassword", true).loadOrCreate(file1.path(), false, false).value().myClientId;
    EXPECT_NE(myClientId, loader("mypassword", true).loadOrCreate(file2.path(), false, false).value().myClientId);
}

TEST_F(CryConfigLoaderTest, MyClientIdIsLoadedCorrectly) {
    TempFile file(false);
    uint32_t myClientId = loader("mypassword", true).loadOrCreate(file.path(), false, false).value().myClientId;
    EXPECT_EQ(myClientId, loader("mypassword", true).loadOrCreate(file.path(), false, false).value().myClientId);
}

TEST_F(CryConfigLoaderTest, DoesNotAskForMigrationWhenUpgradesAllowedByProgramArguments_NoninteractiveMode) {
    EXPECT_CALL(*console, askYesNo(HasSubstr("migrate"), _)).Times(0);

    string version = olderVersion();
    CreateWithVersion(version, version);
    EXPECT_NE(boost::none, Load("mypassword", none, true, true));
}

TEST_F(CryConfigLoaderTest, DoesNotAskForMigrationWhenUpgradesAllowedByProgramArguments_InteractiveMode) {
  EXPECT_CALL(*console, askYesNo(HasSubstr("migrate"), _)).Times(0);

  string version = olderVersion();
  CreateWithVersion(version, version);
  EXPECT_NE(boost::none, Load("mypassword", none, false, true));
}
