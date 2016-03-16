#include <gtest/gtest.h>
#include "blockstore/implementations/caching/CachingBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"

using ::testing::Test;

using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

using blockstore::testfake::FakeBlockStore;

using namespace blockstore::caching;

class CachingBlockStoreTest: public Test {
public:
    CachingBlockStoreTest():
            baseBlockStore(new FakeBlockStore),
            blockStore(std::move(cpputils::nullcheck(std::unique_ptr<FakeBlockStore>(baseBlockStore)).value()))  {
    }
    FakeBlockStore *baseBlockStore;
    CachingBlockStore blockStore;

    blockstore::Key CreateBlockReturnKey(const Data &initData) {
        auto block = blockStore.create(initData);
        block->flush();
        return block->key();
    }
};

TEST_F(CachingBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(0));
}

TEST_F(CachingBlockStoreTest, PhysicalBlockSize_zerovirtual) {
    auto key = CreateBlockReturnKey(Data(0));
    auto base = baseBlockStore->load(key).value();
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(base->size()));
}

TEST_F(CachingBlockStoreTest, PhysicalBlockSize_negativeboundaries) {
    // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
    // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
    auto physicalSizeForVirtualSizeZero = baseBlockStore->load(CreateBlockReturnKey(Data(0))).value()->size();
    if (physicalSizeForVirtualSizeZero > 0) {
        EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
    }
    EXPECT_EQ(0u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
    EXPECT_EQ(1u, blockStore.blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(CachingBlockStoreTest, PhysicalBlockSize_positive) {
    auto key = CreateBlockReturnKey(Data(10*1024));
    auto base = baseBlockStore->load(key).value();
    EXPECT_EQ(10*1024u, blockStore.blockSizeFromPhysicalBlockSize(base->size()));
}
