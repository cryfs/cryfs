#include "blockstore/interface/BlockStore.h"
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

class BlockStoreMock: public BlockStore {
public:
    MOCK_METHOD0(createBlockId, BlockId());
    optional<unique_ref<Block>> tryCreate(const BlockId &blockId, Data data) {
        return cpputils::nullcheck(std::unique_ptr<Block>(do_create(blockId, data)));
    }
    MOCK_METHOD2(do_create, Block*(const BlockId &, const Data &data));
    unique_ref<Block> overwrite(const BlockId &blockId, Data data) {
        return cpputils::nullcheck(std::unique_ptr<Block>(do_overwrite(blockId, data))).value();
    }
    MOCK_METHOD2(do_overwrite, Block*(const BlockId &, const Data &data));
    optional<unique_ref<Block>> load(const BlockId &blockId) {
        return cpputils::nullcheck(std::unique_ptr<Block>(do_load(blockId)));
    }
    MOCK_METHOD1(do_load, Block*(const BlockId &));
    void remove(unique_ref<Block> block) {UNUSED(block);}
    MOCK_METHOD1(remove, void(const BlockId &));
    MOCK_CONST_METHOD0(numBlocks, uint64_t());
    MOCK_CONST_METHOD0(estimateNumFreeBytes, uint64_t());
    MOCK_CONST_METHOD1(blockSizeFromPhysicalBlockSize, uint64_t(uint64_t));
    MOCK_CONST_METHOD1(forEachBlock, void(std::function<void (const blockstore::BlockId &)>));
};

class BlockMock: public Block {
public:
    BlockMock(): Block(BlockId::Random()) {}
    MOCK_CONST_METHOD0(data, const void*());
    MOCK_METHOD3(write, void(const void*, uint64_t, uint64_t));
    MOCK_METHOD0(flush, void());
    MOCK_CONST_METHOD0(size, size_t());
    MOCK_METHOD1(resize, void(size_t));
    MOCK_CONST_METHOD0(blockId, const BlockId&());
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

TEST_F(BlockStoreTest, DataIsPassedThrough0) {
    Data data = createDataWithSize(0);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data)))).WillOnce(Return(new BlockMock));
    blockStore.create(data);
}

TEST_F(BlockStoreTest, DataIsPassedThrough1) {
    Data data = createDataWithSize(1);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data)))).WillOnce(Return(new BlockMock));
    blockStore.create(data);
}

TEST_F(BlockStoreTest, DataIsPassedThrough1024) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data)))).WillOnce(Return(new BlockMock));
    blockStore.create(data);
}

TEST_F(BlockStoreTest, BlockIdIsCorrect) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId()).WillOnce(Return(blockId1));
    EXPECT_CALL(blockStoreMock, do_create(blockId1, _)).WillOnce(Return(new BlockMock));
    blockStore.create(data);
}

TEST_F(BlockStoreTest, TwoBlocksGetDifferentIds) {
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2));
    EXPECT_CALL(blockStoreMock, do_create(_, _))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId1, blockId);
                return new BlockMock;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId2, blockId);
                return new BlockMock;
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
    EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data))))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId1, blockId);
                return nullptr;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId2, blockId);
                return new BlockMock;
            }));

    blockStore.create(data);
}

TEST_F(BlockStoreTest, WillTryADifferentIdIfIdAlreadyExistsTwoTimes) {
    Data data = createDataWithSize(1024);
    EXPECT_CALL(blockStoreMock, createBlockId())
            .WillOnce(Return(blockId1))
            .WillOnce(Return(blockId2))
            .WillOnce(Return(blockId3));
    EXPECT_CALL(blockStoreMock, do_create(_, Eq(ByRef(data))))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId1, blockId);
                return nullptr;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId2, blockId);
                return nullptr;
            }))
            .WillOnce(Invoke([this](const BlockId &blockId, const Data &) {
                EXPECT_EQ(blockId3, blockId);
                return new BlockMock;
            }));

    blockStore.create(data);
}
