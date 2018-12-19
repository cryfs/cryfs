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
using cpputils::serialize;
using cpputils::deserialize;
using boost::none;
using std::unique_ptr;

using blockstore::inmemory::InMemoryBlockStore2;

using namespace blockstore::integrity;

namespace {
class FakeCallback final {
public:
  FakeCallback(): wasCalled_(false) {}

  bool wasCalled() const {
    return wasCalled_;
  }

  std::function<void ()> callback() {
    return [this] () {
      wasCalled_ = true;
    };
  }

private:
  bool wasCalled_;
};
}

template<bool AllowIntegrityViolations, bool MissingBlockIsIntegrityViolation>
class IntegrityBlockStoreTest: public Test {
public:
  static constexpr unsigned int BLOCKSIZE = 1024;
  IntegrityBlockStoreTest():
    stateFile(false),
    onIntegrityViolation(),
    baseBlockStore(new InMemoryBlockStore2),
    blockStore(make_unique_ref<IntegrityBlockStore2>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), stateFile.path(), myClientId, AllowIntegrityViolations, MissingBlockIsIntegrityViolation, onIntegrityViolation.callback())),
    data(DataFixture::generate(BLOCKSIZE)) {
  }
  static constexpr uint32_t myClientId = 0x12345678;
  TempFile stateFile;
  FakeCallback onIntegrityViolation;
  InMemoryBlockStore2 *baseBlockStore;
  unique_ref<IntegrityBlockStore2> blockStore;
  Data data;

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
    CryptoPP::byte* first_byte = static_cast<CryptoPP::byte*>(block.data());
    *first_byte = *first_byte + 1;
    blockStore->store(blockId, block);
  }

  void rollbackBaseBlock(const blockstore::BlockId &blockId, const Data &data) {
    baseBlockStore->store(blockId, data);
  }

  void decreaseVersionNumber(const blockstore::BlockId &blockId) {
    auto baseBlock = baseBlockStore->load(blockId).value();
    void* versionPtr = static_cast<uint8_t*>(baseBlock.data()) + IntegrityBlockStore2::VERSION_HEADER_OFFSET;
    uint64_t version = deserialize<uint64_t>(versionPtr);
    ASSERT(version > 1, "Can't decrease the lowest allowed version number");
    serialize<uint64_t>(versionPtr, version-1);
    baseBlockStore->store(blockId, baseBlock);
  }

  void increaseVersionNumber(const blockstore::BlockId &blockId) {
    auto baseBlock = baseBlockStore->load(blockId).value();
    void* versionPtr = static_cast<uint8_t*>(baseBlock.data()) + IntegrityBlockStore2::VERSION_HEADER_OFFSET;
    uint64_t version = deserialize<uint64_t>(versionPtr);
    serialize<uint64_t>(versionPtr, version+1);
    baseBlockStore->store(blockId, baseBlock);
  }

  void changeClientId(const blockstore::BlockId &blockId) {
    auto baseBlock = baseBlockStore->load(blockId).value();
    void* clientIdPtr = static_cast<uint8_t*>(baseBlock.data()) + IntegrityBlockStore2::CLIENTID_HEADER_OFFSET;
    uint64_t clientId = deserialize<uint64_t>(clientIdPtr);
    serialize<uint64_t>(clientIdPtr, clientId+1);
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

using IntegrityBlockStoreTest_Default = IntegrityBlockStoreTest<false, false>;
using IntegrityBlockStoreTest_MissingBlockIsIntegrityViolation = IntegrityBlockStoreTest<false, true>;
using IntegrityBlockStoreTest_AllowIntegrityViolations = IntegrityBlockStoreTest<true, false>;
using IntegrityBlockStoreTest_AllowIntegrityViolations_MissingBlockIsIntegrityViolation = IntegrityBlockStoreTest<true, true>;

template<bool AllowIntegrityViolations, bool MissingBlockIsIntegrityViolation>
constexpr uint32_t IntegrityBlockStoreTest<AllowIntegrityViolations, MissingBlockIsIntegrityViolation>::myClientId;

// Test that a decreasing version number is not allowed
TEST_F(IntegrityBlockStoreTest_Default, RollbackPrevention_DoesntAllowDecreasingVersionNumberForSameClient_1) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  modifyBlock(blockId);
  rollbackBaseBlock(blockId, oldBaseBlock);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_TRUE(onIntegrityViolation.wasCalled());
}

