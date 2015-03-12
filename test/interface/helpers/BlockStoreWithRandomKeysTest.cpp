#include "../../../interface/helpers/BlockStoreWithRandomKeys.h"
#include "google/gtest/gtest.h"
#include "google/gmock/gmock.h"


using ::testing::Test;
using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;

using std::string;
using std::unique_ptr;
using std::make_unique;

using namespace blockstore;

class BlockStoreWithRandomKeysMock: public BlockStoreWithRandomKeys {
public:
  unique_ptr<Block> create(const Key &key, size_t size) {
    return unique_ptr<Block>(do_create(key, size));
  }
  MOCK_METHOD2(do_create, Block*(const Key &, size_t));
  unique_ptr<Block> load(const Key &key) {
    return unique_ptr<Block>(do_load(key));
  }
  MOCK_METHOD1(do_load, Block*(const Key &));
  void remove(unique_ptr<Block> block) {}
  MOCK_CONST_METHOD0(numBlocks, uint64_t());
};

class BlockMock: public Block {
public:
  BlockMock(): Block(Key::CreateRandomKey()) {}
  MOCK_CONST_METHOD0(data, const void*());
  MOCK_METHOD3(write, void(const void*, uint64_t, uint64_t));
  MOCK_METHOD0(flush, void());
  MOCK_CONST_METHOD0(size, size_t());
  MOCK_CONST_METHOD0(key, const Key&());
};

class BlockStoreWithRandomKeysTest: public Test {
public:
  BlockStoreWithRandomKeysMock blockStoreMock;
  BlockStore &blockStore = blockStoreMock;
  const blockstore::Key key = Key::FromString("1491BB4932A389EE14BC7090AC772972");
};

TEST_F(BlockStoreWithRandomKeysTest, SizeIsPassedThrough0) {
  EXPECT_CALL(blockStoreMock, do_create(_, 0)).WillOnce(Return(new BlockMock));
  blockStore.create(0);
}

TEST_F(BlockStoreWithRandomKeysTest, SizeIsPassedThrough1) {
  EXPECT_CALL(blockStoreMock, do_create(_, 1)).WillOnce(Return(new BlockMock));
  blockStore.create(1);
}

TEST_F(BlockStoreWithRandomKeysTest, SizeIsPassedThrough1024) {
  EXPECT_CALL(blockStoreMock, do_create(_, 1024)).WillOnce(Return(new BlockMock));
  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, KeyHasCorrectSize) {
  EXPECT_CALL(blockStoreMock, do_create(_, _)).WillOnce(Invoke([](const Key &key, size_t) {
    EXPECT_EQ(Key::KEYLENGTH_STRING, key.ToString().size());
    return new BlockMock;
  }));

  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, TwoBlocksGetDifferentKeys) {
  Key first_key = key;
  EXPECT_CALL(blockStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const Key &key, size_t) {
        first_key = key;
        return new BlockMock;
      }))
      .WillOnce(Invoke([&first_key](const Key &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlockMock;
      }));

  blockStore.create(1024);
  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExists) {
  Key first_key = key;
  EXPECT_CALL(blockStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const Key &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const Key &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlockMock;
      }));

  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExistsTwoTimes) {
  Key first_key = key;
  EXPECT_CALL(blockStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const Key &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const Key &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const Key &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlockMock;
      }));

  blockStore.create(1024);
}
