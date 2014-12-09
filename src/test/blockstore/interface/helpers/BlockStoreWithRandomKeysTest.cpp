#include <blockstore/interface/helpers/BlockStoreWithRandomKeys.h>
#include <gtest/gtest.h>
#include <gmock/gmock.h>


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
};

class BlockMock: public Block {
public:
  MOCK_METHOD0(data, void*());
  MOCK_CONST_METHOD0(data, const void*());
  MOCK_METHOD0(flush, void());
  MOCK_CONST_METHOD0(size, size_t());
};

class BlockStoreWithRandomKeysTest: public Test {
public:
  BlockStoreWithRandomKeysMock blockStoreMock;
  BlockStore &blockStore = blockStoreMock;
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
    EXPECT_EQ(Key::KEYLENGTH_STRING, key.AsString().size());
    return new BlockMock;
  }));

  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, TwoBlocksGetDifferentKeys) {
  Key first_key = Key::CreateDummyKey();
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
  Key first_key = Key::CreateDummyKey();
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
  Key first_key = Key::CreateDummyKey();
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
