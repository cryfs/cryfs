#pragma once
#ifndef TEST_BLOBSTORE_IMPLEMENTATIONS_TESTUTILS_BLOBSTOREWITHRANDOMKEYSTEST_H_
#define TEST_BLOBSTORE_IMPLEMENTATIONS_TESTUTILS_BLOBSTOREWITHRANDOMKEYSTEST_H_

#include <test/testutils/DataBlockFixture.h>
#include "test/testutils/TempDir.h"
#include "blobstore/interface/BlobStore.h"
#include "blobstore/utils/RandomKeyGenerator.h"

class BlobStoreWithRandomKeysTestFixture {
public:
  virtual std::unique_ptr<blobstore::BlobStoreWithRandomKeys> createBlobStore() = 0;
};

template<class ConcreteBlobStoreWithRandomKeysTestFixture>
class BlobStoreWithRandomKeysTest: public ::testing::Test {
public:
  BOOST_STATIC_ASSERT_MSG(
    (std::is_base_of<BlobStoreWithRandomKeysTestFixture, ConcreteBlobStoreWithRandomKeysTestFixture>::value),
    "Given test fixture for instantiating the (type parameterized) BlobStoreWithRandomKeysTest must inherit from BlobStoreWithRandomKeysTestFixture"
  );

  const std::vector<size_t> SIZES = {0, 1, 1024, 4096, 10*1024*1024};

  ConcreteBlobStoreWithRandomKeysTestFixture fixture;
};

TYPED_TEST_CASE_P(BlobStoreWithRandomKeysTest);

TYPED_TEST_P(BlobStoreWithRandomKeysTest, CreateTwoBlobsWithSameKeyAndSameSize) {
  auto blobStore = this->fixture.createBlobStore();
  auto blob = blobStore->create("mykey", 1024);
  auto blob2 = blobStore->create("mykey", 1024);
  EXPECT_TRUE((bool)blob);
  EXPECT_FALSE((bool)blob2);
}

TYPED_TEST_P(BlobStoreWithRandomKeysTest, CreateTwoBlobsWithSameKeyAndDifferentSize) {
  auto blobStore = this->fixture.createBlobStore();
  auto blob = blobStore->create("mykey", 1024);
  auto blob2 = blobStore->create("mykey", 4096);
  EXPECT_TRUE((bool)blob);
  EXPECT_FALSE((bool)blob2);
}

TYPED_TEST_P(BlobStoreWithRandomKeysTest, CreateTwoBlobsWithSameKeyAndFirstNullSize) {
  auto blobStore = this->fixture.createBlobStore();
  auto blob = blobStore->create("mykey", 0);
  auto blob2 = blobStore->create("mykey", 1024);
  EXPECT_TRUE((bool)blob);
  EXPECT_FALSE((bool)blob2);
}

TYPED_TEST_P(BlobStoreWithRandomKeysTest, CreateTwoBlobsWithSameKeyAndSecondNullSize) {
  auto blobStore = this->fixture.createBlobStore();
  auto blob = blobStore->create("mykey", 1024);
  auto blob2 = blobStore->create("mykey", 0);
  EXPECT_TRUE((bool)blob);
  EXPECT_FALSE((bool)blob2);
}

TYPED_TEST_P(BlobStoreWithRandomKeysTest, CreateTwoBlobsWithSameKeyAndBothNullSize) {
  auto blobStore = this->fixture.createBlobStore();
  auto blob = blobStore->create("mykey", 0);
  auto blob2 = blobStore->create("mykey", 0);
  EXPECT_TRUE((bool)blob);
  EXPECT_FALSE((bool)blob2);
}

REGISTER_TYPED_TEST_CASE_P(BlobStoreWithRandomKeysTest,
  CreateTwoBlobsWithSameKeyAndSameSize,
  CreateTwoBlobsWithSameKeyAndDifferentSize,
  CreateTwoBlobsWithSameKeyAndFirstNullSize,
  CreateTwoBlobsWithSameKeyAndSecondNullSize,
  CreateTwoBlobsWithSameKeyAndBothNullSize
);


#endif
