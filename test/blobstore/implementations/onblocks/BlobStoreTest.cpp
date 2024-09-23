#include "testutils/BlobStoreTest.h"
#include "blockstore/utils/BlockId.h"
#include <boost/none.hpp>
#include <gtest/gtest.h>
#include <utility>

using blockstore::BlockId;
using boost::none;

TEST_F(BlobStoreTest, LoadNonexistingKeyOnEmptyBlobstore) {
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(none, blobStore->load(blockId));
}

TEST_F(BlobStoreTest, LoadNonexistingKeyOnNonEmptyBlobstore) {
  blobStore->create();
  const blockstore::BlockId blockId = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(none, blobStore->load(blockId));
}

TEST_F(BlobStoreTest, TwoCreatedBlobsHaveDifferentKeys) {
  auto blob1 = blobStore->create();
  auto blob2 = blobStore->create();
  EXPECT_NE(blob1->blockId(), blob2->blockId());
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteDirectly) {
  auto blob = blobStore->create();
  const BlockId blockId = blob->blockId();
  blobStore->remove(std::move(blob));
  EXPECT_FALSE(static_cast<bool>(blobStore->load(blockId)));
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteByKey) {
  auto blockId = blobStore->create()->blockId();
  blobStore->remove(blockId);
  EXPECT_FALSE(static_cast<bool>(blobStore->load(blockId)));
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteAfterLoading) {
  auto blob = blobStore->create();
  const BlockId blockId = blob->blockId();
  reset(std::move(blob));
  blobStore->remove(loadBlob(blockId));
  EXPECT_FALSE(static_cast<bool>(blobStore->load(blockId)));
}
