#include "gtest/gtest.h"

#include "blobstore/implementations/ondisk/OnDiskBlobStore.h"

#include "test/testutils/TempDir.h"
#include "test/testutils/VirtualTestFile.h"

using ::testing::Test;
using ::testing::WithParamInterface;
using ::testing::Values;

using std::string;
using std::unique_ptr;

using blobstore::Blob;
using namespace blobstore::ondisk;

class OnDiskBlobStoreTest: public Test {
public:
  TempDir dir;
  OnDiskBlobStore blobStore;

  OnDiskBlobStoreTest(): dir(), blobStore(dir.path()) {}
};

class OnDiskBlobStoreSizeTest: public OnDiskBlobStoreTest, public WithParamInterface<size_t> {
public:
  unique_ptr<Blob> StoreDataToBlobAndLoadIt(const VirtualTestFile &data) {
    string key = StoreDataToBlobAndGetKey(data);
    return blobStore.load(key);
  }

  string StoreDataToBlobAndGetKey(const VirtualTestFile &data) {
    auto blob = blobStore.create(GetParam());
    std::memcpy(blob.blob->data(), data.data(), data.size());
    return blob.key;
  }

  unique_ptr<Blob> StoreDataToBlobAndLoadItDirectlyAfterFlushing(const VirtualTestFile &data) {
    auto blob = blobStore.create(GetParam());
    std::memcpy(blob.blob->data(), data.data(), data.size());
    blob.blob->flush();
    return blobStore.load(blob.key);
  }
};
INSTANTIATE_TEST_CASE_P(OnDiskBlobStoreSizeTest, OnDiskBlobStoreSizeTest, Values(0, 1, 1024, 4096, 10*1024*1024));

TEST_P(OnDiskBlobStoreSizeTest, CreateBlobAndCheckSize) {
  auto blob = blobStore.create(GetParam());
  EXPECT_EQ(GetParam(), blob.blob->size());
}

TEST_P(OnDiskBlobStoreSizeTest, LoadedBlobIsCorrect) {
  VirtualTestFile randomData(GetParam());
  auto loaded_blob = StoreDataToBlobAndLoadIt(randomData);
  EXPECT_EQ(randomData.size(), loaded_blob->size());
  EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_blob->data(), randomData.size()));
}

TEST_P(OnDiskBlobStoreSizeTest, LoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing) {
  VirtualTestFile randomData(GetParam());
  auto loaded_blob = StoreDataToBlobAndLoadItDirectlyAfterFlushing(randomData);
  EXPECT_EQ(randomData.size(), loaded_blob->size());
  EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_blob->data(), randomData.size()));
}

TEST_F(OnDiskBlobStoreTest, TwoCreatedBlobsHaveDifferentKeys) {
  auto blob1 = blobStore.create(1024);
  auto blob2 = blobStore.create(1024);
  EXPECT_NE(blob1.key, blob2.key);
}
