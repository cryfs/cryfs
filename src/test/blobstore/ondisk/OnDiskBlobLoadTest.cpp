#include "gtest/gtest.h"

#include "test/testutils/TempFile.h"
#include "test/testutils/VirtualTestFile.h"

#include "blobstore/implementations/ondisk/OnDiskBlob.h"
#include "blobstore/implementations/ondisk/Data.h"
#include "blobstore/implementations/ondisk/FileDoesntExistException.h"

#include <fstream>

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::ofstream;
using std::unique_ptr;
using std::ios;

using namespace blobstore::ondisk;

namespace bf = boost::filesystem;

class OnDiskBlobLoadTest: public Test, public WithParamInterface<size_t> {
public:
  TempFile file;

  void SetFileSize(size_t size) {
    Data data(size);
    data.StoreToFile(file.path());
  }

  void StoreData(const VirtualTestFile &data) {
    //TODO Implement data.StoreToFile(filepath) instead
    Data dataobj(data.size());
    std::memcpy(dataobj.data(), data.data(), data.size());
    dataobj.StoreToFile(file.path());
  }

  unique_ptr<OnDiskBlob> LoadBlob() {
    return OnDiskBlob::LoadFromDisk(file.path());
  }

  void EXPECT_BLOB_DATA_EQ(const VirtualTestFile &expected, const OnDiskBlob &actual) {
    EXPECT_EQ(expected.size(), actual.size());
    EXPECT_EQ(0, std::memcmp(expected.data(), actual.data(), expected.size()));
  }
};
INSTANTIATE_TEST_CASE_P(OnDiskBlobLoadTest, OnDiskBlobLoadTest, Values(0, 1, 5, 1024, 10*1024*1024));

TEST_P(OnDiskBlobLoadTest, FileSizeIsCorrect) {
  SetFileSize(GetParam());

  auto blob = LoadBlob();

  EXPECT_EQ(GetParam(), blob->size());
}

TEST_P(OnDiskBlobLoadTest, LoadedDataIsCorrect) {
  VirtualTestFile randomData(GetParam());
  StoreData(randomData);

  auto blob = LoadBlob();

  EXPECT_BLOB_DATA_EQ(randomData, *blob);
}

TEST_F(OnDiskBlobLoadTest, LoadNotExistingBlob) {
  TempFile file2(false); // Pass false, so the file isn't created.
  EXPECT_THROW(
      OnDiskBlob::LoadFromDisk(file2.path()),
      FileDoesntExistException
  );
}