// Test that a decreasing version number is allowed if allowIntegrityViolations is set.
TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, RollbackPrevention_AllowsDecreasingVersionNumberForSameClient_1) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  modifyBlock(blockId);
  rollbackBaseBlock(blockId, oldBaseBlock);
  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_Default, RollbackPrevention_DoesntAllowDecreasingVersionNumberForSameClient_2) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  // Decrease the version number again
  decreaseVersionNumber(blockId);

  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_TRUE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, RollbackPrevention_AllowsDecreasingVersionNumberForSameClient_2) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  // Decrease the version number again
  decreaseVersionNumber(blockId);

  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// Test that a different client doesn't need to have a higher version number (i.e. version numbers are per client).
TEST_F(IntegrityBlockStoreTest_Default, RollbackPrevention_DoesAllowDecreasingVersionNumberForDifferentClient) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  // Fake a modification by a different client with lower version numbers
  changeClientId(blockId);
  decreaseVersionNumber(blockId);
  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, RollbackPrevention_DoesAllowDecreasingVersionNumberForDifferentClient) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  // Fake a modification by a different client with lower version numbers
  changeClientId(blockId);
  decreaseVersionNumber(blockId);
  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// Test that it doesn't allow a rollback to the "newest" block of a client, when this block was superseded by a version of a different client
TEST_F(IntegrityBlockStoreTest_Default, RollbackPrevention_DoesntAllowSameVersionNumberForOldClient) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  Data oldBaseBlock = loadBaseBlock(blockId);
  // Fake a modification by a different client with lower version numbers
  changeClientId(blockId);
  loadBlock(blockId); // make the block store know about this other client's modification
  // Rollback to old client
  rollbackBaseBlock(blockId, oldBaseBlock);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_TRUE(onIntegrityViolation.wasCalled());
}

// Test that it does allow a rollback to the "newest" block of a client, when this block was superseded by a version of a different client, but integrity violations are allowed
TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, RollbackPrevention_AllowsSameVersionNumberForOldClient) {
  auto blockId = CreateBlockReturnKey();
  // Increase the version number
  modifyBlock(blockId);
  Data oldBaseBlock = loadBaseBlock(blockId);
  // Fake a modification by a different client with lower version numbers
  changeClientId(blockId);
  loadBlock(blockId); // make the block store know about this other client's modification
  // Rollback to old client
  rollbackBaseBlock(blockId, oldBaseBlock);
  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// Test that deleted blocks cannot be re-introduced
TEST_F(IntegrityBlockStoreTest_Default, RollbackPrevention_DoesntAllowReintroducingDeletedBlocks) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  deleteBlock(blockId);
  insertBaseBlock(blockId, std::move(oldBaseBlock));
  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_TRUE(onIntegrityViolation.wasCalled());
}

// Test that deleted blocks can be re-introduced if integrity violations are allowed
TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, RollbackPrevention_AllowsReintroducingDeletedBlocks) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  deleteBlock(blockId);
  insertBaseBlock(blockId, std::move(oldBaseBlock));
  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// This can happen if a client synchronization is delayed. Another client might have won the conflict and pushed a new version for the deleted block.
