#include <gtest/gtest.h>
#include "blockstore/implementations/parallelaccess/ParallelAccessBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"

using ::testing::Test;

using cpputils::Data;

using blockstore::testfake::FakeBlockStore;

using namespace blockstore::parallelaccess;

class ParallelAccessBlockStoreTest: public Test {
public:
    ParallelAccessBlockStoreTest():
            baseBlockStore(new FakeBlockStore),
            blockStore(std::move(cpputils::nullcheck(std::unique_ptr<FakeBlockStore>(baseBlockStore)).value()))  {
    }
    FakeBlockStore *baseBlockStore;
    ParallelAccessBlockStore blockStore;

    blockstore::BlockId CreateBlockReturnKey(const Data &initData) {
        return blockStore.create(initData)->blockId();
    }
};

TEST_F(ParallelAccessBlockStoreTest, PhysicalBlockSize_zerophysical) {
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(0));
}

TEST_F(ParallelAccessBlockStoreTest, PhysicalBlockSize_zerovirtual) {
    auto blockId = CreateBlockReturnKey(Data(0));
    auto base = baseBlockStore->load(blockId).value();
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(base->size()));
}

TEST_F(ParallelAccessBlockStoreTest, PhysicalBlockSize_negativeboundaries) {
    // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
    // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
    auto physicalSizeForVirtualSizeZero = baseBlockStore->load(CreateBlockReturnKey(Data(0))).value()->size();
    if (physicalSizeForVirtualSizeZero > 0) {
        EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
    }
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
    EXPECT_EQ(1u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(ParallelAccessBlockStoreTest, PhysicalBlockSize_positive) {
    auto blockId = CreateBlockReturnKey(Data(10*1024));
    auto base = baseBlockStore->load(blockId).value();
    EXPECT_EQ(10*1024u, blockStore.blockSizeFromPhysicalBlockSize(base->size()));
}
