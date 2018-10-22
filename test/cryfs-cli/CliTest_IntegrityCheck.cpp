#include "testutils/CliTest.h"
#include <cryfs/config/CryConfigFile.h>
#include <cryfs/ErrorCodes.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include <cpp-utils/data/DataFixture.h>

using std::vector;
using std::string;
using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::ErrorCode;
using cryfs::CryKeyProvider;
using cpputils::Data;
using cpputils::EncryptionKey;
using cpputils::SCrypt;

class FakeCryKeyProvider final : public CryKeyProvider {
  EncryptionKey requestKeyForExistingFilesystem(size_t keySize, const Data& kdfParameters) override {
    return SCrypt(SCrypt::TestSettings).deriveExistingKey(keySize, "pass", kdfParameters);
  }

  KeyResult requestKeyForNewFilesystem(size_t keySize) override {
    auto derived = SCrypt(SCrypt::TestSettings).deriveNewKey(keySize, "pass");
    return {
      std::move(derived.key),
      std::move(derived.kdfParameters)
    };
  }
};

class CliTest_IntegrityCheck: public CliTest {
public:
  void modifyFilesystemId() {
    FakeCryKeyProvider keyProvider;
    auto configFile = CryConfigFile::load(basedir / "cryfs.config", &keyProvider).value();
    configFile.config()->SetFilesystemId(CryConfig::FilesystemID::FromString("0123456789ABCDEF0123456789ABCDEF"));
    configFile.save();
  }

  void modifyFilesystemKey() {
    FakeCryKeyProvider keyProvider;
    auto configFile = CryConfigFile::load(basedir / "cryfs.config", &keyProvider).value();
    configFile.config()->SetEncryptionKey("0123456789ABCDEF0123456789ABCDEF");
    configFile.save();
  }
};

TEST_F(CliTest_IntegrityCheck, givenIncorrectFilesystemId_thenFails) {
  vector<string> args {basedir.string().c_str(), mountdir.string().c_str(), "--cipher", "aes-256-gcm", "-f"};
  //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
  EXPECT_RUN_SUCCESS(args, mountdir);
  modifyFilesystemId();
  EXPECT_RUN_ERROR(
      args,
      "Error: The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir.",
      ErrorCode::FilesystemIdChanged
  );
}

TEST_F(CliTest_IntegrityCheck, givenIncorrectFilesystemKey_thenFails) {
  vector<string> args {basedir.string().c_str(), mountdir.string().c_str(), "--cipher", "aes-256-gcm", "-f"};
  //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
  EXPECT_RUN_SUCCESS(args, mountdir);
  modifyFilesystemKey();
  EXPECT_RUN_ERROR(
      args,
      "Error: The filesystem encryption key differs from the last time we loaded this filesystem. Did an attacker replace the file system?",
      ErrorCode::EncryptionKeyChanged
  );
}
