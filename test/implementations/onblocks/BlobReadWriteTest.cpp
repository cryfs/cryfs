#include "testutils/BlobStoreTest.h"
#include <messmer/blockstore/utils/Data.h>
#include "../../testutils/DataBlockFixture.h"
#include "../../../implementations/onblocks/datanodestore/DataNodeView.h"

using std::unique_ptr;
using ::testing::WithParamInterface;
using ::testing::Values;

using namespace blobstore;
using blobstore::onblocks::datanodestore::DataNodeLayout;
using blockstore::Key;
using blockstore::Data;

class BlobReadWriteTest: public BlobStoreTest {
public:
  static constexpr uint32_t LARGE_SIZE = 10 * 1024 * 1024;
  static constexpr DataNodeLayout LAYOUT = DataNodeLayout(BLOCKSIZE_BYTES);

  BlobReadWriteTest()
    :randomData(LARGE_SIZE),
     blob(blobStore->create()) {
  }

  Data readBlob(const Blob &blob) {
    Data data(blob.size());
    blob.read(data.data(), 0, data.size());
    return data;
  }

  template<class DataClass>
  void EXPECT_DATA_READS_AS(const DataClass &expected, const Blob &actual, uint64_t offset, uint64_t size) {
  	Data read(size);
  	actual.read(read.data(), offset, size);
    EXPECT_EQ(0, std::memcmp(expected.data(), read.data(), size));
  }

  DataBlockFixture randomData;
  unique_ptr<Blob> blob;
};
constexpr uint32_t BlobReadWriteTest::LARGE_SIZE;
constexpr DataNodeLayout BlobReadWriteTest::LAYOUT;

TEST_F(BlobReadWriteTest, WritingImmediatelyFlushes_SmallSize) {
	blob->resize(5);
	blob->write(randomData.data(), 0, 5);
	auto loaded = blobStore->load(blob->key());
	EXPECT_DATA_READS_AS(randomData, *loaded, 0, 5);
}

TEST_F(BlobReadWriteTest, WritingImmediatelyFlushes_LargeSize) {
	blob->resize(LARGE_SIZE);
	blob->write(randomData.data(), 0, LARGE_SIZE);
	auto loaded = blobStore->load(blob->key());
	EXPECT_DATA_READS_AS(randomData, *loaded, 0, LARGE_SIZE);
}

struct DataRange {
  DataRange(size_t blobsize_, off_t offset_, size_t count_): blobsize(blobsize_), offset(offset_), count(count_) {}
  size_t blobsize;
  off_t offset;
  size_t count;
};
class BlobReadWriteDataTest: public BlobReadWriteTest, public WithParamInterface<DataRange> {
public:
  DataBlockFixture foregroundData;
  DataBlockFixture backgroundData;

  BlobReadWriteDataTest()
    : foregroundData(GetParam().count),
      backgroundData(GetParam().blobsize) {
      }

  template<class DataClass>
  void EXPECT_DATA_READS_AS_OUTSIDE_OF(const DataClass &expected, const Blob &blob, off_t start, size_t count) {
    Data begin(start);
    Data end(GetParam().blobsize - count - start);

    std::memcpy(begin.data(), expected.data(), start);
    std::memcpy(end.data(), (uint8_t*)expected.data()+start+count, end.size());

    EXPECT_DATA_READS_AS(begin, blob, 0, start);
    EXPECT_DATA_READS_AS(end, blob, start + count, end.size());
  }

