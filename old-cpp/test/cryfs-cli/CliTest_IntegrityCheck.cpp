#include "testutils/CliTest.h"
#include <cryfs/impl/config/CryConfigFile.h>
#include <cryfs/impl/ErrorCodes.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cryfs/impl/filesystem/cachingfsblobstore/CachingFsBlobStore.h>

using std::vector;
using std::string;
using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::ErrorCode;
using cryfs::CryKeyProvider;
using cpputils::Data;
using cpputils::EncryptionKey;
using cpputils::SCrypt;
using cpputils::TempDir;
namespace bf = boost::filesystem;

namespace {

void writeFile(const bf::path& filename, const string& content) {
  std::ofstream file(filename.c_str(), std::ios::trunc);
  file << content;
  ASSERT(file.good(), "Failed writing file to file system");
}

bool readingFileIsSuccessful(const bf::path& filename) {
  std::ifstream file(filename.c_str());
  std::string content;
  file >> content; // just read a little bit so we have a file access
  return file.good();
}

// NOLINTNEXTLINE(misc-no-recursion)
void recursive_copy(const bf::path &src, const bf::path &dst) {
  if (bf::exists(dst)) {
    throw std::runtime_error(dst.generic_string() + " already exists");
  }

  if (bf::is_directory(src)) {
    bf::create_directories(dst);
    for (auto& item : bf::directory_iterator(src)) {
      recursive_copy(item.path(), dst / item.path().filename());
    }
  } else if (bf::is_regular_file(src)) {
    bf::copy_file(src, dst);
  } else {
    throw std::runtime_error(dst.generic_string() + " neither dir nor file");
  }
}

class FakeCryKeyProvider final : public CryKeyProvider {
  EncryptionKey requestKeyForExistingFilesystem(size_t keySize, const Data &kdfParameters) override {
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

class CliTest_IntegrityCheck : public CliTest {
public:
  void modifyFilesystemId() {
    FakeCryKeyProvider keyProvider;
    auto configFile = CryConfigFile::load(basedir / "cryfs.config", &keyProvider, CryConfigFile::Access::ReadWrite).right_opt().value();
    configFile->config()->SetFilesystemId(CryConfig::FilesystemID::FromString("0123456789ABCDEF0123456789ABCDEF"));
    configFile->save();
  }

  void modifyFilesystemKey() {
    FakeCryKeyProvider keyProvider;
    auto configFile = CryConfigFile::load(basedir / "cryfs.config", &keyProvider, CryConfigFile::Access::ReadWrite).right_opt().value();
    configFile->config()->SetEncryptionKey("0123456789ABCDEF0123456789ABCDEF");
    configFile->save();
  }
};

TEST_F(CliTest_IntegrityCheck, givenIncorrectFilesystemId_thenFails) {
  vector<string> args{basedir.string().c_str(), mountdir.string().c_str(), "--cipher", "aes-256-gcm", "-f"};
  //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
  EXPECT_RUN_SUCCESS(args, mountdir);
  modifyFilesystemId();
  EXPECT_RUN_ERROR(
      args,
      "Error 20: The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir.",
      ErrorCode::FilesystemIdChanged
  );
}

TEST_F(CliTest_IntegrityCheck, givenIncorrectFilesystemKey_thenFails) {
  vector<string> args{basedir.string().c_str(), mountdir.string().c_str(), "--cipher", "aes-256-gcm", "-f"};
  //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS can handle that
  EXPECT_RUN_SUCCESS(args, mountdir);
  modifyFilesystemKey();
  EXPECT_RUN_ERROR(
      args,
      "Error 21: The filesystem encryption key differs from the last time we loaded this filesystem. Did an attacker replace the file system?",
      ErrorCode::EncryptionKeyChanged
  );
}

// TODO Also enable this
TEST_F(CliTest_IntegrityCheck, givenFilesystemWithRolledBackBasedir_whenMounting_thenFails) {
  vector<string> args{basedir.string().c_str(), mountdir.string().c_str(), "--cipher", "aes-256-gcm", "-f"};
  //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS/EXPECT_RUN_ERROR can handle that

  // create a filesystem with one file
  EXPECT_RUN_SUCCESS(args, mountdir, [&] {
    writeFile(mountdir / "myfile", "hello world");
  });

  // backup the base directory
  TempDir backup;
  recursive_copy(basedir, backup.path() / "basedir");

  // modify the file system contents
  EXPECT_RUN_SUCCESS(args, mountdir, [&] {
    writeFile(mountdir / "myfile", "hello world 2");
  });

  // roll back base directory
  bf::remove_all(basedir);
  recursive_copy(backup.path() / "basedir", basedir);

  // error code is success because it unmounts normally
  EXPECT_RUN_ERROR(args, "Integrity violation detected. Unmounting.", ErrorCode::IntegrityViolation, [&] {
    EXPECT_FALSE(readingFileIsSuccessful(mountdir / "myfile"));
  });

  // Test it doesn't mount anymore now because it's marked with an integrity violation
  EXPECT_RUN_ERROR(args, "There was an integrity violation detected. Preventing any further access to the file system.", ErrorCode::IntegrityViolationOnPreviousRun);
}

TEST_F(CliTest_IntegrityCheck, whenRollingBackBasedirWhileMounted_thenUnmounts) {
  vector<string> args{basedir.string().c_str(), mountdir.string().c_str(), "--cipher", "aes-256-gcm", "-f"};
  //TODO Remove "-f" parameter, once EXPECT_RUN_SUCCESS/EXPECT_RUN_ERROR can handle that

  // create a filesystem with one file
  EXPECT_RUN_SUCCESS(args, mountdir, [&] {
    writeFile(mountdir / "myfile", "hello world");
  });

  // backup the base directory
  TempDir backup;
  recursive_copy(basedir, backup.path() / "basedir");

  EXPECT_RUN_ERROR(args, "Integrity violation detected. Unmounting.", ErrorCode::IntegrityViolation, [&] {
    // modify the file system contents
    writeFile(mountdir / "myfile", "hello world 2");
    ASSERT(readingFileIsSuccessful(mountdir / "myfile"), ""); // just to make sure reading usually works

    // wait for cache timeout (i.e. flush file system to disk)
    const size_t CACHING_BLOCKSTORE_MAX_LIFETIME_SEC = 1; // TODO Use actual constant from the cache lifetime
    constexpr auto cache_timeout = CACHING_BLOCKSTORE_MAX_LIFETIME_SEC + cryfs::cachingfsblobstore::CachingFsBlobStore::MAX_LIFETIME_SEC;
    boost::this_thread::sleep_for(boost::chrono::seconds(static_cast<int>(std::ceil(cache_timeout * 3))));

    // roll back base directory
    bf::remove_all(basedir);
    recursive_copy(backup.path() / "basedir", basedir);

    // expect reading now fails
    EXPECT_FALSE(readingFileIsSuccessful(mountdir / "myfile"));
  });

  // Test it doesn't mount anymore now because it's marked with an integrity violation
  EXPECT_RUN_ERROR(args, "There was an integrity violation detected. Preventing any further access to the file system.", ErrorCode::IntegrityViolationOnPreviousRun);
}

}
