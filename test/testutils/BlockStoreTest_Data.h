// This file is meant to be included by BlockStoreTest.h only

struct DataRange {
  constexpr DataRange(size_t blocksize_, off_t offset_, size_t count_): blocksize(blocksize_), offset(offset_), count(count_) {}
  size_t blocksize;
  off_t offset;
  size_t count;
};

class BlockStoreDataParametrizedTest {
public:
  BlockStoreDataParametrizedTest(std::unique_ptr<blockstore::BlockStore> blockStore_, const DataRange &testData_)
    : blockStore(std::move(blockStore_)),
      testData(testData_),
      foregroundData(testData.count), backgroundData(testData.blocksize) {
    DataBlockFixture _foregroundData(testData.count);
    DataBlockFixture _backgroundData(testData.blocksize);
    std::memcpy(foregroundData.data(), _foregroundData.data(), foregroundData.size());
    std::memcpy(backgroundData.data(), _backgroundData.data(), backgroundData.size());
  }

  void TestWriteAndReadImmediately() {
    auto block = blockStore->create(blockstore::Data(testData.blocksize).FillWithZeroes());
    block->write(foregroundData.data(), testData.offset, testData.count);

    EXPECT_DATA_READS_AS(foregroundData, *block, testData.offset, testData.count);
    EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*block, testData.offset, testData.count);
  }

  void TestWriteAndReadAfterLoading() {
    blockstore::Key key = CreateBlockWriteToItAndReturnKey(foregroundData);

    auto loaded_block = blockStore->load(key);
    EXPECT_DATA_READS_AS(foregroundData, *loaded_block, testData.offset, testData.count);
    EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*loaded_block, testData.offset, testData.count);
  }

  void TestOverwriteAndRead() {
    auto block = blockStore->create(blockstore::Data(testData.blocksize));
    block->write(backgroundData.data(), 0, testData.blocksize);
    block->write(foregroundData.data(), testData.offset, testData.count);
    EXPECT_DATA_READS_AS(foregroundData, *block, testData.offset, testData.count);
    EXPECT_DATA_READS_AS_OUTSIDE_OF(backgroundData, *block, testData.offset, testData.count);
  }

private:
  std::unique_ptr<blockstore::BlockStore> blockStore;
  DataRange testData;
  blockstore::Data foregroundData;
  blockstore::Data backgroundData;

  void EXPECT_DATA_EQ(const blockstore::Data &expected, const blockstore::Data &actual) {
    EXPECT_EQ(expected.size(), actual.size());
    EXPECT_EQ(0, std::memcmp(expected.data(), actual.data(), expected.size()));
  }

  blockstore::Key CreateBlockWriteToItAndReturnKey(const blockstore::Data &to_write) {
    auto newblock = blockStore->create(blockstore::Data(testData.blocksize).FillWithZeroes());

    newblock->write(to_write.data(), testData.offset, testData.count);
    return newblock->key();
  }

  void EXPECT_DATA_READS_AS(const blockstore::Data &expected, const blockstore::Block &block, off_t offset, size_t count) {
    blockstore::Data read(count);
    std::memcpy(read.data(), (uint8_t*)block.data() + offset, count);
    EXPECT_DATA_EQ(expected, read);
  }

  void EXPECT_DATA_READS_AS_OUTSIDE_OF(const blockstore::Data &expected, const blockstore::Block &block, off_t start, size_t count) {
    blockstore::Data begin(start);
    blockstore::Data end(testData.blocksize - count - start);

    std::memcpy(begin.data(), expected.data(), start);
    std::memcpy(end.data(), (uint8_t*)expected.data()+start+count, end.size());

    EXPECT_DATA_READS_AS(begin, block, 0, start);
    EXPECT_DATA_READS_AS(end, block, start + count, end.size());
  }

  void EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(const blockstore::Block &block, off_t start, size_t count) {
    blockstore::Data ZEROES(testData.blocksize);
    ZEROES.FillWithZeroes();
    EXPECT_DATA_READS_AS_OUTSIDE_OF(ZEROES, block, start, count);
  }
};
constexpr std::initializer_list<DataRange> DATA_RANGES = {
  DataRange(1024,     0,   1024),     // full size leaf, access beginning to end
  DataRange(1024,     100, 1024-200), // full size leaf, access middle to middle
  DataRange(1024,     0,   1024-100), // full size leaf, access beginning to middle
  DataRange(1024,     100, 1024-100), // full size leaf, access middle to end
  DataRange(1024-100, 0,   1024-100), // non-full size leaf, access beginning to end
  DataRange(1024-100, 100, 1024-300), // non-full size leaf, access middle to middle
  DataRange(1024-100, 0,   1024-200), // non-full size leaf, access beginning to middle
  DataRange(1024-100, 100, 1024-200)  // non-full size leaf, access middle to end
};
#define TYPED_TEST_P_FOR_ALL_DATA_RANGES(TestName)                                   \
  TYPED_TEST_P(BlockStoreTest, TestName) {                                           \
    for (auto dataRange: DATA_RANGES) {                                              \
      BlockStoreDataParametrizedTest(this->fixture.createBlockStore(), dataRange)    \
        .Test##TestName();                                                           \
    }                                                                                \
  }

TYPED_TEST_P_FOR_ALL_DATA_RANGES(WriteAndReadImmediately);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(WriteAndReadAfterLoading);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteAndRead);
