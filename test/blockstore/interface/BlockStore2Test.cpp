#include "blockstore/interface/BlockStore2.h"
#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/data/DataFixture.h>

using ::testing::Test;
using ::testing::Return;
using ::testing::Invoke;
using ::testing::Eq;
using ::testing::ByRef;

using std::string;
using cpputils::Data;
using cpputils::DataFixture;
using boost::optional;

namespace boost {
    inline void PrintTo(const optional<cpputils::Data> &, ::std::ostream *os) {
        *os << "optional<Data>";
    }
}

using namespace blockstore;

class BlockStore2Mock: public BlockStore2 {
public:
    MOCK_METHOD(BlockId, createBlockId, (), (const, override));
    MOCK_METHOD(bool, tryCreate, (const BlockId &blockId, const cpputils::Data &data), (override));
    MOCK_METHOD(void, store, (const BlockId &, const Data &data), (override));
    MOCK_METHOD(optional<Data>, load, (const BlockId &), (const, override));
    MOCK_METHOD(bool, remove, (const BlockId &), (override));
    MOCK_METHOD(uint64_t, numBlocks, (), (const, override));
    MOCK_METHOD(uint64_t, estimateNumFreeBytes, (), (const, override));
    MOCK_METHOD(uint64_t, blockSizeFromPhysicalBlockSize, (uint64_t), (const, override));
    MOCK_METHOD(void, forEachBlock, (std::function<void (const blockstore::BlockId &)>), (const, override));
};

class BlockStore2Test: public Test {
public:
    BlockStore2Test() :blockStoreMock(), blockStore(blockStoreMock),
                      blockId1(BlockId::FromString("1491BB4932A389EE14BC7090AC772972")),
                      blockId2(BlockId::FromString("AC772971491BB4932A389EE14BC7090A")),
                      blockId3(BlockId::FromString("1BB4932A38AC77C7090A2971499EE14B")) {}

    BlockStore2Mock blockStoreMock;
    BlockStore2 &blockStore;
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

TEST_F(BlockStore2Test, DataIsPassedThrough0) {
    Data data = createDataWithSize(0);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data)))).WillOnce(Return(true));
    EXPECT_EQ(blockId1, blockStore.create(data));
}

TEST_F(BlockStore2Test, DataIsPassedThrough1) {
    Data data = createDataWithSize(1);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data)))).WillOnce(Return(true));
    EXPECT_EQ(blockId1, blockStore.create(data));
}

TEST_F(BlockStore2Test, DataIsPassedThrough1024) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data)))).WillOnce(Return(true));
    EXPECT_EQ(blockId1, blockStore.create(data));
}

TEST_F(BlockStore2Test, BlockIdIsCorrect) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, tryCreate(blockId1, testing::_)).WillOnce(Return(true));
    EXPECT_EQ(blockId1, blockStore.create(data));
}

TEST_F(BlockStore2Test, TwoBlocksGetDifferentIds) {
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, testing::_))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId1, blockId);
                return true;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId2, blockId);
                return true;
            }));

    Data data = createDataWithSize(1024);
    EXPECT_EQ(blockId1, blockStore.create(data));
    EXPECT_EQ(blockId2, blockStore.create(data));
}

TEST_F(BlockStore2Test, WillTryADifferentIdIfKeyAlreadyExists) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data))))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId1, blockId);
                return false;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId2, blockId);
                return true;
            }));

    EXPECT_EQ(blockId2, blockStore.create(data));
}

TEST_F(BlockStore2Test, WillTryADifferentIdIfIdAlreadyExistsTwoTimes) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2))
            .WillOnce(Return(blockId3));
    EXPECT_CALL(blockStoreMock, tryCreate(testing::_, Eq(ByRef(data))))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId1, blockId);
                return false;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId2, blockId);
                return false;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId3, blockId);
                return true;
            }));

    EXPECT_EQ(blockId3, blockStore.create(data));
}
