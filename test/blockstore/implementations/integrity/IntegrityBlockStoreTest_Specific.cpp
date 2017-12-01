#include <gtest/gtest.h>
#include "blockstore/implementations/integrity/IntegrityBlockStore2.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "blockstore/utils/BlockStoreUtils.h"
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/tempfile/TempFile.h>
#include "../../testutils/gtest_printers.h"

using ::testing::Test;

using cpputils::DataFixture;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::TempFile;
using boost::none;
using std::make_unique;
using std::unique_ptr;

using blockstore::inmemory::InMemoryBlockStore2;

using namespace blockstore::integrity;

class IntegrityBlockStoreTest: public Test {
public:
  static constexpr unsigned int BLOCKSIZE = 1024;
  IntegrityBlockStoreTest():
    stateFile(false),
    baseBlockStore(new InMemoryBlockStore2),
    blockStore(make_unique_ref<IntegrityBlockStore2>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), stateFile.path(), myClientId, false, false)),
    data(DataFixture::generate(BLOCKSIZE)) {
  }
  static constexpr uint32_t myClientId = 0x12345678;
  TempFile stateFile;
  InMemoryBlockStore2 *baseBlockStore;
  unique_ref<IntegrityBlockStore2> blockStore;
  Data data;

  std::pair<InMemoryBlockStore2 *, unique_ptr<IntegrityBlockStore2>> makeBlockStoreWithDeletionPrevention() {
    InMemoryBlockStore2 *baseBlockStore = new InMemoryBlockStore2;
    auto blockStore = make_unique<IntegrityBlockStore2>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), stateFile.path(), myClientId, false, true);
    return std::make_pair(baseBlockStore, std::move(blockStore));
  }

  std::pair<InMemoryBlockStore2 *, unique_ptr<IntegrityBlockStore2>> makeBlockStoreWithoutDeletionPrevention() {
    InMemoryBlockStore2 *baseBlockStore = new InMemoryBlockStore2;
    auto blockStore = make_unique<IntegrityBlockStore2>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), stateFile.path(), myClientId, false, false);
    return std::make_pair(baseBlockStore, std::move(blockStore));
  }

  blockstore::BlockId CreateBlockReturnKey() {
    return CreateBlockReturnKey(data);
  }

  blockstore::BlockId CreateBlockReturnKey(const Data &initData) {
    return blockStore->create(initData.copy());
  }

  Data loadBaseBlock(const blockstore::BlockId &blockId) {
    return baseBlockStore->load(blockId).value();
  }

  Data loadBlock(const blockstore::BlockId &blockId) {
    return blockStore->load(blockId).value();
  }

  void modifyBlock(const blockstore::BlockId &blockId) {
    auto block = blockStore->load(blockId).value();
    byte* first_byte = (byte*)block.data();
    *first_byte = *first_byte + 1;
    blockStore->store(blockId, block);
  }

  void rollbackBaseBlock(const blockstore::BlockId &blockId, const Data &data) {
    baseBlockStore->store(blockId, data);
  }

  void decreaseVersionNumber(const blockstore::BlockId &blockId) {
    auto baseBlock = baseBlockStore->load(blockId).value();
    uint64_t* version = (uint64_t*)((uint8_t*)baseBlock.data()+IntegrityBlockStore2::VERSION_HEADER_OFFSET);
    ASSERT(*version > 1, "Can't decrease the lowest allowed version number");
    *version -= 1;
    baseBlockStore->store(blockId, baseBlock);
  }

  void increaseVersionNumber(const blockstore::BlockId &blockId) {
    auto baseBlock = baseBlockStore->load(blockId).value();
    uint64_t* version = (uint64_t*)((uint8_t*)baseBlock.data()+IntegrityBlockStore2::VERSION_HEADER_OFFSET);
    *version += 1;
    baseBlockStore->store(blockId, baseBlock);
  }

  void changeClientId(const blockstore::BlockId &blockId) {
    auto baseBlock = baseBlockStore->load(blockId).value();
    uint32_t* clientId = (uint32_t*)((uint8_t*)baseBlock.data()+IntegrityBlockStore2::CLIENTID_HEADER_OFFSET);
    *clientId += 1;
    baseBlockStore->store(blockId, baseBlock);
  }

  void deleteBlock(const blockstore::BlockId &blockId) {
    blockStore->remove(blockId);
  }

  void insertBaseBlock(const blockstore::BlockId &blockId, Data data) {
    EXPECT_TRUE(baseBlockStore->tryCreate(blockId, data));
  }

private:
  DISALLOW_COPY_AND_ASSIGN(IntegrityBlockStoreTest);
};

constexpr uint32_t IntegrityBlockStoreTest::myClientId;

