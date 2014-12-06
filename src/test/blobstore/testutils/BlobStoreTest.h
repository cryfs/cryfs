#pragma once
#ifndef TEST_BLOBSTORE_IMPLEMENTATIONS_TESTUTILS_BLOBSTORETEST_H_
#define TEST_BLOBSTORE_IMPLEMENTATIONS_TESTUTILS_BLOBSTORETEST_H_

#include "test/testutils/TempDir.h"
#include "test/testutils/VirtualTestFile.h"
#include "blobstore/interface/BlobStore.h"
#include "blobstore/utils/RandomKeyGenerator.h"

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
};

template<class ConcreateBlobStoreTestFixture>
class BlobStoreSizeParameterizedTest {
public:
  BlobStoreSizeParameterizedTest(ConcreateBlobStoreTestFixture &fixture, size_t size_): blobStore(fixture.createBlobStore()), size(size_) {}

  void TestCreatedBlobHasCorrectSize() {
    auto blob = blobStore->create(size);
    EXPECT_EQ(size, blob.blob->size());
  }

  void TestLoadingUnchangedBlobHasCorrectSize() {
    auto blob = blobStore->create(size);
    auto loaded_blob = blobStore->load(blob.key);
    EXPECT_EQ(size, loaded_blob->size());
  }

  void TestCreatedBlobIsZeroedOut() {
    auto blob = blobStore->create(size);
    EXPECT_EQ(0, std::memcmp(ZEROES(size).data(), blob.blob->data(), size));
  }

  void TestLoadingUnchangedBlobIsZeroedOut() {
    auto blob = blobStore->create(size);
    auto loaded_blob = blobStore->load(blob.key);
    EXPECT_EQ(0, std::memcmp(ZEROES(size).data(), loaded_blob->data(), size));
  }

  void TestLoadedBlobIsCorrect() {
    VirtualTestFile randomData(size);
    auto loaded_blob = StoreDataToBlobAndLoadIt(randomData);
    EXPECT_EQ(size, loaded_blob->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_blob->data(), size));
  }

  void TestLoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing() {
    VirtualTestFile randomData(size);
    auto loaded_blob = StoreDataToBlobAndLoadItDirectlyAfterFlushing(randomData);
    EXPECT_EQ(size, loaded_blob->size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), loaded_blob->data(), size));
  }

  void TestAfterCreate_FlushingDoesntChangeBlob() {
    VirtualTestFile randomData(size);
    auto blob =  CreateBlob();
    WriteDataToBlob(blob.get(), randomData);
    blob->flush();

    EXPECT_BLOB_DATA_CORRECT(*blob, randomData);
  }

  void TestAfterLoad_FlushingDoesntChangeBlob() {
    VirtualTestFile randomData(size);
    auto blob =  CreateBlobAndLoadIt();
    WriteDataToBlob(blob.get(), randomData);
    blob->flush();

    EXPECT_BLOB_DATA_CORRECT(*blob, randomData);
  }

  void TestAfterCreate_FlushesWhenDestructed() {
    VirtualTestFile randomData(size);
    std::string key;
    {
      auto blob = blobStore->create(size);
      key = blob.key;
      WriteDataToBlob(blob.blob.get(), randomData);
    }
    auto loaded_blob = blobStore->load(key);
    EXPECT_BLOB_DATA_CORRECT(*loaded_blob, randomData);
  }

  void TestAfterLoad_FlushesWhenDestructed() {
    VirtualTestFile randomData(size);
    std::string key;
    {
      key = blobStore->create(size).key;
      auto blob = blobStore->load(key);
      WriteDataToBlob(blob.get(), randomData);
    }
    auto loaded_blob = blobStore->load(key);
    EXPECT_BLOB_DATA_CORRECT(*loaded_blob, randomData);
  }

  void TestLoadNonExistingBlobWithDefinitelyValidKey() {
    EXPECT_FALSE(
        (bool)blobStore->load(blobstore::RandomKeyGenerator::singleton().create())
    );
  }

  void TestLoadNonExistingBlobWithMaybeInvalidKey() {
    EXPECT_FALSE(
        (bool)blobStore->load("not-existing-key")
    );
  }

  void TestLoadNonExistingBlobWithEmptyKey() {
    EXPECT_FALSE(
        (bool)blobStore->load("")
    );
  }

