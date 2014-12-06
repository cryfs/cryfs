#include "gtest/gtest.h"

#include "test/testutils/TempFile.h"
#include "test/testutils/VirtualTestFile.h"

#include "blobstore/implementations/ondisk/OnDiskBlob.h"
#include "blobstore/implementations/ondisk/FileAlreadyExistsException.h"

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::unique_ptr;

using namespace blobstore;
using namespace blobstore::ondisk;

namespace bf = boost::filesystem;

class OnDiskBlobFlushTest: public Test, public WithParamInterface<size_t> {
public:
  OnDiskBlobFlushTest()
  // Don't create the temp file yet (therefore pass false to the TempFile constructor)
  : file(false), randomData(GetParam()) {
  }
  TempFile file;

  VirtualTestFile randomData;

  unique_ptr<OnDiskBlob> CreateBlobAndLoadItFromDisk() {
    {
      auto blob = OnDiskBlob::CreateOnDisk(file.path(), randomData.size());
    }
    return OnDiskBlob::LoadFromDisk(file.path());
  }

  unique_ptr<OnDiskBlob> CreateBlob() {
    return OnDiskBlob::CreateOnDisk(file.path(), randomData.size());
  }

  void WriteDataToBlob(const unique_ptr<OnDiskBlob> &blob) {
    std::memcpy(blob->data(), randomData.data(), randomData.size());
  }

  void EXPECT_BLOB_DATA_CORRECT(const unique_ptr<OnDiskBlob> &blob) {
    EXPECT_EQ(randomData.size(), blob->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), blob->data(), randomData.size()));
  }

  void EXPECT_STORED_FILE_DATA_CORRECT() {
    Data actual = Data::LoadFromFile(file.path());
    EXPECT_EQ(randomData.size(), actual.size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), actual.data(), randomData.size()));
  }
};
INSTANTIATE_TEST_CASE_P(OnDiskBlobFlushTest, OnDiskBlobFlushTest, Values((size_t)0, (size_t)1, (size_t)1024, (size_t)4096, (size_t)10*1024*1024));

TEST_P(OnDiskBlobFlushTest, AfterCreate_FlushingDoesntChangeBlob) {
  auto blob =  CreateBlob();
  WriteDataToBlob(blob);
  blob->flush();

  EXPECT_BLOB_DATA_CORRECT(blob);
}

TEST_P(OnDiskBlobFlushTest, AfterLoad_FlushingDoesntChangeBlob) {
  auto blob =  CreateBlobAndLoadItFromDisk();
  WriteDataToBlob(blob);
  blob->flush();

  EXPECT_BLOB_DATA_CORRECT(blob);
}

TEST_P(OnDiskBlobFlushTest, AfterCreate_FlushingWritesCorrectData) {
  auto blob = CreateBlob();
  WriteDataToBlob(blob);
  blob->flush();

  EXPECT_STORED_FILE_DATA_CORRECT();
}

TEST_P(OnDiskBlobFlushTest, AfterLoad_FlushingWritesCorrectData) {
  auto blob = CreateBlobAndLoadItFromDisk();
  WriteDataToBlob(blob);
  blob->flush();

  EXPECT_STORED_FILE_DATA_CORRECT();
}

TEST_P(OnDiskBlobFlushTest, AfterCreate_FlushesWhenDestructed) {
  {
    auto blob = CreateBlob();
    WriteDataToBlob(blob);
  }
  EXPECT_STORED_FILE_DATA_CORRECT();
}

TEST_P(OnDiskBlobFlushTest, AfterLoad_FlushesWhenDestructed) {
  {
    auto blob = CreateBlobAndLoadItFromDisk();
    WriteDataToBlob(blob);
  }
  EXPECT_STORED_FILE_DATA_CORRECT();
}
