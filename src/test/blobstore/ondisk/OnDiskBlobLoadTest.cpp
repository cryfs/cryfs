#include "gtest/gtest.h"

#include "test/testutils/TempFile.h"

#include "blobstore/implementations/ondisk/OnDiskBlob.h"
#include "blobstore/implementations/ondisk/Data.h"

#include <fstream>

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::ofstream;
using std::unique_ptr;
using std::ios;

using namespace blobstore::ondisk;

namespace bf = boost::filesystem;

class OnDiskBlobLoadTest: public Test {
public:
  TempFile file;

  void SetFileSize(size_t size) {
    Data data(size);
    data.StoreToFile(file.path());
  }
};

class OnDiskBlobLoadSizeTest: public OnDiskBlobLoadTest, public WithParamInterface<size_t> {};
INSTANTIATE_TEST_CASE_P(OnDiskBlobLoadSizeTest, OnDiskBlobLoadSizeTest, Values(0, 1, 5, 1024, 10*1024*1024));

TEST_P(OnDiskBlobLoadSizeTest, FileSizeIsCorrect) {
  SetFileSize(GetParam());

  auto blob = OnDiskBlob::LoadFromDisk(file.path());

  EXPECT_EQ(GetParam(), blob->size());
}

//TODO Load and compare actual data
//TODO Test file doesn't exist

