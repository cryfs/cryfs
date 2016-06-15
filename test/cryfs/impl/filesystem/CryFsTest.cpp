#include <gtest/gtest.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/pointer/cast.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <cryfs/impl/filesystem/CryDir.h>
#include <cryfs/impl/filesystem/CryFile.h>
#include <cryfs/impl/filesystem/CryOpenFile.h>
#include "../testutils/MockConsole.h"
#include <cryfs/impl/config/CryConfigLoader.h>

//TODO (whole project) Make constructors explicit when implicit construction not needed

using ::testing::Test;
using ::testing::Return;
using ::testing::_;
using cpputils::TempDir;
using cpputils::TempFile;
using cpputils::dynamic_pointer_move;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::Console;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::Data;
using blockstore::ondisk::OnDiskBlockStore;
using boost::none;
using std::shared_ptr;

namespace bf = boost::filesystem;
using namespace cryfs;

class CryFsTest: public Test, public TestWithMockConsole {
public:
  CryFsTest(): rootdir(), config(false) {
  }

  shared_ptr<CryConfigFile> loadOrCreateConfig() {
    auto askPassword = [] {return "mypassword";};
    auto configFile = CryConfigLoader(mockConsole(), Random::PseudoRandom(), SCrypt::TestSettings, askPassword, askPassword, none, none, true).loadOrCreate(config.path()).value();
    return cpputils::to_unique_ptr(std::move(configFile));
  }

  unique_ref<OnDiskBlockStore> blockStore() {
    return make_unique_ref<OnDiskBlockStore>(rootdir.path());
  }

  TempDir rootdir;
  TempFile config;
};

TEST_F(CryFsTest, CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(loadOrCreateConfig(), blockStore());
  }
  CryDevice dev(loadOrCreateConfig(), blockStore());
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root.get()).get()->children();
}

TEST_F(CryFsTest, LoadingFilesystemDoesntModifyConfigFile) {
  {
    CryDevice dev(loadOrCreateConfig(), blockStore());
  }
  Data configAfterCreating = Data::LoadFromFile(config.path()).value();
  {
    CryDevice dev(loadOrCreateConfig(), blockStore());
  }
  Data configAfterLoading = Data::LoadFromFile(config.path()).value();
  EXPECT_EQ(configAfterCreating, configAfterLoading);
}
