#include "testutils/BlobStoreTest.h"
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using blockstore::Key;
using cpputils::unique_ref;
using blobstore::Blob;
using boost::none;

TEST_F(BlobStoreTest, LoadNonexistingKeyOnEmptyBlobstore) {
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(none, blobStore->load(key));
}

TEST_F(BlobStoreTest, LoadNonexistingKeyOnNonEmptyBlobstore) {
  blobStore->create();
  const blockstore::Key key = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  EXPECT_EQ(none, blobStore->load(key));
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
  EXPECT_FALSE((bool)blobStore->load(key));
}

TEST_F(BlobStoreTest, BlobIsNotLoadableAfterDeletion_DeleteAfterLoading) {
  auto blob = blobStore->create();
  Key key = blob->key();
  reset(std::move(blob));
  blobStore->remove(loadBlob(key));
  EXPECT_FALSE((bool)blobStore->load(key));
}
