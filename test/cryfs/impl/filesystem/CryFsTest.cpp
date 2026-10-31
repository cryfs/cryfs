#include <gtest/gtest.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/pointer/cast.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore2.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <cryfs/impl/filesystem/CryDir.h>
#include <cryfs/impl/filesystem/CryFile.h>
#include <cryfs/impl/filesystem/CryOpenFile.h>
#include "../testutils/MockConsole.h"
#include <cryfs/impl/config/CryConfigLoader.h>
#include <cryfs/impl/config/CryPresetPasswordBasedKeyProvider.h>
#include <cpp-utils/system/homedir.h>
#include "../testutils/TestWithFakeHomeDirectory.h"
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <chrono>
#include <thread>

//TODO (whole project) Make constructors explicit when implicit construction not needed

using ::testing::Test;
using std::make_shared;
using std::shared_ptr;
using cpputils::TempDir;
using cpputils::TempFile;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::Data;
using cpputils::NoninteractiveConsole;
using blockstore::ondisk::OnDiskBlockStore2;
using boost::none;
using cryfs::CryPresetPasswordBasedKeyProvider;

namespace bf = boost::filesystem;
using namespace cryfs;

namespace {

class CryFsTest: public Test, public TestWithMockConsole, public TestWithFakeHomeDirectory {
public:
  CryFsTest(): tempLocalStateDir(), localStateDir(tempLocalStateDir.path()), rootdir(), config(false) {
  }

  shared_ptr<CryConfigFile> loadOrCreateConfig() {
    auto keyProvider = make_unique_ref<CryPresetPasswordBasedKeyProvider>("mypassword", make_unique_ref<SCrypt>(SCrypt::TestSettings));
    return CryConfigLoader(make_shared<NoninteractiveConsole>(mockConsole()), Random::Csprng(), std::move(keyProvider), localStateDir, none, none, none).loadOrCreate(config.path(), false, false).right().configFile;
  }

  unique_ref<OnDiskBlockStore2> blockStore() {
    return make_unique_ref<OnDiskBlockStore2>(rootdir.path());
  }

  cpputils::TempDir tempLocalStateDir;
  LocalStateDir localStateDir;
  TempDir rootdir;
  TempFile config;
};

auto failOnIntegrityViolation() {
  return [] {
    EXPECT_TRUE(false);
  };
}

TEST_F(CryFsTest, CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
    dev.setContext(fspp::Context {fspp::relatime()});
  }
  CryDevice dev(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
  dev.setContext(fspp::Context {fspp::relatime()});
  auto rootDir = dev.LoadDir(bf::path("/"));
  rootDir.value()->children();
}

TEST_F(CryFsTest, LoadingFilesystemDoesntModifyConfigFile) {
  {
    CryDevice dev(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
    dev.setContext(fspp::Context {fspp::relatime()});
  }
  const Data configAfterCreating = Data::LoadFromFile(config.path()).value();
  {
    CryDevice dev(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
    dev.setContext(fspp::Context {fspp::relatime()});
  }
  const Data configAfterLoading = Data::LoadFromFile(config.path()).value();
  EXPECT_EQ(configAfterCreating, configAfterLoading);
}

// The caches write a new file's directory entry to disk within seconds, even while the file is still open.
// The file's blob has to be on disk by then too, or after a crash the entry can't be read or deleted.
TEST_F(CryFsTest, NewFileThatIsStillOpenIsReadableAfterCrash) {
  {
    // Create the file system, so its root directory is on disk
    CryDevice dev(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
    dev.setContext(fspp::Context {fspp::relatime()});
  }
  CryDevice dev(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
  dev.setContext(fspp::Context {fspp::relatime()});
  const auto openFile = dev.LoadDir(bf::path("/")).value()->createAndOpenFile("newfile", fspp::mode_t().addFileFlag().addUserReadFlag().addUserWriteFlag(), fspp::uid_t(0), fspp::gid_t(0));

  // Load the file system again while the first instance keeps running with the file open, as if it had crashed.
  // Wait until its caches have written the directory entry, and then a bit longer, so they're done writing.
  bool entryIsOnDisk = false;
  for (int attempt = 0; attempt < 100 && !entryIsOnDisk; ++attempt) {
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    try {
      CryDevice afterCrash(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
      afterCrash.setContext(fspp::Context {fspp::relatime()});
      entryIsOnDisk = afterCrash.Load(bf::path("/newfile")) != none;
    } catch (const std::exception &) { // NOLINT(bugprone-empty-catch)
      // The first instance doesn't write block files atomically, and it may have been writing the one we read
    }
  }
  ASSERT_TRUE(entryIsOnDisk) << "The directory entry didn't reach the disk";
  std::this_thread::sleep_for(std::chrono::seconds(2));

  CryDevice afterCrash(loadOrCreateConfig(), blockStore(), localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
  afterCrash.setContext(fspp::Context {fspp::relatime()});
  const auto file = afterCrash.Load(bf::path("/newfile"));
  ASSERT_TRUE(file != none);
  EXPECT_EQ(fspp::num_bytes_t(0), file.value()->stat().size);
}

}
