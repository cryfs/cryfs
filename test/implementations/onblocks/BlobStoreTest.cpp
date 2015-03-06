#include "testutils/BlobStoreTest.h"

using blockstore::Key;

TEST_F(BlobStoreTest, LoadNonexistingKeyOnEmptyBlobstore) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(nullptr, blobStore->load(key));
}

TEST_F(BlobStoreTest, LoadNonexistingKeyOnNonEmptyBlobstore) {
  blobStore->create();
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(nullptr, blobStore->load(key));
}

TEST_F(BlobStoreTest, TwoCreatedBlobsHaveDifferentKeys) {
  auto blob1 = blobStore->create();
  auto blob2 = blobStore->create();
  EXPECT_NE(blob1->key(), blob2->key());
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteDirectly) {
  auto blob = blobStore->create();
  Key key = blob->key();
  blobStore->remove(std::move(blob));
  EXPECT_EQ(nullptr, blobStore->load(key).get());
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteAfterLoading) {
  auto blob = blobStore->create();
  Key key = blob->key();
  blob.reset();
  blobStore->remove(blobStore->load(key));
  EXPECT_EQ(nullptr, blobStore->load(key).get());
}
