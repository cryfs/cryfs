#include <gtest/gtest.h>
#include "blockstore/implementations/ondisk/OnDiskBlockStore2.h"
#include <cpp-utils/tempfile/TempDir.h>

using ::testing::Test;

using cpputils::TempDir;
using cpputils::Data;
using std::ifstream;
using blockstore::BlockId;

using namespace blockstore::ondisk;

class OnDiskBlockStoreTest: public Test {
public:
    OnDiskBlockStoreTest():
    baseDir(),
    blockStore(baseDir.path()) {
  }
  TempDir baseDir;
  OnDiskBlockStore2 blockStore;

  blockstore::BlockId CreateBlockReturnKey(const Data &initData) {
    return blockStore.create(initData.copy());
  }

  uint64_t getPhysicalBlockSize(const BlockId &blockId) {
    ifstream stream((baseDir.path() / blockId.ToString().substr(0,3) / blockId.ToString().substr(3)).c_str());
    stream.seekg(0, stream.end);
    return stream.tellg();
  }
};

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(0));
}

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto blockId = CreateBlockReturnKey(Data(0));
  auto baseSize = getPhysicalBlockSize(blockId);
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(baseSize));
}

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_negativeboundaries) {
  // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
  // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
  auto physicalSizeForVirtualSizeZero = getPhysicalBlockSize(CreateBlockReturnKey(Data(0)));
  if (physicalSizeForVirtualSizeZero > 0) {
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
  }
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
  EXPECT_EQ(1u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(OnDiskBlockStoreTest, PhysicalBlockSize_positive) {
  auto blockId = CreateBlockReturnKey(Data(10*1024));
  auto baseSize = getPhysicalBlockSize(blockId);
  EXPECT_EQ(10*1024u, blockStore.blockSizeFromPhysicalBlockSize(baseSize));
}

TEST_F(OnDiskBlockStoreTest, NumBlocksIsCorrectAfterAddingTwoBlocksWithSameKeyPrefix) {
  const BlockId key1 = BlockId::FromString("4CE72ECDD20877A12ADBF4E3927C0A13");
  const BlockId key2 = BlockId::FromString("4CE72ECDD20877A12ADBF4E3927C0A14");
  EXPECT_TRUE(blockStore.tryCreate(key1, cpputils::Data(0)));
  EXPECT_TRUE(blockStore.tryCreate(key2, cpputils::Data(0)));
  EXPECT_EQ(2u, blockStore.numBlocks());
}
