#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include <messmer/cpp-utils/pointer/cast.h>
#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "../../src/filesystem/CryDevice.h"
#include "../../src/filesystem/CryDir.h"
#include "../../src/filesystem/CryFile.h"
#include "../../src/filesystem/CryOpenFile.h"

//TODO (whole project) Make constructors explicit when implicit construction not needed

using ::testing::Test;
using cpputils::TempDir;
using cpputils::TempFile;
using cpputils::dynamic_pointer_move;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::Console;
using blockstore::ondisk::OnDiskBlockStore;

namespace bf = boost::filesystem;
using namespace cryfs;

class MockConsole: public Console {
  void print(const std::string &) override {}
  unsigned int ask(const std::string &, const std::vector<std::string> &) override {
    return 0;
  }
  bool askYesNo(const std::string &) override {
    return true;
  }
};

class CryFsTest: public Test {
public:
  CryFsTest(): rootdir(), config(false) {}
  TempDir rootdir;
  TempFile config;
};

TEST_F(CryFsTest, CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(CryConfigLoader().createNewForTest(config.path()), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  }
  CryDevice dev(CryConfigFile::load(config.path()).value(), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root.get()).get()->children();
}

TEST_F(CryFsTest, UsingStrongKey1_CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(CryConfigLoader(make_unique_ref<MockConsole>()).createNew(config.path()), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  }
  CryDevice dev(CryConfigFile::load(config.path()).value(), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root.get()).get()->children();
}

TEST_F(CryFsTest, UsingStrongKey2_CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(CryConfigLoader(make_unique_ref<MockConsole>()).loadOrCreate(config.path()), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  }
  CryDevice dev(CryConfigLoader().loadOrCreate(config.path()), make_unique_ref<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root.get()).get()->children();
}
