#include <gtest/gtest.h>

#include "blockstore/implementations/caching/cache/PeriodicTask.h"

#include <mutex>
#include <condition_variable>
#include <atomic>

using ::testing::Test;
using std::mutex;
using std::unique_lock;
using std::condition_variable;

using namespace blockstore::caching;

class AtomicCounter {
public:
  AtomicCounter(int count): _mutex(), _cv(), _counter(count) {}

  void decrease() {
    unique_lock<mutex> lock(_mutex);
    --_counter;
    _cv.notify_all();
  }

  void waitForZero() {
    unique_lock<mutex> lock(_mutex);
    _cv.wait(lock, [this] () {return _counter <= 0;});
  }
private:
  mutex _mutex;
  condition_variable _cv;
  int _counter;
};

class PeriodicTaskTest: public Test {
};

TEST_F(PeriodicTaskTest, DoesntDeadlockInDestructorWhenDestructedImmediately) {
  PeriodicTask task([](){}, 1, "test");
}

TEST_F(PeriodicTaskTest, CallsCallbackAtLeast10Times) {
  AtomicCounter counter(10);

  PeriodicTask task([&counter](){
    counter.decrease();
  }, 0.001, "test");

  counter.waitForZero();
}

TEST_F(PeriodicTaskTest, DoesntCallCallbackAfterDestruction) {
  std::atomic<int> callCount(0);
  {
    PeriodicTask task([&callCount](){
      callCount += 1;
    }, 0.001, "test");
  }
  int callCountDirectlyAfterDestruction = callCount;
  boost::this_thread::sleep_for(boost::chrono::seconds(1));
  EXPECT_EQ(callCountDirectlyAfterDestruction, callCount);
}
