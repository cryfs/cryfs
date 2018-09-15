#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_TESTUTILS_BLOCKSTORETEST_DATA_H_
#define MESSMER_BLOCKSTORE_TEST_TESTUTILS_BLOCKSTORETEST_DATA_H_

// This file is meant to be included by BlockStoreTest.h only

struct DataRange {
  uint64_t blocksize;
  uint64_t offset;
  uint64_t count;
};

class BlockStoreDataParametrizedTest {
public:
  BlockStoreDataParametrizedTest(cpputils::unique_ref<blockstore::BlockStore> blockStore_, const DataRange &testData_)
    : blockStore(std::move(blockStore_)),
      testData(testData_),
      foregroundData(cpputils::DataFixture::generate(testData.count, 0)),
      backgroundData(cpputils::DataFixture::generate(testData.blocksize, 1)) {
  }

  void TestWriteAndReadImmediately() {
    auto block = blockStore->create(cpputils::Data(testData.blocksize).FillWithZeroes());
    block->write(foregroundData.data(), testData.offset, testData.count);

    EXPECT_DATA_READS_AS(foregroundData, *block, testData.offset, testData.count);
    EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*block, testData.offset, testData.count);
  }

  void TestWriteAndReadAfterLoading() {
    blockstore::BlockId blockId = CreateBlockWriteToItAndReturnKey(foregroundData);

    auto loaded_block = blockStore->load(blockId).value();
    EXPECT_DATA_READS_AS(foregroundData, *loaded_block, testData.offset, testData.count);
    EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(*loaded_block, testData.offset, testData.count);
  }

  void TestWriteTwiceAndRead() {
    auto block = blockStore->create(cpputils::Data(testData.blocksize));
    block->write(backgroundData.data(), 0, testData.blocksize);
    block->write(foregroundData.data(), testData.offset, testData.count);
    EXPECT_DATA_READS_AS(foregroundData, *block, testData.offset, testData.count);
    EXPECT_DATA_READS_AS_OUTSIDE_OF(backgroundData, *block, testData.offset, testData.count);
  }

  void TestOverwriteSameSizeAndReadImmediately() {
    auto blockId = blockStore->create(cpputils::Data(testData.blocksize))->blockId();
    auto block = blockStore->overwrite(blockId, backgroundData.copy());
    EXPECT_EQ(testData.blocksize, block->size());
    EXPECT_DATA_READS_AS(backgroundData, *block, 0, testData.blocksize);
  }

  void TestOverwriteSameSizeAndReadAfterLoading() {
    auto blockId = blockStore->create(cpputils::Data(testData.blocksize))->blockId();
    blockStore->overwrite(blockId, backgroundData.copy());
    auto block = blockStore->load(blockId).value();
    EXPECT_EQ(testData.blocksize, block->size());
    EXPECT_DATA_READS_AS(backgroundData, *block, 0, testData.blocksize);
  }

  void TestOverwriteSmallerSizeAndReadImmediately() {
    auto blockId = blockStore->create(cpputils::Data(testData.blocksize))->blockId();
    auto block = blockStore->overwrite(blockId, foregroundData.copy());
    EXPECT_EQ(testData.count, block->size());
    EXPECT_DATA_READS_AS(foregroundData, *block, 0, testData.count);
  }

  void TestOverwriteSmallerSizeAndReadAfterLoading() {
    auto blockId = blockStore->create(cpputils::Data(testData.blocksize))->blockId();
    blockStore->overwrite(blockId, foregroundData.copy());
    auto block = blockStore->load(blockId).value();
    EXPECT_EQ(testData.count, block->size());
    EXPECT_DATA_READS_AS(foregroundData, *block, 0, testData.count);
  }

  void TestOverwriteLargerSizeAndReadImmediately() {
    auto blockId = blockStore->create(cpputils::Data(testData.count))->blockId();
    auto block = blockStore->overwrite(blockId, backgroundData.copy());
    EXPECT_EQ(testData.blocksize, block->size());
    EXPECT_DATA_READS_AS(backgroundData, *block, 0, testData.blocksize);
  }

  void TestOverwriteLargerSizeAndReadAfterLoading() {
    auto blockId = blockStore->create(cpputils::Data(testData.count))->blockId();
    blockStore->overwrite(blockId, backgroundData.copy());
    auto block = blockStore->load(blockId).value();
    EXPECT_EQ(testData.blocksize, block->size());
    EXPECT_DATA_READS_AS(backgroundData, *block, 0, testData.blocksize);
  }

  void TestOverwriteNonexistingAndReadImmediately() {
    auto blockId = blockStore->createBlockId();
    auto block = blockStore->overwrite(blockId, backgroundData.copy());
    EXPECT_EQ(testData.blocksize, block->size());
    EXPECT_DATA_READS_AS(backgroundData, *block, 0, testData.blocksize);
  }

  void TestOverwriteNonexistingAndReadAfterLoading() {
    auto blockId = blockStore->createBlockId();
    blockStore->overwrite(blockId, backgroundData.copy());
    auto block = blockStore->load(blockId).value();
    EXPECT_EQ(testData.blocksize, block->size());
    EXPECT_DATA_READS_AS(backgroundData, *block, 0, testData.blocksize);
  }

