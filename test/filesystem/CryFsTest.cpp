#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include <messmer/cpp-utils/pointer/cast.h>
#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "../../src/filesystem/CryDevice.h"
#include "../../src/filesystem/CryDir.h"
#include "../../src/filesystem/CryFile.h"
#include "../../src/filesystem/CryOpenFile.h"
#include "../testutils/MockConsole.h"

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
using blockstore::ondisk::OnDiskBlockStore;

namespace bf = boost::filesystem;
using namespace cryfs;

class CryFsTest: public Test {
public:
  CryFsTest(): rootdir(), config(false) {
  }
  unique_ref<MockConsole> mockConsole() {
    auto console = make_unique_ref<MockConsole>();
    EXPECT_CALL(*console, ask(_, _)).WillRepeatedly(Return(0));
    EXPECT_CALL(*console, askYesNo(_)).WillRepeatedly(Return(true));
    return console;
  }
  TempDir rootdir;
  TempFile config;
};

TEST_F(CryFsTest, CreatedRootdirIsLoadableAfterClosing_1) {
  {
    CryDevice dev(
        CryConfigLoader(mockConsole(), cpputils::Random::PseudoRandom())
            .createNew(config.path()), make_unique_ref<OnDiskBlockStore>(rootdir.path())
    );
  }
  CryDevice dev(CryConfigFile::load(config.path()).value(), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root.get()).get()->children();
}

TEST_F(CryFsTest, CreatedRootdirIsLoadableAfterClosing_2) {
  {
    CryDevice dev(
        CryConfigLoader(mockConsole(), cpputils::Random::PseudoRandom())
            .loadOrCreate(config.path()), make_unique_ref<OnDiskBlockStore>(rootdir.path())
    );
  }
  CryDevice dev(CryConfigLoader().loadOrCreate(config.path()), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root.get()).get()->children();
}
