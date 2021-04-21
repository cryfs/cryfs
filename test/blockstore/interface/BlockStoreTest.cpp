#include "blockstore/interface/BlockStore.h"
#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/data/DataFixture.h>
#include <cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h>

using ::testing::Test;
using ::testing::Return;
using ::testing::Invoke;
using ::testing::Eq;
using ::testing::ByRef;
using ::testing::Action;

using std::string;
using cpputils::Data;
using cpputils::DataFixture;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;

using namespace blockstore;

class BlockStoreMock: public BlockStore {
public:
    MOCK_METHOD(BlockId, createBlockId, (), (override));
    MOCK_METHOD(optional<unique_ref<Block>>, tryCreate, (const BlockId &, Data data), (override));
    MOCK_METHOD(unique_ref<Block>, overwrite, (const BlockId &, Data data), (override));
    MOCK_METHOD(optional<unique_ref<Block>>, load, (const BlockId &), (override));
    MOCK_METHOD(void, remove, (unique_ref<Block>), (override));
    MOCK_METHOD(void, remove, (const BlockId &), (override));
    MOCK_METHOD(uint64_t, numBlocks, (), (const, override));
    MOCK_METHOD(uint64_t, estimateNumFreeBytes, (), (const, override));
    MOCK_METHOD(uint64_t, blockSizeFromPhysicalBlockSize, (uint64_t), (const, override));
    MOCK_METHOD(void, forEachBlock, (std::function<void (const blockstore::BlockId &)>), (const, override));
};

class BlockMock: public Block {
public:
    BlockMock(): Block(BlockId::Random()) {}
    MOCK_METHOD(const void*, data, (), (const, override));
    MOCK_METHOD(void, write, (const void*, uint64_t, uint64_t), (override));
    MOCK_METHOD(void, flush, (), (override));
    MOCK_METHOD(size_t, size, (), (const, override));
    MOCK_METHOD(void, resize, (size_t), (override));
};

class BlockStoreTest: public Test {
public:
    BlockStoreTest() :blockStoreMock(), blockStore(blockStoreMock),
                      blockId1(BlockId::FromString("1491BB4932A389EE14BC7090AC772972")),
                      blockId2(BlockId::FromString("AC772971491BB4932A389EE14BC7090A")),
                      blockId3(BlockId::FromString("1BB4932A38AC77C7090A2971499EE14B")) {}

    BlockStoreMock blockStoreMock;
    BlockStore &blockStore;
    const BlockId blockId1;
    const BlockId blockId2;
    const BlockId blockId3;

    Data createDataWithSize(size_t size) {
        Data fixture(DataFixture::generate(size));
        Data data(size);
        std::memcpy(data.data(), fixture.data(), size);
        return data;
    }
};

const Action<optional<unique_ref<Block>>(const BlockId &, cpputils::Data)> ReturnNewBlockMock = Invoke(
    [] (const BlockId&, cpputils::Data) {
        return optional<unique_ref<Block>>(unique_ref<Block>(make_unique_ref<BlockMock>()));
    });

TEST_F(BlockStoreTest, DataIsPassedThrough0) {
    Data data = createDataWithSize(0);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data)))).WillOnce(ReturnNewBlockMock);
    blockStore.create(data);
}

TEST_F(BlockStoreTest, DataIsPassedThrough1) {
    Data data = createDataWithSize(1);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data)))).WillOnce(ReturnNewBlockMock);
    blockStore.create(data);
}

TEST_F(BlockStoreTest, DataIsPassedThrough1024) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data)))).WillOnce(ReturnNewBlockMock);
    blockStore.create(data);
}

TEST_F(BlockStoreTest, BlockIdIsCorrect) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(blockId1, testing::_)).WillOnce(ReturnNewBlockMock);
    blockStore.create(data);
}

TEST_F(BlockStoreTest, TwoBlocksGetDifferentIds) {
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, testing::_))
            .WillOnce(Invoke([this](const BlockId &blockId, Data) {
                EXPECT_EQ(blockId1, blockId);
                return optional<unique_ref<Block>>(unique_ref<Block>(make_unique_ref<BlockMock>()));
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, Data) {
                EXPECT_EQ(blockId2, blockId);
                return optional<unique_ref<Block>>(unique_ref<Block>(make_unique_ref<BlockMock>()));
            }));

    Data data = createDataWithSize(1024);
    blockStore.create(data);
    blockStore.create(data);
}

TEST_F(BlockStoreTest, WillTryADifferentIdIfKeyAlreadyExists) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data))))
            .WillOnce(Invoke([this](const BlockId &blockId, Data ) {
                EXPECT_EQ(blockId1, blockId);
                return boost::none;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, Data ) {
                EXPECT_EQ(blockId2, blockId);
                return optional<unique_ref<Block>>(unique_ref<Block>(make_unique_ref<BlockMock>()));
            }));

    blockStore.create(data);
}

TEST_F(BlockStoreTest, WillTryADifferentIdIfIdAlreadyExistsTwoTimes) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2))
            .WillOnce(Return(blockId3));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data))))
            .WillOnce(Invoke([this](const BlockId &blockId, Data) {
                EXPECT_EQ(blockId1, blockId);
                return boost::none;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, Data) {
                EXPECT_EQ(blockId2, blockId);
                return boost::none;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, Data) {
                EXPECT_EQ(blockId3, blockId);
                return optional<unique_ref<Block>>(unique_ref<Block>(make_unique_ref<BlockMock>()));
            }));

    blockStore.create(data);
}