TEST_F(IntegrityBlockStoreTest_Default, RollbackPrevention_AllowsReintroducingDeletedBlocksWithNewVersionNumber) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  deleteBlock(blockId);
  insertBaseBlock(blockId, std::move(oldBaseBlock));
  increaseVersionNumber(blockId);
  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, RollbackPrevention_AllowsReintroducingDeletedBlocksWithNewVersionNumber) {
  auto blockId = CreateBlockReturnKey();
  Data oldBaseBlock = loadBaseBlock(blockId);
  deleteBlock(blockId);
  insertBaseBlock(blockId, std::move(oldBaseBlock));
  increaseVersionNumber(blockId);
  EXPECT_NE(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// Check that in a multi-client scenario, missing blocks are not integrity errors, because another client might have deleted them.
TEST_F(IntegrityBlockStoreTest_Default, DeletionPrevention_AllowsDeletingBlocksWhenDeactivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, DeletionPrevention_AllowsDeletingBlocksWhenDeactivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// Check that in a single-client scenario, missing blocks are integrity errors.
TEST_F(IntegrityBlockStoreTest_MissingBlockIsIntegrityViolation, DeletionPrevention_DoesntAllowDeletingBlocksWhenActivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_TRUE(onIntegrityViolation.wasCalled());
}

// Check that in a single-client scenario, missing blocks don't throw if integrity violations are allowed.
TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations_MissingBlockIsIntegrityViolation, DeletionPrevention_AllowsDeletingBlocksWhenActivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  EXPECT_EQ(boost::none, blockStore->load(blockId));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// Check that in a multi-client scenario, missing blocks are not integrity errors, because another client might have deleted them.
TEST_F(IntegrityBlockStoreTest_Default, DeletionPrevention_InForEachBlock_AllowsDeletingBlocksWhenDeactivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  int count = 0;
  blockStore->forEachBlock([&count] (const blockstore::BlockId &) {
      ++count;
  });
  EXPECT_EQ(0, count);
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, DeletionPrevention_InForEachBlock_AllowsDeletingBlocksWhenDeactivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  int count = 0;
  blockStore->forEachBlock([&count] (const blockstore::BlockId &) {
    ++count;
  });
  EXPECT_EQ(0, count);
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// Check that in a single-client scenario, missing blocks are integrity errors.
TEST_F(IntegrityBlockStoreTest_MissingBlockIsIntegrityViolation, DeletionPrevention_InForEachBlock_DoesntAllowDeletingBlocksWhenActivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  blockStore->forEachBlock([] (const blockstore::BlockId &) {});
  EXPECT_TRUE(onIntegrityViolation.wasCalled());
}

// Check that in a single-client scenario, missing blocks don't throw if integrity violations are allowed.
TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations_MissingBlockIsIntegrityViolation, DeletionPrevention_InForEachBlock_AllowsDeletingBlocksWhenActivated) {
  auto blockId = blockStore->create(Data(0));
  baseBlockStore->remove(blockId);
  blockStore->forEachBlock([] (const blockstore::BlockId &) {});
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_Default, LoadingWithDifferentBlockIdFails) {
  auto blockId = CreateBlockReturnKey();
  blockstore::BlockId key2 = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  baseBlockStore->store(key2, baseBlockStore->load(blockId).value());
  EXPECT_EQ(boost::none, blockStore->load(key2));
  EXPECT_TRUE(onIntegrityViolation.wasCalled());
}

TEST_F(IntegrityBlockStoreTest_AllowIntegrityViolations, LoadingWithDifferentBlockIdDoesntFail) {
  auto blockId = CreateBlockReturnKey();
  blockstore::BlockId key2 = blockstore::BlockId::FromString("1491BB4932A389EE14BC7090AC772972");
  baseBlockStore->store(key2, baseBlockStore->load(blockId).value());
  EXPECT_NE(boost::none, blockStore->load(key2));
  EXPECT_FALSE(onIntegrityViolation.wasCalled());
}

// TODO Test more integrity cases:
//   - RollbackPrevention_DoesntAllowReintroducingDeletedBlocks with different client id (i.e. trying to re-introduce the newest block of a different client)
//   - RollbackPrevention_AllowsReintroducingDeletedBlocksWithNewVersionNumber with different client id
//   - Think about more...
// TODO Test that disabling integrity checks allows all these cases

TEST_F(IntegrityBlockStoreTest_Default, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(0));
}

TEST_F(IntegrityBlockStoreTest_Default, PhysicalBlockSize_zerovirtual) {
  auto blockId = CreateBlockReturnKey(Data(0));
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}

TEST_F(IntegrityBlockStoreTest_Default, PhysicalBlockSize_negativeboundaries) {
  // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
  // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
  auto physicalSizeForVirtualSizeZero = baseBlockStore->load(CreateBlockReturnKey(Data(0))).value().size();
  if (physicalSizeForVirtualSizeZero > 0) {
    EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
  }
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
  EXPECT_EQ(1u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(IntegrityBlockStoreTest_Default, PhysicalBlockSize_positive) {
  auto blockId = CreateBlockReturnKey(Data(10*1024));
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(10*1024u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}
