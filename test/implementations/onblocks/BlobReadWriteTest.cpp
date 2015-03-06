#include "testutils/BlobStoreTest.h"
#include <messmer/blockstore/utils/Data.h>
#include "../../testutils/DataBlockFixture.h"

using std::unique_ptr;

using namespace blobstore;
using blockstore::Key;
using blockstore::Data;

class BlobReadWriteTest: public BlobStoreTest {
public:
  static constexpr uint32_t LARGE_SIZE = 10 * 1024 * 1024;

  BlobReadWriteTest()
    :randomData(LARGE_SIZE),
     blob(blobStore->create()) {
  }

  Data readBlob(const Blob &blob) {
    Data data(blob.size());
    blob.read(data.data(), 0, data.size());
    return data;
  }

  DataBlockFixture randomData;
  unique_ptr<Blob> blob;
};
constexpr uint32_t BlobReadWriteTest::LARGE_SIZE;

TEST_F(BlobReadWriteTest, WritingImmediatelyFlushes_SmallSize) {
	blob->resize(5);
	blob->write(randomData.data(), 0, 5);
	auto loaded = blobStore->load(blob->key());
	EXPECT_EQ(0, std::memcmp(readBlob(*loaded).data(), randomData.data(), 5));
}

TEST_F(BlobReadWriteTest, WritingImmediatelyFlushes_LargeSize) {
	blob->resize(LARGE_SIZE);
	blob->write(randomData.data(), 0, LARGE_SIZE);
	auto loaded = blobStore->load(blob->key());
	EXPECT_EQ(0, std::memcmp(readBlob(*loaded).data(), randomData.data(), LARGE_SIZE));
}

//TODO Test read/write
//TODO Test read/write with loading inbetween
