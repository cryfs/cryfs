#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include <messmer/cpp-utils/pointer.h>
#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "../src/CryDevice.h"
#include "../src/CryDir.h"

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
    CryDevice dev(make_unique<CryConfig>(config.path()), make_unique<OnDiskBlockStore>(rootdir.path()));
    dev.Load(bf::path("/"));
  }
  CryDevice dev(make_unique<CryConfig>(config.path()), make_unique<OnDiskBlockStore>(rootdir.path()));
  dev.Load(bf::path("/"));
}
