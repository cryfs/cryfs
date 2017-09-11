#include "cpp-utils/crypto/cryptopp_byte.h"
#include <gtest/gtest.h>
#include "../../../cpp-utils/crypto/symmetric/testutils/FakeAuthenticatedCipher.h"
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

  blockstore::Key CreateBlockDirectlyWithFixtureAndReturnKey() {
    return CreateBlockReturnKey(data);
  }

  blockstore::Key CreateBlockReturnKey(const Data &initData) {
    return blockStore->create(initData.copy());
  }

  blockstore::Key CreateBlockWriteFixtureToItAndReturnKey() {
    auto key = blockStore->create(Data(data.size()));
    blockStore->store(key, data);
    return key;
  }

  void ModifyBaseBlock(const blockstore::Key &key) {
    auto block = baseBlockStore->load(key).value();
    byte* middle_byte = ((CryptoPP::byte*)block.data()) + 10;
    *middle_byte = *middle_byte + 1;
    baseBlockStore->store(key, block);
  }

  blockstore::Key CopyBaseBlock(const blockstore::Key &key) {
    auto source = baseBlockStore->load(key).value();
    return baseBlockStore->create(std::move(source));
  }

private:
  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStoreTest);
};

TEST_F(EncryptedBlockStoreTest, LoadingWithSameKeyWorks_WriteOnCreate) {
  auto key = CreateBlockDirectlyWithFixtureAndReturnKey();
  auto loaded = blockStore->load(key);
  EXPECT_NE(boost::none, loaded);
  EXPECT_EQ(data.size(), loaded->size());
  EXPECT_EQ(0, std::memcmp(data.data(), loaded->data(), data.size()));
}

TEST_F(EncryptedBlockStoreTest, LoadingWithSameKeyWorks_WriteSeparately) {
  auto key = CreateBlockWriteFixtureToItAndReturnKey();
  auto loaded = blockStore->load(key);
  EXPECT_NE(boost::none, loaded);
  EXPECT_EQ(data.size(), loaded->size());
  EXPECT_EQ(0, std::memcmp(data.data(), loaded->data(), data.size()));
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentKeyDoesntWork_WriteOnCreate) {
  auto key = CreateBlockDirectlyWithFixtureAndReturnKey();
  blockStore->__setKey(FakeAuthenticatedCipher::Key2());
  auto loaded = blockStore->load(key);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentKeyDoesntWork_WriteSeparately) {
  auto key = CreateBlockWriteFixtureToItAndReturnKey();
  blockStore->__setKey(FakeAuthenticatedCipher::Key2());
  auto loaded = blockStore->load(key);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingModifiedBlockFails_WriteOnCreate) {
  auto key = CreateBlockDirectlyWithFixtureAndReturnKey();
  ModifyBaseBlock(key);
  auto loaded = blockStore->load(key);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingModifiedBlockFails_WriteSeparately) {
  auto key = CreateBlockWriteFixtureToItAndReturnKey();
  ModifyBaseBlock(key);
  auto loaded = blockStore->load(key);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentBlockIdFails_WriteOnCreate) {
  auto key = CreateBlockDirectlyWithFixtureAndReturnKey();
  auto key2 = CopyBaseBlock(key);
  auto loaded = blockStore->load(key2);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentBlockIdFails_WriteSeparately) {
  auto key = CreateBlockWriteFixtureToItAndReturnKey();
  auto key2 = CopyBaseBlock(key);
  auto loaded = blockStore->load(key2);
  EXPECT_EQ(boost::none, loaded);
}

TEST_F(EncryptedBlockStoreTest, PhysicalBlockSize_zerophysical) {
  EXPECT_EQ(0u, blockStore->blockSizeFromPhysicalBlockSize(0));
}

TEST_F(EncryptedBlockStoreTest, PhysicalBlockSize_zerovirtual) {
  auto key = CreateBlockReturnKey(Data(0));
  auto base = baseBlockStore->load(key).value();
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
  auto key = CreateBlockReturnKey(Data(10*1024));
  auto base = baseBlockStore->load(key).value();
  EXPECT_EQ(10*1024u, blockStore->blockSizeFromPhysicalBlockSize(base.size()));
}
