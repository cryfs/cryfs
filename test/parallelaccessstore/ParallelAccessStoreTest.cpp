#include <gtest/gtest.h>
#include <parallelaccessstore/ParallelAccessStore.h>
#include <blockstore/utils/BlockId.h>
#include <cpp-utils/data/SerializationHelper.h>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <array>
#include <atomic>
#include <chrono>
#include <condition_variable>
#include <cstdlib>
#include <iostream>
#include <mutex>
#include <string>
#include <thread>

using blockstore::BlockId;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using parallelaccessstore::ParallelAccessBaseStore;
using parallelaccessstore::ParallelAccessStore;

// Regression tests for a race condition in ParallelAccessStore::remove(key).
// It looked the key up in _openResources without holding the lock, while load() and release() add to and erase from
// _openResources under the lock in other threads. In CryFS, release() runs on the cache flusher's threads and
// remove(key) is called for every node that gets deleted. Besides being a data race on the map, the lookup and the
// registration of the promise for the resource were two separate critical sections, so release() could erase the
// resource in between and remove() would then wait forever for a promise nobody fulfils.
// Run these under ThreadSanitizer to catch the unlocked lookup, a plain run only catches crashes and hangs.

namespace {

class Resource final {
public:
  explicit Resource(const BlockId &blockId): _blockId(blockId) {}

  const BlockId &blockId() const {
    return _blockId;
  }

private:
  BlockId _blockId;

  DISALLOW_COPY_AND_ASSIGN(Resource);
};

class ResourceRef final: public ParallelAccessStore<Resource, ResourceRef, BlockId>::ResourceRefBase {
public:
  explicit ResourceRef(Resource *baseResource): _baseResource(baseResource) {}

  const BlockId &blockId() const {
    return _baseResource->blockId();
  }

private:
  Resource *_baseResource;

  DISALLOW_COPY_AND_ASSIGN(ResourceRef);
};

// Has every key, i.e. each load hands out a fresh resource. Counts the removals, so the tests can check that each
// remove() call reached the base store, and through which of the two removeFromBaseStore() overloads it did.
class FakeBaseStore final: public ParallelAccessBaseStore<Resource, BlockId> {
public:
  FakeBaseStore(std::atomic<unsigned int> *numRemovedByResource, std::atomic<unsigned int> *numRemovedByKey)
      : _numRemovedByResource(numRemovedByResource), _numRemovedByKey(numRemovedByKey) {}

  boost::optional<unique_ref<Resource>> loadFromBaseStore(const BlockId &blockId) override {
    return make_unique_ref<Resource>(blockId);
  }

  void removeFromBaseStore(unique_ref<Resource> /*resource*/) override {
    ++*_numRemovedByResource;
  }

  void removeFromBaseStore(const BlockId & /*blockId*/) override {
    ++*_numRemovedByKey;
  }

private:
  std::atomic<unsigned int> *_numRemovedByResource;
  std::atomic<unsigned int> *_numRemovedByKey;

  DISALLOW_COPY_AND_ASSIGN(FakeBaseStore);
};

BlockId blockId(unsigned int index) {
  // std::hash<BlockId> only looks at the first bytes of the id, so that's where the index goes.
  std::array<unsigned char, BlockId::BINARY_LENGTH> data{};
  cpputils::serialize<unsigned int>(data.data(), index);
  return BlockId::FromBinary(data.data());
}

// Generous on purpose: both tests run in well under a second natively and the point of the limit is
// only to turn a hang into a failure, not to measure anything.
constexpr std::chrono::seconds TEST_TIMEOUT(300);

// gtest has no per-test timeout and the hang these tests guard against happens inside
// ParallelAccessStore::remove(), i.e. below the join() the test would be sitting in, so a regression
// would block until the CI job's six hour limit and print nothing - run_tests only shows a test
// binary's output once that binary has finished. Abort instead, so the failure is immediate and the
// core dump CI keeps shows which thread is stuck. Failing and returning is not an option: the stuck
// thread still uses the fixture, so letting the test finish underneath it would be undefined behavior.
class AbortOnTimeout final {
public:
  explicit AbortOnTimeout(std::chrono::seconds timeout, std::string message)
      : _message(std::move(message)), _finished(false),
        _watchdog([this, timeout] {
          std::unique_lock<std::mutex> lock(_mutex);
          if (!_finishedCondition.wait_for(lock, timeout, [this] { return _finished; })) {
            std::cerr << "Timeout: " << _message << std::endl;
            std::abort();
          }
        }) {}

  ~AbortOnTimeout() {
    {
      const std::lock_guard<std::mutex> lock(_mutex);
      _finished = true;
    }
    _finishedCondition.notify_all();
    _watchdog.join();
  }

private:
  std::string _message;
  bool _finished;
  std::mutex _mutex;
  std::condition_variable _finishedCondition;
  std::thread _watchdog;

