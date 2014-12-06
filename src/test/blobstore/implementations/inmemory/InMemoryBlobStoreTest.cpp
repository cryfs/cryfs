#include "gtest/gtest.h"

#include "blobstore/implementations/inmemory/InMemoryBlobStore.h"
#include "blobstore/implementations/inmemory/InMemoryBlob.h"

#include "test/blobstore/testutils/BlobStoreTest.h"

using blobstore::BlobStore;
using blobstore::inmemory::InMemoryBlobStore;

using std::unique_ptr;
using std::make_unique;

class InMemoryBlobStoreTestFixture: public BlobStoreTestFixture {
public:
  unique_ptr<BlobStore> createBlobStore() override {
    return make_unique<InMemoryBlobStore>();
  }
};

INSTANTIATE_TYPED_TEST_CASE_P(InMemory, BlobStoreTest, InMemoryBlobStoreTestFixture);
