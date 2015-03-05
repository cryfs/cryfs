#include "testutils/BlobStoreTest.h"

class BlobOnBlocksTest: public BlobStoreTest {};

TEST_F(BlobOnBlocksTest, CreatedBlobIsEmpty) {
  auto blob = blobStore->create();
  EXPECT_EQ(0, blob->size());
}
