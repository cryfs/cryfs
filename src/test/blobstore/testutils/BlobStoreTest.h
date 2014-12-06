#pragma once
#ifndef TEST_BLOBSTORE_IMPLEMENTATIONS_TESTUTILS_BLOBSTORETEST_H_
#define TEST_BLOBSTORE_IMPLEMENTATIONS_TESTUTILS_BLOBSTORETEST_H_

#include "test/testutils/TempDir.h"
#include "test/testutils/VirtualTestFile.h"
#include "blobstore/interface/BlobStore.h"

class BlobStoreTestFixture {
public:
  virtual std::unique_ptr<blobstore::BlobStore> createBlobStore() = 0;
};

template<class ConcreteBlobStoreTestFixture>
class BlobStoreTest: public ::testing::Test {
public:
  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<BlobStoreTestFixture, ConcreteBlobStoreTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) BlobStoreTest must inherit from BlobStoreTestFixture"
  );

  const std::vector<size_t> SIZES = {0, 1, 1024, 4096, 10*1024*1024};

  ConcreteBlobStoreTestFixture fixture;

  void TestCreateBlobAndCheckSize(size_t size) {
    auto blobStore = fixture.createBlobStore();
    auto blob = blobStore->create(size);
    EXPECT_EQ(size, blob.blob->size());
  }

  void TestLoadedBlobIsCorrect(size_t size) {
    auto blobStore = fixture.createBlobStore();
    VirtualTestFile randomData(size);
    auto loaded_blob = StoreDataToBlobAndLoadIt(blobStore.get(), randomData);
    EXPECT_EQ(size, loaded_blob->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_blob->data(), size));
  }

  std::unique_ptr<blobstore::Blob> StoreDataToBlobAndLoadIt(blobstore::BlobStore *blobStore, const VirtualTestFile &data) {
    std::string key = StoreDataToBlobAndGetKey(blobStore, data);
    return blobStore->load(key);
  }

  std::string StoreDataToBlobAndGetKey(blobstore::BlobStore *blobStore, const VirtualTestFile &data) {
    auto blob = blobStore->create(data.size());
    std::memcpy(blob.blob->data(), data.data(), data.size());
    return blob.key;
  }

  void TestLoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing(size_t size) {
    auto blobStore = fixture.createBlobStore();
    VirtualTestFile randomData(size);
    auto loaded_blob = StoreDataToBlobAndLoadItDirectlyAfterFlushing(blobStore.get(), randomData);
    EXPECT_EQ(size, loaded_blob->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_blob->data(), size));
  }

  std::unique_ptr<blobstore::Blob> StoreDataToBlobAndLoadItDirectlyAfterFlushing(blobstore::BlobStore *blobStore, const VirtualTestFile &data) {
    auto blob = blobStore->create(data.size());
    std::memcpy(blob.blob->data(), data.data(), data.size());
    blob.blob->flush();
    return blobStore->load(blob.key);
  }
};

TYPED_TEST_CASE_P(BlobStoreTest);

TYPED_TEST_P(BlobStoreTest, CreateBlobAndCheckSize) {
  for (auto size: this->SIZES) {
    this->TestCreateBlobAndCheckSize(size);
  }
}

TYPED_TEST_P(BlobStoreTest, LoadedBlobIsCorrect) {
  for (auto size: this->SIZES) {
    this->TestLoadedBlobIsCorrect(size);
  }
}

TYPED_TEST_P(BlobStoreTest, LoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing) {
  for (auto size: this->SIZES) {
    this->TestLoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing(size);
  }
}

TYPED_TEST_P(BlobStoreTest, TwoCreatedBlobsHaveDifferentKeys) {
  auto blobStore = this->fixture.createBlobStore();
  auto blob1 = blobStore->create(1024);
  auto blob2 = blobStore->create(1024);
  EXPECT_NE(blob1.key, blob2.key);
}

REGISTER_TYPED_TEST_CASE_P(BlobStoreTest, CreateBlobAndCheckSize, LoadedBlobIsCorrect, LoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing, TwoCreatedBlobsHaveDifferentKeys);


#endif
