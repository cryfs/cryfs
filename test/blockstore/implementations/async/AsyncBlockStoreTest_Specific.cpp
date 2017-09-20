#include <gmock/gmock.h>
#include <blockstore/implementations/async/AsyncBlockStore2.h>
#include <cpp-utils/lock/ConditionBarrier.h>
#include <boost/fiber/fiber.hpp>
#include <cpp-utils/data/DataFixture.h>

using namespace blockstore;
using blockstore::async::AsyncBlockStore2;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using cpputils::DataFixture;
using boost::optional;
using boost::none;
using testing::Test;
using testing::InvokeWithoutArgs;
using testing::_;
using cpputils::ConditionBarrier;

class BlockStore2Mock: public BlockStore2 {
public:
    MOCK_CONST_METHOD0(createBlockId, BlockId());
    MOCK_METHOD2(tryCreate, bool(const BlockId &blockId, const cpputils::Data &data));
    MOCK_METHOD2(store, void(const BlockId &, const Data &data));
    MOCK_CONST_METHOD1(load, optional<Data>(const BlockId &));
    MOCK_METHOD1(remove, bool(const BlockId &));
    MOCK_CONST_METHOD0(numBlocks, uint64_t());
    MOCK_CONST_METHOD0(estimateNumFreeBytes, uint64_t());
    MOCK_CONST_METHOD1(blockSizeFromPhysicalBlockSize, uint64_t(uint64_t));
    MOCK_CONST_METHOD1(forEachBlock, void(std::function<void (const blockstore::BlockId &)>));
};

class AsyncBlockStore2Test: public Test {
public:
    static constexpr size_t NUM_THREADS = 1;

    AsyncBlockStore2Test() : baseStoreMock_(make_unique_ref<BlockStore2Mock>()),
                             baseStoreMock(baseStoreMock_.get()),
                             blockStore(std::move(baseStoreMock_), NUM_THREADS),
                             blockId1(BlockId::FromString("1491BB4932A389EE14BC7090AC772972")),
                             blockId2(BlockId::FromString("AC772971491BB4932A389EE14BC7090A")),
                             blockId3(BlockId::FromString("1BB4932A38AC77C7090A2971499EE14B")) {}

    unique_ref<BlockStore2Mock> baseStoreMock_;
    BlockStore2Mock *baseStoreMock;
    AsyncBlockStore2 blockStore;
    const BlockId blockId1;
    const BlockId blockId2;
    const BlockId blockId3;
};

// This test is testing that blockStore.load (and other functions) doesn't block the whole thread,
// but allows other fibers to run on the same thread.
class AsyncBlockStore2Test_DoesntBlock : public AsyncBlockStore2Test {
public:
    // The barrier has to use std::mutex because they must block the whole thread.
    // If we allow other fibers to run on the base store thread, this test doesn't make sense anymore.
    ConditionBarrier<std::mutex, std::condition_variable> barrier;

    void createFiberReleasingBarrier() {
        boost::fibers::fiber([this] () {
            // release barrier (this is proof that this fiber got execution time)
            barrier.release();
        }).detach();
    }

    decltype(auto) BlockFullThreadUntilBarrierReleased() {
        return InvokeWithoutArgs([this] () {
            // block full base store thread until fiber ran
            this->barrier.wait();
        });
    }

    template<class Result>
    decltype(auto) BlockFullThreadUntilBarrierReleased(Result result) {
        return InvokeWithoutArgs([this, result = std::move(result)] () {
            // block full base store thread until fiber ran
            this->barrier.wait();
            return std::move(result);
        });
    }
};

TEST_F(AsyncBlockStore2Test_DoesntBlock, whenCallingTryCreate_thenDoesntBlock) {
    EXPECT_CALL(*baseStoreMock, tryCreate(_, _)).WillOnce(BlockFullThreadUntilBarrierReleased(true));

    createFiberReleasingBarrier();
    blockStore.tryCreate(blockId1, Data(0));
    // If this doesn't deadlock, this means that the blockStore call allowed the fiber releasing the barrier to run.
}

TEST_F(AsyncBlockStore2Test_DoesntBlock, whenCallingStore_thenDoesntBlock) {
    EXPECT_CALL(*baseStoreMock, store(_, _)).WillOnce(BlockFullThreadUntilBarrierReleased());

    createFiberReleasingBarrier();
    blockStore.store(blockId1, Data(0));
    // If this doesn't deadlock, this means that the blockStore call allowed the fiber releasing the barrier to run.
}

TEST_F(AsyncBlockStore2Test_DoesntBlock, whenCallingLoad_thenDoesntBlock) {
    EXPECT_CALL(*baseStoreMock, load(_)).WillOnce(BlockFullThreadUntilBarrierReleased(boost::none));

    createFiberReleasingBarrier();
    blockStore.load(blockId1);
    // If this doesn't deadlock, this means that the blockStore call allowed the fiber releasing the barrier to run.
}

TEST_F(AsyncBlockStore2Test_DoesntBlock, whenCallingRemove_thenDoesntBlock) {
    EXPECT_CALL(*baseStoreMock, remove(_)).WillOnce(BlockFullThreadUntilBarrierReleased(true));

    createFiberReleasingBarrier();
    blockStore.remove(blockId1);
    // If this doesn't deadlock, this means that the blockStore call allowed the fiber releasing the barrier to run.
}
