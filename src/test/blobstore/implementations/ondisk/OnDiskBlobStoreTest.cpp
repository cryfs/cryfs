#include "gtest/gtest.h"

#include "blobstore/implementations/ondisk/OnDiskBlobStore.h"
#include "blobstore/implementations/ondisk/OnDiskBlob.h"

#include "test/blobstore/testutils/BlobStoreTest.h"

using blobstore::BlobStore;
using blobstore::ondisk::OnDiskBlobStore;

using std::unique_ptr;
using std::make_unique;

class OnDiskBlobStoreTestFixture: public BlobStoreTestFixture {
public:
  unique_ptr<BlobStore> createBlobStore() override {
    return make_unique<OnDiskBlobStore>(tempdir.path());
  }
private:
  TempDir tempdir;
};

INSTANTIATE_TYPED_TEST_CASE_P(OnDisk, BlobStoreTest, OnDiskBlobStoreTestFixture);
