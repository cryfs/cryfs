#include "testutils/CacheTest.h"
#include <chrono>
#include <thread>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/lock/ConditionBarrier.h>

using namespace std::chrono_literals;
using namespace blockstore::caching;
using std::string;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::ConditionBarrier;

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

// Regression test for a race condition.
// An element could be in the process of being thrown out of the cache and while the destructor is running, another
// thread calls pop() for the element and gets none returned. But since the destructor isn't finished yet, the data from
// the cache element also isn't completely written back yet and an application loading it runs into a race condition.
TEST(CacheTest_RaceCondition, PopBlocksWhileRequestedElementIsThrownOut) {
    ConditionBarrier destructorStarted;
    bool destructorFinished;

    auto obj = make_unique_ref<ObjectWithLongDestructor>(&destructorStarted, &destructorFinished);
    Cache<int, unique_ref<ObjectWithLongDestructor>> cache;
    cache.push(2, std::move(obj));

    destructorStarted.wait();
    EXPECT_FALSE(destructorFinished);
    cache.pop(2);
    EXPECT_TRUE(destructorFinished);
}