  DISALLOW_COPY_AND_ASSIGN(AbortOnTimeout);
};

class ParallelAccessStoreTest: public ::testing::Test {
public:
  ParallelAccessStoreTest()
      : numRemovedByResource(0), numRemovedByKey(0),
        store(make_unique_ref<FakeBaseStore>(&numRemovedByResource, &numRemovedByKey)) {}

  static constexpr unsigned int NUM_ITERATIONS = 2000;
  static constexpr unsigned int NUM_LOAD_KEYS = 10;

  std::atomic<unsigned int> numRemovedByResource;
  std::atomic<unsigned int> numRemovedByKey;
  ParallelAccessStore<Resource, ResourceRef, BlockId> store;
};

}

// The loading thread keeps loading and releasing a few keys of its own. The removing thread calls remove(key) on
// keys that are never opened and remove(key, ref) on resources only it holds, so neither call has to wait, but each
// remove(key) looks the key up in the map the loading thread is modifying.
TEST_F(ParallelAccessStoreTest, RemoveWhileOtherThreadLoadsAndReleases) {
  const AbortOnTimeout abortOnTimeout(TEST_TIMEOUT,
      "ParallelAccessStoreTest.RemoveWhileOtherThreadLoadsAndReleases did not finish. It only ever blocks "
      "if remove() waits for a promise nobody will fulfil, i.e. the bug this test guards against is back.");
  std::thread loader([this] {
    for (unsigned int i = 0; i < NUM_ITERATIONS; ++i) {
      const auto ref = store.load(blockId(i % NUM_LOAD_KEYS));
      EXPECT_TRUE(ref != boost::none);
      // ref goes out of scope here, which releases the resource
    }
  });
  std::thread remover([this] {
    for (unsigned int i = 0; i < NUM_ITERATIONS; ++i) {
      store.remove(blockId(NUM_LOAD_KEYS + 2*i));
      auto ref = store.load(blockId(NUM_LOAD_KEYS + 2*i + 1));
      ASSERT_TRUE(ref != boost::none);
      store.remove(blockId(NUM_LOAD_KEYS + 2*i + 1), std::move(*ref));
    }
  });
  loader.join();
  remover.join();

  EXPECT_EQ(NUM_ITERATIONS, numRemovedByKey.load());
  EXPECT_EQ(NUM_ITERATIONS, numRemovedByResource.load());
}

// The removing thread removes exactly the keys the loading thread loads, and the two threads hand each key over so
// that remove() finds the resource still open: the loading thread keeps the resource open until the removing thread
// announces that it is about to call remove(), and only then drops its reference. remove() then has to register a
// promise and wait for the release, which is the path the bug was in, and numRemovedByResource counts how often that
// path was taken. Without the handshake, remove() almost always found the resource already released and the test
// exercised the other branch instead.
TEST_F(ParallelAccessStoreTest, RemoveKeysWhileOtherThreadReleasesThem) {
  const AbortOnTimeout abortOnTimeout(TEST_TIMEOUT,
      "ParallelAccessStoreTest.RemoveKeysWhileOtherThreadReleasesThem did not finish. It only ever blocks "
      "if remove() waits for a promise nobody will fulfil, i.e. the bug this test guards against is back.");
  std::atomic<unsigned int> numKeysOpened(0);     // the resource for key i is open once this is greater than i
  std::atomic<unsigned int> numRemovesStarted(0); // remove() is about to be called for key i once this is greater than i

  std::thread loader([this, &numKeysOpened, &numRemovesStarted] {
    for (unsigned int i = 0; i < NUM_ITERATIONS; ++i) {
      const auto ref = store.load(blockId(i));
      EXPECT_TRUE(ref != boost::none);
      numKeysOpened.store(i + 1);
      while (numRemovesStarted.load() <= i) {
        std::this_thread::yield();
      }
      // ref goes out of scope here, which releases the resource and fulfils the promise remove() registered for it
    }
  });
  std::thread remover([this, &numKeysOpened, &numRemovesStarted] {
    for (unsigned int i = 0; i < NUM_ITERATIONS; ++i) {
      while (numKeysOpened.load() <= i) {
        std::this_thread::yield();
      }
      numRemovesStarted.store(i + 1);
      store.remove(blockId(i));
    }
  });
  loader.join();
  remover.join();

  // Each key is loaded once and removed once, so every remove() has to reach the base store through exactly one of
  // the two overloads, whether it waited for the release or found the resource already gone.
  EXPECT_EQ(NUM_ITERATIONS, numRemovedByKey.load() + numRemovedByResource.load());
  // Which of the two it is, is still a race - the removing thread can lose it and find the resource already
  // released - so this can't be an equality, but the handshake makes it the waiting path almost every time.
  EXPECT_GT(numRemovedByResource.load(), 0u);
}