  void EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(const Blob &blob, off_t start, size_t count) {
    Data ZEROES(GetParam().blobsize);
    ZEROES.FillWithZeroes();
    EXPECT_DATA_READS_AS_OUTSIDE_OF(ZEROES, blob, start, count);
  }
};
INSTANTIATE_TEST_CASE_P(BlobReadWriteDataTest, BlobReadWriteDataTest, Values(
  //Blob with only one leaf
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf(),     0,   BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()),     // full size leaf, access beginning to end
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf(),     100, BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-200), // full size leaf, access middle to middle
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf(),     0,   BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-100), // full size leaf, access beginning to middle
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf(),     100, BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-100), // full size leaf, access middle to end
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-100, 0,   BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-100), // non-full size leaf, access beginning to end
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-100, 100, BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-300), // non-full size leaf, access middle to middle
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-100, 0,   BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-200), // non-full size leaf, access beginning to middle
  DataRange(BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-100, 100, BlobReadWriteDataTest::LAYOUT.maxBytesPerLeaf()-200),  // non-full size leaf, access middle to end
  //Larger blob
  DataRange(BlobReadWriteDataTest::LARGE_SIZE,     0,   BlobReadWriteDataTest::LARGE_SIZE),     // full size blob, access beginning to end
  DataRange(BlobReadWriteDataTest::LARGE_SIZE,     100, BlobReadWriteDataTest::LARGE_SIZE-200), // full size blob, access middle to middle
  DataRange(BlobReadWriteDataTest::LARGE_SIZE,     0,   BlobReadWriteDataTest::LARGE_SIZE-100), // full size blob, access beginning to middle
  DataRange(BlobReadWriteDataTest::LARGE_SIZE,     100, BlobReadWriteDataTest::LARGE_SIZE-100), // full size blob, access middle to end
  DataRange(BlobReadWriteDataTest::LARGE_SIZE-100, 0,   BlobReadWriteDataTest::LARGE_SIZE-100), // non-full size blob, access beginning to end
  DataRange(BlobReadWriteDataTest::LARGE_SIZE-100, 100, BlobReadWriteDataTest::LARGE_SIZE-300), // non-full size blob, access middle to middle
  DataRange(BlobReadWriteDataTest::LARGE_SIZE-100, 0,   BlobReadWriteDataTest::LARGE_SIZE-200), // non-full size blob, access beginning to middle
  DataRange(BlobReadWriteDataTest::LARGE_SIZE-100, 100, BlobReadWriteDataTest::LARGE_SIZE-200)  // non-full size blob, access middle to end
));

TEST_P(BlobReadWriteDataTest, WriteAndReadImmediately) {
  blob->resize(GetParam().blobsize);
  blob->write(this->foregroundData.data(), GetParam().offset, GetParam().count);

  EXPECT_DATA_READS_AS(this->foregroundData, *blob, GetParam().offset, GetParam().count);
  EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*blob, GetParam().offset, GetParam().count);
}

TEST_P(BlobReadWriteDataTest, WriteAndReadAfterLoading) {
  blob->resize(GetParam().blobsize);
  blob->write(this->foregroundData.data(), GetParam().offset, GetParam().count);
  auto loaded = blobStore->load(blob->key());

  EXPECT_DATA_READS_AS(this->foregroundData, *loaded, GetParam().offset, GetParam().count);
  EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*loaded, GetParam().offset, GetParam().count);
}

TEST_P(BlobReadWriteDataTest, OverwriteAndRead) {
  blob->resize(GetParam().blobsize);
  blob->write(this->backgroundData.data(), 0, GetParam().blobsize);
  blob->write(this->foregroundData.data(), GetParam().offset, GetParam().count);
  EXPECT_DATA_READS_AS(this->foregroundData, *blob, GetParam().offset, GetParam().count);
  EXPECT_DATA_READS_AS_OUTSIDE_OF(this->backgroundData, *blob, GetParam().offset, GetParam().count);
}

TEST_P(BlobReadWriteDataTest, WriteWholeAndReadPart) {
  blob->resize(GetParam().blobsize);
  blob->write(this->backgroundData.data(), 0, GetParam().blobsize);
  Data read(GetParam().count);
  blob->read(read.data(), GetParam().offset, GetParam().count);
  EXPECT_EQ(0, std::memcmp(read.data(), this->backgroundData.data()+GetParam().offset, GetParam().count));
}

TEST_P(BlobReadWriteDataTest, WritePartAndReadWhole) {
  blob->resize(GetParam().blobsize);
  blob->write(this->backgroundData.data(), 0, GetParam().blobsize);
  blob->write(this->foregroundData.data(), GetParam().offset, GetParam().count);
  Data read = readBlob(*blob);
  EXPECT_EQ(0, std::memcmp(read.data(), this->backgroundData.data(), GetParam().offset));
  EXPECT_EQ(0, std::memcmp((uint8_t*)read.data()+GetParam().offset, this->foregroundData.data(), GetParam().count));
  EXPECT_EQ(0, std::memcmp((uint8_t*)read.data()+GetParam().offset+GetParam().count, (uint8_t*)this->backgroundData.data()+GetParam().offset+GetParam().count, GetParam().blobsize-GetParam().count-GetParam().offset));
}
