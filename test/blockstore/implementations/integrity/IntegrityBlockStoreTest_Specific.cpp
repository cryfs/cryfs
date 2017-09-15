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
    blockStore(make_unique_ref<IntegrityBlockStore2>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), stateFile.path(), myClientId, false)),
    data(DataFixture::generate(BLOCKSIZE)) {
  }
  static constexpr uint32_t myClientId = 0x12345678;
  TempFile stateFile;
  InMemoryBlockStore2 *baseBlockStore;
  unique_ref<IntegrityBlockStore2> blockStore;
  Data data;

  std::pair<InMemoryBlockStore2 *, unique_ptr<IntegrityBlockStore2>> makeBlockStoreWithDeletionPrevention() {
    InMemoryBlockStore2 *baseBlockStore = new InMemoryBlockStore2;
    auto blockStore = make_unique<IntegrityBlockStore2>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), stateFile.path(), myClientId, true);
    return std::make_pair(baseBlockStore, std::move(blockStore));
  }

  std::pair<InMemoryBlockStore2 *, unique_ptr<IntegrityBlockStore2>> makeBlockStoreWithoutDeletionPrevention() {
    InMemoryBlockStore2 *baseBlockStore = new InMemoryBlockStore2;
    auto blockStore = make_unique<IntegrityBlockStore2>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), stateFile.path(), myClientId, false);
    return std::make_pair(baseBlockStore, std::move(blockStore));
  }

  blockstore::Key CreateBlockReturnKey() {
    return CreateBlockReturnKey(data);
  }

  blockstore::Key CreateBlockReturnKey(const Data &initData) {
    return blockStore->create(initData.copy());
  }

  Data loadBaseBlock(const blockstore::Key &key) {
    return baseBlockStore->load(key).value();
  }

  Data loadBlock(const blockstore::Key &key) {
    return blockStore->load(key).value();
  }

  void modifyBlock(const blockstore::Key &key) {
    auto block = blockStore->load(key).value();
    byte* first_byte = (byte*)block.data();
    *first_byte = *first_byte + 1;
    blockStore->store(key, block);
  }

  void rollbackBaseBlock(const blockstore::Key &key, const Data &data) {
    baseBlockStore->store(key, data);
  }

  void decreaseVersionNumber(const blockstore::Key &key) {
    auto baseBlock = baseBlockStore->load(key).value();
    uint64_t* version = (uint64_t*)((uint8_t*)baseBlock.data()+IntegrityBlockStore2::VERSION_HEADER_OFFSET);
    ASSERT(*version > 1, "Can't decrease the lowest allowed version number");
    *version -= 1;
    baseBlockStore->store(key, baseBlock);
  }

  void increaseVersionNumber(const blockstore::Key &key) {
    auto baseBlock = baseBlockStore->load(key).value();
    uint64_t* version = (uint64_t*)((uint8_t*)baseBlock.data()+IntegrityBlockStore2::VERSION_HEADER_OFFSET);
    *version += 1;
    baseBlockStore->store(key, baseBlock);
  }

  void changeClientId(const blockstore::Key &key) {
    auto baseBlock = baseBlockStore->load(key).value();
    uint32_t* clientId = (uint32_t*)((uint8_t*)baseBlock.data()+IntegrityBlockStore2::CLIENTID_HEADER_OFFSET);
    *clientId += 1;
    baseBlockStore->store(key, baseBlock);
  }

  void deleteBlock(const blockstore::Key &key) {
    blockStore->remove(key);
  }

  void insertBaseBlock(const blockstore::Key &key, Data data) {
    EXPECT_TRUE(baseBlockStore->tryCreate(key, std::move(data)));
  }

private:
  DISALLOW_COPY_AND_ASSIGN(IntegrityBlockStoreTest);
};

constexpr uint32_t IntegrityBlockStoreTest::myClientId;

// Test that a decreasing version number is not allowed
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowDecreasingVersionNumberForSameClient_1) {
  auto key = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(key);
  modifyBlock(key);
  rollbackBaseBlock(key, oldBaseBlock);
  EXPECT_THROW(
      blockStore->load(key),
      IntegrityViolationError
  );
}

TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowDecreasingVersionNumberForSameClient_2) {
  auto key = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(key);
  // Decrease the version number again
  decreaseVersionNumber(key);
  EXPECT_THROW(
          blockStore->load(key),
          IntegrityViolationError
  );
}

