#include "testutils/CacheTest.h"
#include <chrono>
#include <thread>
#include <memory>
#include <future>
#include <messmer/cpp-utils/lock/ConditionBarrier.h>

using namespace std::chrono_literals;
using namespace blockstore::caching;
using std::string;
using cpputils::ConditionBarrier;
using std::unique_ptr;
using std::make_unique;
using std::future;

// Regression tests for a race condition.
// An element could be in the process of being thrown out of the cache and while the destructor is running, another
// thread calls pop() for the element and gets none returned. But since the destructor isn't finished yet, the data from
// the cache element also isn't completely written back yet and an application loading it runs into a race condition.

class ObjectWithLongDestructor {
public:
    ObjectWithLongDestructor(ConditionBarrier *onDestructorStarted, bool *destructorFinished)
            : _onDestructorStarted(onDestructorStarted), _destructorFinished(destructorFinished) {}
    ~ObjectWithLongDestructor() {
        _onDestructorStarted->release();
        std::this_thread::sleep_for(1s);
        *_destructorFinished = true;
    }
private:
    ConditionBarrier *_onDestructorStarted;
    bool *_destructorFinished;
};

class CacheTest_RaceCondition: public ::testing::Test {
public:
    CacheTest_RaceCondition(): cache(), destructorStarted(), destructorFinished(false) {}

    Cache<int, unique_ptr<ObjectWithLongDestructor>> cache;
    ConditionBarrier destructorStarted;
    bool destructorFinished;

    int pushObjectWithLongDestructor() {
        cache.push(2, make_unique<ObjectWithLongDestructor>(&destructorStarted, &destructorFinished));
        return 2;
    }

    int pushDummyObject() {
        cache.push(3, nullptr);
        return 3;
    }

    future<void> causeCacheOverflowInOtherThread() {
        //Add maximum+1 element in another thread (this causes the cache to flush the first element in another thread)
        return std::async(std::launch::async, [this] {
            for(unsigned int i = 0; i < cache.MAX_ENTRIES+1; ++i) {
                cache.push(cache.MAX_ENTRIES+i, nullptr);
            }
        });
    }

    void EXPECT_POP_BLOCKS_UNTIL_DESTRUCTOR_FINISHED(int key) {
        EXPECT_FALSE(destructorFinished);
        cache.pop(key);
        EXPECT_TRUE(destructorFinished);
    }

    void EXPECT_POP_DOESNT_BLOCK_UNTIL_DESTRUCTOR_FINISHED(int key) {
        EXPECT_FALSE(destructorFinished);
        cache.pop(key);
        EXPECT_FALSE(destructorFinished);
    }
};

TEST_F(CacheTest_RaceCondition, PopBlocksWhileRequestedElementIsThrownOut_ByAge) {
    auto id = pushObjectWithLongDestructor();

    destructorStarted.wait();
    EXPECT_POP_BLOCKS_UNTIL_DESTRUCTOR_FINISHED(id);
}

TEST_F(CacheTest_RaceCondition, PopDoesntBlockWhileOtherElementIsThrownOut_ByAge) {
    pushObjectWithLongDestructor();
    auto id = pushDummyObject();

    destructorStarted.wait();
    EXPECT_POP_DOESNT_BLOCK_UNTIL_DESTRUCTOR_FINISHED(id);
}

TEST_F(CacheTest_RaceCondition, PopBlocksWhileRequestedElementIsThrownOut_ByPush) {
    auto id = pushObjectWithLongDestructor();

    auto future = causeCacheOverflowInOtherThread();
    destructorStarted.wait();
    EXPECT_POP_BLOCKS_UNTIL_DESTRUCTOR_FINISHED(id);
}

TEST_F(CacheTest_RaceCondition, PopDoesntBlockWhileOtherElementIsThrownOut_ByPush) {
    pushObjectWithLongDestructor();
    auto id = pushDummyObject();

    auto future = causeCacheOverflowInOtherThread();
    destructorStarted.wait();
    EXPECT_POP_DOESNT_BLOCK_UNTIL_DESTRUCTOR_FINISHED(id);
}
