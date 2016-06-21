#include <gtest/gtest.h>
#include "blockstore/implementations/versioncounting/VersionCountingBlockStore.h"
#include "blockstore/implementations/testfake/FakeBlockStore.h"
#include "blockstore/utils/BlockStoreUtils.h"
#include <cpp-utils/data/DataFixture.h>

using ::testing::Test;

using cpputils::DataFixture;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

using blockstore::testfake::FakeBlockStore;

using namespace blockstore::versioncounting;

class VersionCountingBlockStoreTest: public Test {
public:
  static constexpr unsigned int BLOCKSIZE = 1024;
  VersionCountingBlockStoreTest():
    baseBlockStore(new FakeBlockStore),
    blockStore(make_unique_ref<VersionCountingBlockStore>(std::move(cpputils::nullcheck(std::unique_ptr<FakeBlockStore>(baseBlockStore)).value()), KnownBlockVersions())),
    data(DataFixture::generate(BLOCKSIZE)) {
  }
  FakeBlockStore *baseBlockStore;
  unique_ref<VersionCountingBlockStore> blockStore;
  Data data;

  blockstore::Key CreateBlockReturnKey() {
    return CreateBlockReturnKey(data);
  }

  blockstore::Key CreateBlockReturnKey(const Data &initData) {
    return blockStore->create(initData)->key();
  }

  Data loadBaseBlock(const blockstore::Key &key) {
    auto block = baseBlockStore->load(key).value();
    Data result(block->size());
    std::memcpy(result.data(), block->data(), data.size());
    return result;
  }

  void modifyBlock(const blockstore::Key &key) {
    auto block = blockStore->load(key).value();
    uint64_t data = 5;
    block->write(&data, 0, sizeof(data));
  }

  void rollbackBaseBlock(const blockstore::Key &key, const Data &data) {
    auto block = baseBlockStore->load(key).value();
    block->resize(data.size());
    block->write(data.data(), 0, data.size());
  }

private:
  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlockStoreTest);
};

TEST_F(VersionCountingBlockStoreTest, DoesntAllowRollbacks) {
  auto key = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(key);
  modifyBlock(key);
  rollbackBaseBlock(key, oldBaseBlock);
  EXPECT_EQ(boost::none, blockStore->load(key));
}

TEST_F(VersionCountingBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(0));
}

TEST_F(VersionCountingBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto key = CreateBlockReturnKey(Data(0));
  auto base = baseBlockStore->load(key).value();
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(base->size()));
}

TEST_F(VersionCountingBlockStoreTest, PhysicalBlockSize_negativeboundaries) {
  // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
  // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
  auto physicalSizeForVirtualSizeZero = baseBlockStore->load(CreateBlockReturnKey(Data(0))).value()->size();
  if (physicalSizeForVirtualSizeZero > 0) {
    EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
  }
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
  EXPECT_EQ(1u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(VersionCountingBlockStoreTest, PhysicalBlockSize_positive) {
  auto key = CreateBlockReturnKey(Data(10*1024));
  auto base = baseBlockStore->load(key).value();
  EXPECT_EQ(10*1024u, blockStore->blockSizeFromPhysicalBlockSize(base->size()));
}