// Test that a decreasing version number is not allowed
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowDecreasingVersionNumberForSameClient_1) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  modifyBlock(blockId);
  rollbackBaseBlock(blockId, oldBaseBlock);
  EXPECT_THROW(
      blockStore->load(blockId),
      IntegrityViolationError
  );
}

TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowDecreasingVersionNumberForSameClient_2) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  // Decrease the version number again
  decreaseVersionNumber(blockId);
  EXPECT_THROW(
          blockStore->load(blockId),
          IntegrityViolationError
  );
}

// Test that a different client doesn't need to have a higher version number (i.e. version numbers are per client).
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesAllowDecreasingVersionNumberForDifferentClient) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  // Fake a modification by a different client with lower version numbers
  changeClientId(blockId);
  decreaseVersionNumber(blockId);
  EXPECT_NE(boost::none, blockStore->load(blockId));
}

// Test that it doesn't allow a rollback to the "newest" block of a client, when this block was superseded by a version of a different client
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowSameVersionNumberForOldClient) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  Data oldBaseBlock = loadBaseBlock(blockId);
  // Fake a modification by a different client with lower version numbers
  changeClientId(blockId);
  loadBlock(blockId); // make the block store know about this other client's modification
  // Rollback to old client
  rollbackBaseBlock(blockId, oldBaseBlock);
  EXPECT_THROW(
          blockStore->load(blockId),
          IntegrityViolationError
  );
}

// Test that deleted blocks cannot be re-introduced
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowReintroducingDeletedBlocks) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  deleteBlock(blockId);
  insertBaseBlock(blockId, std::move(oldBaseBlock));
  EXPECT_THROW(
          blockStore->load(blockId),
          IntegrityViolationError
  );
}

// This can happen if a client synchronization is delayed. Another client might have won the conflict and pushed a new version for the deleted block.
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_AllowsReintroducingDeletedBlocksWithNewVersionNumber) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  deleteBlock(blockId);
  insertBaseBlock(blockId, std::move(oldBaseBlock));
  increaseVersionNumber(blockId);
  EXPECT_NE(boost::none, blockStore->load(blockId));
}

// Check that in a multi-client scenario, missing blocks are not integrity errors, because another client might have deleted them.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_AllowsDeletingBlocksWhenDeactivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithoutDeletionPrevention();
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
}

// Check that in a single-client scenario, missing blocks are integrity errors.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_DoesntAllowDeletingBlocksWhenActivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithDeletionPrevention();
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  EXPECT_THROW(
      blockStore->load(blockId),
      IntegrityViolationError
  );
}

// Check that in a multi-client scenario, missing blocks are not integrity errors, because another client might have deleted them.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_InForEachBlock_AllowsDeletingBlocksWhenDeactivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithoutDeletionPrevention();
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  int count = 0;
  blockStore->forEachBlock([&count] (const blockstore::BlockId &) {
      ++count;
  });
  EXPECT_EQ(0, count);
}

// Check that in a single-client scenario, missing blocks are integrity errors.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_InForEachBlock_DoesntAllowDeletingBlocksWhenActivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithDeletionPrevention();
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  EXPECT_THROW(
      blockStore->forEachBlock([] (const blockstore::BlockId &) {}),
      IntegrityViolationError
  );
}

TEST_F(IntegrityBlockStoreTest, LoadingWithDifferentBlockIdFails) {
  auto blockId = CreateBlockReturnKey();
  blockstore::BlockId key2 = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  baseBlockStore->store(key2, baseBlockStore->load(blockId).value());
  EXPECT_THROW(
      blockStore->load(key2),
      IntegrityViolationError
  );
}

// TODO Test more integrity cases:
//   - RollbackPrevention_DoesntAllowReintroducingDeletedBlocks with different client id (i.e. trying to re-introduce the newest block of a different client)
//   - RollbackPrevention_AllowsReintroducingDeletedBlocksWithNewVersionNumber with different client id
//   - Think about more...
// TODO Test that disabling integrity checks allows all these cases

TEST_F(IntegrityBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(0));
}

TEST_F(IntegrityBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto blockId = CreateBlockReturnKey(Data(0));
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}

TEST_F(IntegrityBlockStoreTest, PhysicalBlockSize_negativeboundaries) {
  // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
  // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
  auto physicalSizeForVirtualSizeZero = baseBlockStore->load(CreateBlockReturnKey(Data(0))).value().size();
  if (physicalSizeForVirtualSizeZero > 0) {
    EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
  }
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
  EXPECT_EQ(1u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(IntegrityBlockStoreTest, PhysicalBlockSize_positive) {
  auto blockId = CreateBlockReturnKey(Data(10*1024));
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(10*1024u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}
