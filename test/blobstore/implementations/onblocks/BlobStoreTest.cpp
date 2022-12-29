#include "testutils/BlobStoreTest.h"
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using blockstore::BlockId;
using boost::none;

TEST_F(BlobStoreTest, TwoCreatedBlobsHaveDifferentKeys)
{
  auto blob1 = blobStore->create();
  auto blob2 = blobStore->create();
  EXPECT_NE(blob1->blockId(), blob2->blockId());
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteDirectly)
{
  auto blob = blobStore->create();
  BlockId blockId = blob->blockId();
  blobStore->remove(std::move(blob));
  EXPECT_FALSE(static_cast<bool>(blobStore->load(blockId)));
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteByKey)
{
  auto blockId = blobStore->create()->blockId();
  blobStore->remove(blockId);
  EXPECT_FALSE(static_cast<bool>(blobStore->load(blockId)));
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteAfterLoading)
{
  auto blob = blobStore->create();
  BlockId blockId = blob->blockId();
  reset(std::move(blob));
  blobStore->remove(loadBlob(blockId));
  EXPECT_FALSE(static_cast<bool>(blobStore->load(blockId)));
}
