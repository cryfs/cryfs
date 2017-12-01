#include <gtest/gtest.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/pointer/cast.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore2.h>
#include <cryfs/filesystem/CryDevice.h>
#include <cryfs/filesystem/CryDir.h>
#include <cryfs/filesystem/CryFile.h>
#include <cryfs/filesystem/CryOpenFile.h>
#include "../testutils/MockConsole.h"
#include <cryfs/config/CryConfigLoader.h>
#include <cpp-utils/system/homedir.h>
#include "../testutils/TestWithFakeHomeDirectory.h"
#include <cpp-utils/io/NoninteractiveConsole.h>

//TODO (whole project) Make constructors explicit when implicit construction not needed

using ::testing::Test;
using std::make_shared;
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

namespace bf = boost::filesystem;
using namespace cryfs;

class CryFsTest: public Test, public TestWithMockConsole, public TestWithFakeHomeDirectory {
public:
  CryFsTest(): rootdir(), config(false) {
  }

  CryConfigFile loadOrCreateConfig() {
    auto askPassword = [] {return "mypassword";};
    return CryConfigLoader(make_shared<NoninteractiveConsole>(mockConsole()), Random::PseudoRandom(), SCrypt::TestSettings, askPassword, askPassword, none, none, none).loadOrCreate(config.path()).value().configFile;
  }

  unique_ref<OnDiskBlockStore2> blockStore() {
    return make_unique_ref<OnDiskBlockStore2>(rootdir.path());
  }

  TempDir rootdir;
  TempFile config;
};

TEST_F(CryFsTest, CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(loadOrCreateConfig(), blockStore(), 0x12345678, false, false);
  }
  CryDevice dev(loadOrCreateConfig(), blockStore(), 0x12345678, false, false);
  auto rootDir = dev.LoadDir(bf::path("/"));
  rootDir.value()->children();
}

TEST_F(CryFsTest, LoadingFilesystemDoesntModifyConfigFile) {
  {
    CryDevice dev(loadOrCreateConfig(), blockStore(), 0x12345678, false, false);
  }
  Data configAfterCreating = Data::LoadFromFile(config.path()).value();
  {
    CryDevice dev(loadOrCreateConfig(), blockStore(), 0x12345678, false, false);
  }
  Data configAfterLoading = Data::LoadFromFile(config.path()).value();
  EXPECT_EQ(configAfterCreating, configAfterLoading);
}
