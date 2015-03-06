#include "testutils/BlobStoreTest.h"

using blockstore::Key;

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

//TODO Test read/write
//TODO Test read/write with loading inbetween

//Taken from BlockStoreTest.h:
//TODO Created blob is zeroed out
//TODO Created blob is zeroed out after loading
//TODO Try loading nonexisting blob
