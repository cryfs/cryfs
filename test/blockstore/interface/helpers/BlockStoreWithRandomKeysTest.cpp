#include "blockstore/interface/helpers/BlockStoreWithRandomKeys.h"
#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/data/DataFixture.h>

using ::testing::Test;
using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;
using ::testing::Eq;
using ::testing::ByRef;

using std::string;
using cpputils::Data;
using cpputils::DataFixture;
using cpputils::unique_ref;
using boost::optional;

using namespace blockstore;

class BlockStoreWithRandomKeysMock: public BlockStoreWithRandomKeys {
public:
  optional<unique_ref<Block>> tryCreate(const Key &key, Data data) {
    return cpputils::nullcheck(std::unique_ptr<Block>(do_create(key, data)));
  }
  MOCK_METHOD2(do_create, Block*(const Key &, const Data &data));
  optional<unique_ref<Block>> load(const Key &key) {
    return cpputils::nullcheck(std::unique_ptr<Block>(do_load(key)));
  }
  MOCK_METHOD1(do_load, Block*(const Key &));
  void remove(unique_ref<Block> block) {UNUSED(block);}
  MOCK_CONST_METHOD0(numBlocks, uint64_t());
  MOCK_CONST_METHOD0(estimateNumFreeBytes, uint64_t());
  MOCK_CONST_METHOD1(blockSizeFromPhysicalBlockSize, uint64_t(uint64_t));
};

class BlockMock: public Block {
public:
  BlockMock(): Block(cpputils::Random::PseudoRandom().getFixedSize<Key::BINARY_LENGTH>()) {}
  MOCK_CONST_METHOD0(data, const void*());
  MOCK_METHOD3(write, void(const void*, uint64_t, uint64_t));
  MOCK_METHOD0(flush, void());
  MOCK_CONST_METHOD0(size, size_t());
  MOCK_METHOD1(resize, void(size_t));
  MOCK_CONST_METHOD0(key, const Key&());
};

class BlockStoreWithRandomKeysTest: public Test {
public:
  BlockStoreWithRandomKeysTest() :blockStoreMock(), blockStore(blockStoreMock),
                                  key(Key::FromString("1491BB4932A389EE14BC7090AC772972")) {}

  BlockStoreWithRandomKeysMock blockStoreMock;
  BlockStore &blockStore;
  const blockstore::Key key;

  Data createDataWithSize(size_t size) {
	Data fixture(DataFixture::generate(size));
	Data data(size);
	std::memcpy(data.data(), fixture.data(), size);
	return data;
  }
};

TEST_F(BlockStoreWithRandomKeysTest, DataIsPassedThrough0) {
  Data data = createDataWithSize(0);
  EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data)))).WillOnce(Return(new BlockMock));
  blockStore.create(data);
}

TEST_F(BlockStoreWithRandomKeysTest, DataIsPassedThrough1) {
  Data data = createDataWithSize(1);
  EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data)))).WillOnce(Return(new BlockMock));
  blockStore.create(data);
}

TEST_F(BlockStoreWithRandomKeysTest, DataIsPassedThrough1024) {
  Data data = createDataWithSize(1024);
  EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data)))).WillOnce(Return(new BlockMock));
  blockStore.create(data);
}

TEST_F(BlockStoreWithRandomKeysTest, KeyHasCorrectSize) {
  EXPECT_CALL(blockStoreMock, do_create(_, _)).WillOnce(Invoke([](const Key &key, const Data &) {
    EXPECT_EQ(Key::STRING_LENGTH, key.ToString().size());
    return new BlockMock;
  }));

  blockStore.create(createDataWithSize(1024));
}

TEST_F(BlockStoreWithRandomKeysTest, TwoBlocksGetDifferentKeys) {
  Key first_key = key;
  EXPECT_CALL(blockStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const Key &key, const Data &) {
        first_key = key;
        return new BlockMock;
      }))
      .WillOnce(Invoke([&first_key](const Key &key, const Data &) {
        EXPECT_NE(first_key, key);
        return new BlockMock;
      }));

  Data data = createDataWithSize(1024);
  blockStore.create(data);
  blockStore.create(data);
}

TEST_F(BlockStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExists) {
  Key first_key = key;
  Data data = createDataWithSize(1024);
  EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data))))
      .WillOnce(Invoke([&first_key](const Key &key, const Data &) {
        first_key = key;
        return nullptr;
      }))
	  //TODO Check that this test case fails when the second do_create call gets different data
      .WillOnce(Invoke([&first_key](const Key &key, const Data &) {
        EXPECT_NE(first_key, key);
        return new BlockMock;
      }));

  blockStore.create(data);
}

TEST_F(BlockStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExistsTwoTimes) {
  Key first_key = key;
  Data data = createDataWithSize(1024);
  EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data))))
      .WillOnce(Invoke([&first_key](const Key &key, const Data &) {
        first_key = key;
        return nullptr;
      }))
	  //TODO Check that this test case fails when the second/third do_create calls get different data
      .WillOnce(Invoke([&first_key](const Key &key, const Data &) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const Key &key, const Data &) {
        EXPECT_NE(first_key, key);
        return new BlockMock;
      }));

  blockStore.create(data);
}
