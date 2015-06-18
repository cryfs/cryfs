#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include <messmer/cpp-utils/pointer.h>
#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "../src/CryDevice.h"
#include "../src/CryDir.h"
#include "../src/CryFile.h"
#include "../src/CryOpenFile.h"

//TODO (whole project) Make constructors explicit when implicit construction not needed

using ::testing::Test;
using cpputils::TempDir;
using cpputils::TempFile;
using cpputils::dynamic_pointer_move;
using std::make_unique;
using blockstore::ondisk::OnDiskBlockStore;

namespace bf = boost::filesystem;
using namespace cryfs;

class CryFsTest: public Test {
public:
  CryFsTest(): rootdir(), config(false) {}
  TempDir rootdir;
  TempFile config;
};

TEST_F(CryFsTest, CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(CryConfigLoader::createNewWithWeakKey(config.path()), make_unique<OnDiskBlockStore>(rootdir.path()));
  }
  CryDevice dev(std::move(CryConfigLoader::loadExisting(config.path()).get()), make_unique<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root)->children();
}

TEST_F(CryFsTest, UsingStrongKey1_CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(CryConfigLoader::createNew(config.path()), make_unique<OnDiskBlockStore>(rootdir.path()));
  }
  CryDevice dev(std::move(CryConfigLoader::loadExisting(config.path()).get()), make_unique<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root)->children();
}

TEST_F(CryFsTest, UsingStrongKey2_CreatedRootdirIsLoadableAfterClosing) {
  {
    CryDevice dev(CryConfigLoader::loadOrCreate(config.path()), make_unique<OnDiskBlockStore>(rootdir.path()));
  }
  CryDevice dev(CryConfigLoader::loadOrCreate(config.path()), make_unique<OnDiskBlockStore>(rootdir.path()));
  auto root = dev.Load(bf::path("/"));
  dynamic_pointer_move<CryDir>(root)->children();
}
