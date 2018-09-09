#include <gtest/gtest.h>
#include "blockstore/implementations/caching/CachingBlockStore2.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"

using ::testing::Test;

using cpputils::Data;

using blockstore::inmemory::InMemoryBlockStore2;

using namespace blockstore::caching;

class CachingBlockStore2Test: public Test {
public:
  CachingBlockStore2Test():
      baseBlockStore(new InMemoryBlockStore2),
      blockStore(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()))  {
  }
    InMemoryBlockStore2 *baseBlockStore;
  CachingBlockStore2 blockStore;
};

TEST_F(CachingBlockStore2Test, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(0));
}

TEST_F(CachingBlockStore2Test, PhysicalBlockSize_zerovirtual) {
  auto blockId = blockStore.create(Data(0));
  blockStore.flush();
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(base.size()));
}

TEST_F(CachingBlockStore2Test, PhysicalBlockSize_negativeboundaries) {
  // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
  // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
  auto blockId = blockStore.create(Data(0));
  blockStore.flush();
  auto physicalSizeForVirtualSizeZero = baseBlockStore->load(blockId).value().size();
  if (physicalSizeForVirtualSizeZero > 0) {
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
  }
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
  EXPECT_EQ(1u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(CachingBlockStore2Test, PhysicalBlockSize_positive) {
  auto blockId = blockStore.create(Data(10*1024u));
  blockStore.flush();
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(10*1024u, blockStore.blockSizeFromPhysicalBlockSize(base.size()));
}

// TODO Add test cases that flushing the block store doesn't destroy things (i.e. all test cases from BlockStoreTest, but with flushes inbetween)