private:
  std::unique_ptr<blobstore::BlobStore> blobStore;
  size_t size;

  blobstore::Data ZEROES(size_t size) {
    blobstore::Data ZEROES(size);
    ZEROES.FillWithZeroes();
    return ZEROES;
  }

  std::unique_ptr<blobstore::Blob> StoreDataToBlobAndLoadIt(const VirtualTestFile &data) {
    std::string key = StoreDataToBlobAndGetKey(data);
    return blobStore->load(key);
  }

  std::string StoreDataToBlobAndGetKey(const VirtualTestFile &data) {
    auto blob = blobStore->create(data.size());
    std::memcpy(blob.blob->data(), data.data(), data.size());
    return blob.key;
  }

  std::unique_ptr<blobstore::Blob> StoreDataToBlobAndLoadItDirectlyAfterFlushing(const VirtualTestFile &data) {
    auto blob = blobStore->create(data.size());
    std::memcpy(blob.blob->data(), data.data(), data.size());
    blob.blob->flush();
    return blobStore->load(blob.key);
  }

  std::unique_ptr<blobstore::Blob> CreateBlobAndLoadIt() {
    std::string key = blobStore->create(size).key;
    return blobStore->load(key);
  }

  std::unique_ptr<blobstore::Blob> CreateBlob() {
    return blobStore->create(size).blob;
  }

  void WriteDataToBlob(blobstore::Blob *blob, const VirtualTestFile &randomData) {
    std::memcpy(blob->data(), randomData.data(), randomData.size());
  }

  void EXPECT_BLOB_DATA_CORRECT(const blobstore::Blob &blob, const VirtualTestFile &randomData) {
    EXPECT_EQ(randomData.size(), blob.size());
    EXPECT_EQ(0, std::memcmp(randomData.data(), blob.data(), randomData.size()));
  }
};

TYPED_TEST_CASE_P(BlobStoreTest);

#define TYPED_TEST_P_FOR_ALL_SIZES(TestName)                           \
  TYPED_TEST_P(BlobStoreTest, TestName) {                              \
    for (auto size: this->SIZES) {                                     \
      BlobStoreSizeParameterizedTest<TypeParam>(this->fixture, size)   \
        .Test##TestName();                                             \
    }                                                                  \
  }                                                                    \


TYPED_TEST_P_FOR_ALL_SIZES(CreatedBlobHasCorrectSize);
TYPED_TEST_P_FOR_ALL_SIZES(LoadingUnchangedBlobHasCorrectSize);
TYPED_TEST_P_FOR_ALL_SIZES(CreatedBlobIsZeroedOut);
TYPED_TEST_P_FOR_ALL_SIZES(LoadingUnchangedBlobIsZeroedOut);
TYPED_TEST_P_FOR_ALL_SIZES(LoadedBlobIsCorrect);
TYPED_TEST_P_FOR_ALL_SIZES(LoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing);
TYPED_TEST_P_FOR_ALL_SIZES(AfterCreate_FlushingDoesntChangeBlob);
TYPED_TEST_P_FOR_ALL_SIZES(AfterLoad_FlushingDoesntChangeBlob);
TYPED_TEST_P_FOR_ALL_SIZES(AfterCreate_FlushesWhenDestructed);
TYPED_TEST_P_FOR_ALL_SIZES(AfterLoad_FlushesWhenDestructed);
TYPED_TEST_P_FOR_ALL_SIZES(LoadNonExistingBlobWithDefinitelyValidKey);
TYPED_TEST_P_FOR_ALL_SIZES(LoadNonExistingBlobWithMaybeInvalidKey);
TYPED_TEST_P_FOR_ALL_SIZES(LoadNonExistingBlobWithEmptyKey);

TYPED_TEST_P(BlobStoreTest, TwoCreatedBlobsHaveDifferentKeys) {
  auto blobStore = this->fixture.createBlobStore();
  auto blob1 = blobStore->create(1024);
  auto blob2 = blobStore->create(1024);
  EXPECT_NE(blob1.key, blob2.key);
}

REGISTER_TYPED_TEST_CASE_P(BlobStoreTest,
    CreatedBlobHasCorrectSize,
    LoadingUnchangedBlobHasCorrectSize,
    CreatedBlobIsZeroedOut,
    LoadingUnchangedBlobIsZeroedOut,
    LoadedBlobIsCorrect,
    LoadedBlobIsCorrectWhenLoadedDirectlyAfterFlushing,
    AfterCreate_FlushingDoesntChangeBlob,
    AfterLoad_FlushingDoesntChangeBlob,
    AfterCreate_FlushesWhenDestructed,
    AfterLoad_FlushesWhenDestructed,
    LoadNonExistingBlobWithDefinitelyValidKey,
    LoadNonExistingBlobWithMaybeInvalidKey,
    LoadNonExistingBlobWithEmptyKey,
    TwoCreatedBlobsHaveDifferentKeys
);


#endif