private:
  cpputils::unique_ref<blockstore::BlockStore> blockStore;
  DataRange testData;
  cpputils::Data foregroundData;
  cpputils::Data backgroundData;

  blockstore::BlockId CreateBlockWriteToItAndReturnKey(const cpputils::Data &to_write) {
    auto newblock = blockStore->create(cpputils::Data(testData.blocksize).FillWithZeroes());

    newblock->write(to_write.data(), testData.offset, testData.count);
    return newblock->blockId();
  }

  void EXPECT_DATA_READS_AS(const cpputils::Data &expected, const blockstore::Block &block, uint64_t offset, uint64_t count) {
    cpputils::Data read(count);
    std::memcpy(read.data(), static_cast<const uint8_t*>(block.data()) + offset, count);
    EXPECT_EQ(expected, read);
  }

  void EXPECT_DATA_READS_AS_OUTSIDE_OF(const cpputils::Data &expected, const blockstore::Block &block, uint64_t start, uint64_t count) {
    cpputils::Data begin(start);
    cpputils::Data end(testData.blocksize - count - start);

    std::memcpy(begin.data(), expected.data(), start);
    std::memcpy(end.data(), expected.dataOffset(start+count), end.size());

    EXPECT_DATA_READS_AS(begin, block, 0, start);
    EXPECT_DATA_READS_AS(end, block, start + count, end.size());
  }

  void EXPECT_DATA_IS_ZEROES_OUTSIDE_OF(const blockstore::Block &block, uint64_t start, uint64_t count) {
    cpputils::Data ZEROES(testData.blocksize);
    ZEROES.FillWithZeroes();
    EXPECT_DATA_READS_AS_OUTSIDE_OF(ZEROES, block, start, count);
  }
};

inline std::vector<DataRange> DATA_RANGES() {
  return {
          DataRange{1024, 0, 1024},     // full size leaf, access beginning to end
          DataRange{1024, 100, 1024 - 200}, // full size leaf, access middle to middle
          DataRange{1024, 0, 1024 - 100}, // full size leaf, access beginning to middle
          DataRange{1024, 100, 1024 - 100}, // full size leaf, access middle to end
          DataRange{1024 - 100, 0, 1024 - 100}, // non-full size leaf, access beginning to end
          DataRange{1024 - 100, 100, 1024 - 300}, // non-full size leaf, access middle to middle
          DataRange{1024 - 100, 0, 1024 - 200}, // non-full size leaf, access beginning to middle
          DataRange{1024 - 100, 100, 1024 - 200}  // non-full size leaf, access middle to end
  };
};
#define TYPED_TEST_P_FOR_ALL_DATA_RANGES(TestName)                                   \
  TYPED_TEST_P(BlockStoreTest, TestName) {                                           \
    for (auto dataRange: DATA_RANGES()) {                                            \
      BlockStoreDataParametrizedTest(this->fixture.createBlockStore(), dataRange)    \
        .Test##TestName();                                                           \
    }                                                                                \
  }

TYPED_TEST_P_FOR_ALL_DATA_RANGES(WriteAndReadImmediately);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(WriteAndReadAfterLoading);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(WriteTwiceAndRead);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteSameSizeAndReadImmediately);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteSameSizeAndReadAfterLoading);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteSmallerSizeAndReadImmediately);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteSmallerSizeAndReadAfterLoading);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteLargerSizeAndReadImmediately);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteLargerSizeAndReadAfterLoading);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteNonexistingAndReadImmediately);
TYPED_TEST_P_FOR_ALL_DATA_RANGES(OverwriteNonexistingAndReadAfterLoading);

#endif
