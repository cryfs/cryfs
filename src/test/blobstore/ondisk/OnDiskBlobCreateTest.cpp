#include "gtest/gtest.h"

#include "test/testutils/TempFile.h"

#include "blobstore/implementations/ondisk/OnDiskBlob.h"
#include "blobstore/implementations/ondisk/FileAlreadyExistsException.h"

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::unique_ptr;

using namespace blobstore::ondisk;

namespace bf = boost::filesystem;

class OnDiskBlobCreateTest: public Test {
public:
  OnDiskBlobCreateTest()
  // Don't create the temp file yet (therefore pass false to the TempFile constructor)
  : file(false) {
  }
  TempFile file;
};

TEST_F(OnDiskBlobCreateTest, CreatingBlobCreatesFile) {
  EXPECT_FALSE(bf::exists(file.path()));

  auto blob = OnDiskBlob::CreateOnDisk(file.path(), 0);

  EXPECT_TRUE(bf::exists(file.path()));
  EXPECT_TRUE(bf::is_regular_file(file.path()));
}

TEST_F(OnDiskBlobCreateTest, CreatingExistingBlobThrowsException) {
  auto blob1 = OnDiskBlob::CreateOnDisk(file.path(), 0);
  EXPECT_THROW(OnDiskBlob::CreateOnDisk(file.path(), 0), FileAlreadyExistsException);
}

class OnDiskBlobCreateSizeTest: public OnDiskBlobCreateTest, public WithParamInterface<size_t> {};
INSTANTIATE_TEST_CASE_P(OnDiskBlobCreateSizeTest, OnDiskBlobCreateSizeTest, Values(0, 1, 5, 1024, 10*1024*1024));

TEST_P(OnDiskBlobCreateSizeTest, FileSizeIsCorrect) {
  auto blob = OnDiskBlob::CreateOnDisk(file.path(), GetParam());

  EXPECT_EQ(GetParam(), bf::file_size(file.path()));
}

TEST_P(OnDiskBlobCreateSizeTest, InMemorySizeIsCorrect) {
  auto blob = OnDiskBlob::CreateOnDisk(file.path(), GetParam());

  EXPECT_EQ(GetParam(), blob->size());
}

