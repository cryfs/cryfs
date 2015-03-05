#include "testutils/BlobStoreTest.h"

using std::unique_ptr;

using namespace blobstore;

class BlobResizeTest: public BlobStoreTest {
public:
  BlobResizeTest(): blob(blobStore->create()) {}

  static constexpr uint32_t LARGE_SIZE = 10 * 1024 * 1024;

  unique_ptr<Blob> blob;
};
constexpr uint32_t BlobResizeTest::LARGE_SIZE;

TEST_F(BlobResizeTest, CreatedBlobIsEmpty) {
  EXPECT_EQ(0, blob->size());
}

TEST_F(BlobResizeTest, Growing_1Byte) {
  blob->resize(1);
  EXPECT_EQ(1, blob->size());
}

TEST_F(BlobResizeTest, Growing_Large) {
  blob->resize(LARGE_SIZE);
  EXPECT_EQ(LARGE_SIZE, blob->size());
}

TEST_F(BlobResizeTest, Shrinking_Empty) {
  blob->resize(LARGE_SIZE);
  blob->resize(0);
  EXPECT_EQ(0, blob->size());
}

TEST_F(BlobResizeTest, Shrinking_1Byte) {
  blob->resize(LARGE_SIZE);
  blob->resize(1);
  EXPECT_EQ(1, blob->size());
}

TEST_F(BlobResizeTest, ResizingToItself_Empty) {
  blob->resize(0);
  EXPECT_EQ(0, blob->size());
}

TEST_F(BlobResizeTest, ResizingToItself_1Byte) {
  blob->resize(1);
  blob->resize(1);
  EXPECT_EQ(1, blob->size());
}

TEST_F(BlobResizeTest, ResizingToItself_Large) {
  blob->resize(LARGE_SIZE);
  blob->resize(LARGE_SIZE);
  EXPECT_EQ(LARGE_SIZE, blob->size());
}
