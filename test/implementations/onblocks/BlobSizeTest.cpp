#include "testutils/BlobStoreTest.h"

using std::unique_ptr;

using namespace blobstore;
using blockstore::Key;

class BlobSizeTest: public BlobStoreTest {
public:
  BlobSizeTest(): blob(blobStore->create()) {}

  static constexpr uint32_t LARGE_SIZE = 10 * 1024 * 1024;

  unique_ptr<Blob> blob;
};
constexpr uint32_t BlobSizeTest::LARGE_SIZE;

TEST_F(BlobSizeTest, CreatedBlobIsEmpty) {
  EXPECT_EQ(0, blob->size());
}

TEST_F(BlobSizeTest, Growing_1Byte) {
  blob->resize(1);
  EXPECT_EQ(1, blob->size());
}

TEST_F(BlobSizeTest, Growing_Large) {
  blob->resize(LARGE_SIZE);
  EXPECT_EQ(LARGE_SIZE, blob->size());
}

TEST_F(BlobSizeTest, Shrinking_Empty) {
  blob->resize(LARGE_SIZE);
  blob->resize(0);
  EXPECT_EQ(0, blob->size());
}

TEST_F(BlobSizeTest, Shrinking_1Byte) {
  blob->resize(LARGE_SIZE);
  blob->resize(1);
  EXPECT_EQ(1, blob->size());
}

TEST_F(BlobSizeTest, ResizingToItself_Empty) {
  blob->resize(0);
  EXPECT_EQ(0, blob->size());
}

TEST_F(BlobSizeTest, ResizingToItself_1Byte) {
  blob->resize(1);
  blob->resize(1);
  EXPECT_EQ(1, blob->size());
}

TEST_F(BlobSizeTest, ResizingToItself_Large) {
  blob->resize(LARGE_SIZE);
  blob->resize(LARGE_SIZE);
  EXPECT_EQ(LARGE_SIZE, blob->size());
}

TEST_F(BlobSizeTest, EmptyBlobStaysEmptyWhenLoading) {
  Key key = blob->key();
  blob.reset();
  auto loaded = blobStore->load(key);
  EXPECT_EQ(0, loaded->size());
}

TEST_F(BlobSizeTest, BlobSizeStaysIntactWhenLoading) {
  blob->resize(LARGE_SIZE);
  Key key = blob->key();
  blob.reset();
  auto loaded = blobStore->load(key);
  EXPECT_EQ(LARGE_SIZE, loaded->size());
}

TEST_F(BlobSizeTest, WritingAtEndOfBlobGrowsBlob_Empty) {
  int value;
  blob->write(&value, 0, 4);
  EXPECT_EQ(4, blob->size());
}

TEST_F(BlobSizeTest, WritingAfterEndOfBlobGrowsBlob_Empty) {
  int value;
  blob->write(&value, 2, 4);
  EXPECT_EQ(6, blob->size());
}

TEST_F(BlobSizeTest, WritingOverEndOfBlobGrowsBlob_NonEmpty) {
  blob->resize(1);
  int value;
  blob->write(&value, 0, 4);
  EXPECT_EQ(4, blob->size());
}

TEST_F(BlobSizeTest, WritingAtEndOfBlobGrowsBlob_NonEmpty) {
  blob->resize(1);
  int value;
  blob->write(&value, 1, 4);
  EXPECT_EQ(5, blob->size());
}

TEST_F(BlobSizeTest, WritingAfterEndOfBlobGrowsBlob_NonEmpty) {
  blob->resize(1);
  int value;
  blob->write(&value, 2, 4);
  EXPECT_EQ(6, blob->size());
}
