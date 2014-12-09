#include <blockstore/interface/helpers/BlockStoreWithRandomKeys.h>
#include <blockstore/utils/RandomKeyGenerator.h>
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
  unique_ptr<BlockWithKey> create(const std::string &key, size_t size) {
    return unique_ptr<BlockWithKey>(do_create(key, size));
  }
  MOCK_METHOD2(do_create, BlockWithKey*(const std::string &, size_t));
  unique_ptr<Block> load(const string &key) {
    return unique_ptr<Block>(do_load(key));
  }
  MOCK_METHOD1(do_load, Block*(const string &));
  MOCK_METHOD1(exists, bool(const string &));
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
  EXPECT_CALL(blockStoreMock, do_create(_, 0)).WillOnce(Return(new BlockWithKey("", make_unique<BlockMock>())));
  blockStore.create(0);
}

TEST_F(BlockStoreWithRandomKeysTest, SizeIsPassedThrough1) {
  EXPECT_CALL(blockStoreMock, do_create(_, 1)).WillOnce(Return(new BlockWithKey("", make_unique<BlockMock>())));
  blockStore.create(1);
}

TEST_F(BlockStoreWithRandomKeysTest, SizeIsPassedThrough1024) {
  EXPECT_CALL(blockStoreMock, do_create(_, 1024)).WillOnce(Return(new BlockWithKey("", make_unique<BlockMock>())));
  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, KeyHasCorrectSize) {
  EXPECT_CALL(blockStoreMock, do_create(_, _)).WillOnce(Invoke([](const string &key, size_t) {
    EXPECT_EQ(RandomKeyGenerator::KEYLENGTH, key.size());
    return new BlockWithKey("", make_unique<BlockMock>());
  }));

  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, TwoBlocksGetDifferentKeys) {
  string first_key;
  EXPECT_CALL(blockStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return new BlockWithKey("", make_unique<BlockMock>());
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlockWithKey("", make_unique<BlockMock>());
      }));

  blockStore.create(1024);
  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExists) {
  string first_key;
  EXPECT_CALL(blockStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlockWithKey("", make_unique<BlockMock>());
      }));

  blockStore.create(1024);
}

TEST_F(BlockStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExistsTwoTimes) {
  string first_key;
  EXPECT_CALL(blockStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlockWithKey("", make_unique<BlockMock>());
      }));

  blockStore.create(1024);
}
