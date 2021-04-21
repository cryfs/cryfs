#include <gtest/gtest.h>
#include "cpp-utils/crypto/symmetric/testutils/FakeAuthenticatedCipher.h"
#include "blockstore/implementations/encrypted/EncryptedBlockStore2.h"
#include "blockstore/implementations/inmemory/InMemoryBlockStore2.h"
#include "blockstore/utils/BlockStoreUtils.h"
#include "../../testutils/gtest_printers.h"
#include <cpp-utils/data/DataFixture.h>

using ::testing::Test;

using cpputils::DataFixture;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::FakeAuthenticatedCipher;

using blockstore::inmemory::InMemoryBlockStore2;

using namespace blockstore::encrypted;

class EncryptedBlockStoreTest: public Test {
public:
  static constexpr unsigned int BLOCKSIZE = 1024;
  EncryptedBlockStoreTest():
    baseBlockStore(new InMemoryBlockStore2),
    blockStore(make_unique_ref<EncryptedBlockStore2<FakeAuthenticatedCipher>>(std::move(cpputils::nullcheck(std::unique_ptr<InMemoryBlockStore2>(baseBlockStore)).value()), FakeAuthenticatedCipher::Key1())),
    data(DataFixture::generate(BLOCKSIZE)) {
  }
  InMemoryBlockStore2 *baseBlockStore;
  unique_ref<EncryptedBlockStore2<FakeAuthenticatedCipher>> blockStore;
  Data data;

  blockstore::BlockId CreateBlockDirectlyWithFixtureAndReturnKey() {
    return CreateBlockReturnKey(data);
  }

  blockstore::BlockId CreateBlockReturnKey(const Data &initData) {
    return blockStore->create(initData.copy());
  }

  blockstore::BlockId CreateBlockWriteFixtureToItAndReturnKey() {
    auto blockId = blockStore->create(Data(data.size()));
    blockStore->store(blockId, data);
    return blockId;
  }

  void ModifyBaseBlock(const blockstore::BlockId &blockId) {
    auto block = baseBlockStore->load(blockId).value();
    CryptoPP::byte* middle_byte = static_cast<CryptoPP::byte*>(block.data()) + 10;
    *middle_byte = *middle_byte + 1;
    baseBlockStore->store(blockId, block);
  }

  blockstore::BlockId CopyBaseBlock(const blockstore::BlockId &blockId) {
    auto source = baseBlockStore->load(blockId).value();
    return baseBlockStore->create(source);
  }

private:
  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStoreTest);
};

TEST_F(EncryptedBlockStoreTest, LoadingWithSameKeyWorks_WriteOnCreate) {
  auto blockId = CreateBlockDirectlyWithFixtureAndReturnKey();
  auto loaded = blockStore->load(blockId);
  EXPECT_NE(boost::none, loaded);
  EXPECT_EQ(data.size(), loaded->size());
  EXPECT_EQ(0, std::memcmp(data.data(), loaded->data(), data.size()));
}

TEST_F(EncryptedBlockStoreTest, LoadingWithSameKeyWorks_WriteSeparately) {
  auto blockId = CreateBlockWriteFixtureToItAndReturnKey();
  auto loaded = blockStore->load(blockId);
  EXPECT_NE(boost::none, loaded);
  EXPECT_EQ(data.size(), loaded->size());
  EXPECT_EQ(0, std::memcmp(data.data(), loaded->data(), data.size()));
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentKeyDoesntWork_WriteOnCreate) {
  auto blockId = CreateBlockDirectlyWithFixtureAndReturnKey();
  blockStore->_setKey(FakeAuthenticatedCipher::Key2());
  auto loaded = blockStore->load(blockId);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentKeyDoesntWork_WriteSeparately) {
  auto blockId = CreateBlockWriteFixtureToItAndReturnKey();
  blockStore->_setKey(FakeAuthenticatedCipher::Key2());
  auto loaded = blockStore->load(blockId);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingModifiedBlockFails_WriteOnCreate) {
  auto blockId = CreateBlockDirectlyWithFixtureAndReturnKey();
  ModifyBaseBlock(blockId);
  auto loaded = blockStore->load(blockId);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingModifiedBlockFails_WriteSeparately) {
  auto blockId = CreateBlockWriteFixtureToItAndReturnKey();
  ModifyBaseBlock(blockId);
  auto loaded = blockStore->load(blockId);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(0));
}

TEST_F(EncryptedBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto blockId = CreateBlockReturnKey(Data(0));
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}

TEST_F(EncryptedBlockStoreTest, PhysicalBlockSize_negativeboundaries) {
  // This tests that a potential if/else in blockSizeFromPhysicalBlockSize that catches negative values has the
  // correct boundary set. We test the highest value that is negative and the smallest value that is positive.
  auto physicalSizeForVirtualSizeZero = baseBlockStore->load(CreateBlockReturnKey(Data(0))).value().size();
  if (physicalSizeForVirtualSizeZero > 0) {
    EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero - 1));
  }
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero));
  EXPECT_EQ(1u, blockStore->blockSizeFromPhysicalBlockSize(physicalSizeForVirtualSizeZero + 1));
}

TEST_F(EncryptedBlockStoreTest, PhysicalBlockSize_positive) {
  auto blockId = CreateBlockReturnKey(Data(10*1024));
  auto base = baseBlockStore->load(blockId).value();
  EXPECT_EQ(10*1024u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}