// Test that a different client doesn't need to have a higher version number (i.e. version numbers are per client).
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesAllowDecreasingVersionNumberForDifferentClient) {
  auto key = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(key);
  // Fake a modification by a different client with lower version numbers
  changeClientId(key);
  decreaseVersionNumber(key);
  EXPECT_NE(boost::none, blockStore->load(key));
}

// Test that it doesn't allow a rollback to the "newest" block of a client, when this block was superseded by a version of a different client
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowSameVersionNumberForOldClient) {
  auto key = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(key);
  Data oldBaseBlock = loadBaseBlock(key);
  // Fake a modification by a different client with lower version numbers
  changeClientId(key);
  loadBlock(key); // make the block store know about this other client's modification
  // Rollback to old client
  rollbackBaseBlock(key, oldBaseBlock);
  EXPECT_THROW(
          blockStore->load(key),
          IntegrityViolationError
  );
}

// Test that deleted blocks cannot be re-introduced
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_DoesntAllowReintroducingDeletedBlocks) {
  auto key = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(key);
  deleteBlock(key);
  insertBaseBlock(key, std::move(oldBaseBlock));
  EXPECT_THROW(
          blockStore->load(key),
          IntegrityViolationError
  );
}

// This can happen if a client synchronization is delayed. Another client might have won the conflict and pushed a new version for the deleted block.
TEST_F(IntegrityBlockStoreTest, RollbackPrevention_AllowsReintroducingDeletedBlocksWithNewVersionNumber) {
  auto key = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(key);
  deleteBlock(key);
  insertBaseBlock(key, std::move(oldBaseBlock));
  increaseVersionNumber(key);
  EXPECT_NE(boost::none, blockStore->load(key));
}

// Check that in a multi-client scenario, missing blocks are not integrity errors, because another client might have deleted them.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_AllowsDeletingBlocksWhenDeactivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithoutDeletionPrevention();
  auto key = blockStore->create(Data(0));
  baseBlockStore->remove(key);
  EXPECT_EQ(boost::none, blockStore->load(key));
}

// Check that in a single-client scenario, missing blocks are integrity errors.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_DoesntAllowDeletingBlocksWhenActivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithDeletionPrevention();
  auto key = blockStore->create(Data(0));
  baseBlockStore->remove(key);
  EXPECT_THROW(
      blockStore->load(key),
      IntegrityViolationError
  );
}

// Check that in a multi-client scenario, missing blocks are not integrity errors, because another client might have deleted them.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_InForEachBlock_AllowsDeletingBlocksWhenDeactivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithoutDeletionPrevention();
  auto key = blockStore->create(Data(0));
  baseBlockStore->remove(key);
  int count = 0;
  blockStore->forEachBlock([&count] (const blockstore::Key &) {
      ++count;
  });
  EXPECT_EQ(0, count);
}

// Check that in a single-client scenario, missing blocks are integrity errors.
TEST_F(IntegrityBlockStoreTest, DeletionPrevention_InForEachBlock_DoesntAllowDeletingBlocksWhenActivated) {
  InMemoryBlockStore2 *baseBlockStore;
  unique_ptr<IntegrityBlockStore2> blockStore;
  std::tie(baseBlockStore, blockStore) = makeBlockStoreWithDeletionPrevention();
  auto key = blockStore->create(Data(0));
  baseBlockStore->remove(key);
  EXPECT_THROW(
      blockStore->forEachBlock([] (const blockstore::Key &) {}),
      IntegrityViolationError
  );
}

TEST_F(IntegrityBlockStoreTest, LoadingWithDifferentBlockIdFails) {
  auto key = CreateBlockReturnKey();
  blockstore::Key key2 = blockstore::Key::FromString("1491BB4932A389EE14BC7090AC772972");
  baseBlockStore->store(key2, baseBlockStore->load(key).value());
  EXPECT_THROW(
      blockStore->load(key2),
      IntegrityViolationError
  );
}

// TODO Test more integrity cases:
//   - RollbackPrevention_DoesntAllowReintroducingDeletedBlocks with different client id (i.e. trying to re-introduce the newest block of a different client)
//   - RollbackPrevention_AllowsReintroducingDeletedBlocksWithNewVersionNumber with different client id
//   - Think about more...

TEST_F(IntegrityBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(0));
}

TEST_F(IntegrityBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto key = CreateBlockReturnKey(Data(0));
  auto base = baseBlockStore->load(key).value();
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
  auto key = CreateBlockReturnKey(Data(10*1024));
  auto base = baseBlockStore->load(key).value();
  EXPECT_EQ(10*1024u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}
