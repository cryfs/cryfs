#include <google/gtest/gtest.h>

#include "../../../../interface/BlobStore.h"

class BlobStoreTest: public ::testing::Test {
public:
  BlobStoreTest();

  static constexpr uint32_t BLOCKSIZE_BYTES = 4096;

  std::unique_ptr<blobstore::BlobStore> blobStore;
};
