#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_CACHE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_CACHE_H_

#include "CacheEntry.h"
#include "QueueMap.h"
#include "PeriodicTask.h"
#include <memory>
#include <boost/optional.hpp>
#include <future>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/lock/MutexPoolLock.h>
#include <cpp-utils/pointer/gcc_4_8_compatibility.h>

namespace blockstore {
namespace caching {

template<class Key, class Value, uint32_t MAX_ENTRIES>
class Cache final {
public:
  //TODO Current MAX_LIFETIME_SEC only considers time since the element was last pushed to the Cache. Also insert a real MAX_LIFETIME_SEC that forces resync of entries that have been pushed/popped often (e.g. the root blob)
  //TODO Experiment with good values
  static constexpr double PURGE_LIFETIME_SEC = 0.5; //When an entry has this age, it will be purged from the cache
  static constexpr double PURGE_INTERVAL = 0.5; // With this interval, we check for entries to purge
  static constexpr double MAX_LIFETIME_SEC = PURGE_LIFETIME_SEC + PURGE_INTERVAL; // This is the oldest age an entry can reach (given purging works in an ideal world, i.e. with the ideal interval and in zero time)

  Cache(const std::string& cacheName);
  ~Cache();

  uint32_t size() const;

  void push(const Key &key, Value value);
  boost::optional<Value> pop(const Key &key);

  void flush();

private:
  void _makeSpaceForEntry(std::unique_lock<std::mutex> *lock);
  void _deleteEntry(std::unique_lock<std::mutex> *lock);
  void _deleteOldEntriesParallel();
  void _deleteAllEntriesParallel();
  void _deleteMatchingEntriesAtBeginningParallel(std::function<bool (const CacheEntry<Key, Value> &)> matches);
  void _deleteMatchingEntriesAtBeginning(std::function<bool (const CacheEntry<Key, Value> &)> matches);
  bool _deleteMatchingEntryAtBeginning(std::function<bool (const CacheEntry<Key, Value> &)> matches);

  mutable std::mutex _mutex;
  cpputils::LockPool<Key> _currentlyFlushingEntries;
  QueueMap<Key, CacheEntry<Key, Value>> _cachedBlocks;
  std::unique_ptr<PeriodicTask> _timeoutFlusher;

  DISALLOW_COPY_AND_ASSIGN(Cache);
};

template<class Key, class Value, uint32_t MAX_ENTRIES> constexpr double Cache<Key, Value, MAX_ENTRIES>::PURGE_LIFETIME_SEC;
template<class Key, class Value, uint32_t MAX_ENTRIES> constexpr double Cache<Key, Value, MAX_ENTRIES>::PURGE_INTERVAL;
template<class Key, class Value, uint32_t MAX_ENTRIES> constexpr double Cache<Key, Value, MAX_ENTRIES>::MAX_LIFETIME_SEC;

template<class Key, class Value, uint32_t MAX_ENTRIES>
Cache<Key, Value, MAX_ENTRIES>::Cache(const std::string& cacheName): _mutex(), _currentlyFlushingEntries(), _cachedBlocks(), _timeoutFlusher(nullptr) {
  //Don't initialize timeoutFlusher in the initializer list,
  //because it then might already call Cache::popOldEntries() before Cache is done constructing.
  _timeoutFlusher = std::make_unique<PeriodicTask>(std::bind(&Cache::_deleteOldEntriesParallel, this), PURGE_INTERVAL, "flush_" + cacheName);
}

template<class Key, class Value, uint32_t MAX_ENTRIES>
Cache<Key, Value, MAX_ENTRIES>::~Cache() {
  _deleteAllEntriesParallel();
  ASSERT(_cachedBlocks.size() == 0, "Error in _deleteAllEntriesParallel()");
}

template<class Key, class Value, uint32_t MAX_ENTRIES>
boost::optional<Value> Cache<Key, Value, MAX_ENTRIES>::pop(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  const cpputils::MutexPoolLock<Key> lockEntryFromBeingPopped(&_currentlyFlushingEntries, key, &lock);

  auto found = _cachedBlocks.pop(key);
  if (!found) {
    return boost::none;
  }
  return found->releaseValue();
}

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::push(const Key &key, Value value) {
  std::unique_lock<std::mutex> lock(_mutex);
  ASSERT(_cachedBlocks.size() <= MAX_ENTRIES, "Cache too full");
  _makeSpaceForEntry(&lock);
  _cachedBlocks.push(key, CacheEntry<Key, Value>(std::move(value)));
}

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::_makeSpaceForEntry(std::unique_lock<std::mutex> *lock) {
  // _deleteEntry releases the lock while the Value destructor is running.
  // So we can destruct multiple entries in parallel and also call pop() or push() while doing so.
  // However, if another thread calls push() before we get the lock back, the cache is full again.
  // That's why we need the while() loop here.
  while (_cachedBlocks.size() == MAX_ENTRIES) {
    _deleteEntry(lock);
  }
  ASSERT(_cachedBlocks.size() < MAX_ENTRIES, "Removing entry from cache didn't work");
};

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::_deleteEntry(std::unique_lock<std::mutex> *lock) {
  ASSERT(lock->owns_lock(), "The operations in this function require a locked mutex");
  auto key = _cachedBlocks.peekKey();
  ASSERT(key != boost::none, "There was no entry to delete");
  cpputils::MutexPoolLock<Key> lockEntryFromBeingPopped(&_currentlyFlushingEntries, *key);
  auto value = _cachedBlocks.pop();
  // Call destructor outside of the unique_lock,
  // i.e. pop() and push() can be called here, except for pop() on the element in _currentlyFlushingEntries
  lock->unlock();
  value = boost::none; // Call destructor
  lockEntryFromBeingPopped.unlock();  // unlock this one first to keep same locking oder (preventing potential deadlock)
  lock->lock();
};

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::_deleteAllEntriesParallel() {
  return _deleteMatchingEntriesAtBeginningParallel([] (const CacheEntry<Key, Value> &) {
      return true;
  });
}

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::_deleteOldEntriesParallel() {
  return _deleteMatchingEntriesAtBeginningParallel([] (const CacheEntry<Key, Value> &entry) {
      return entry.ageSeconds() > PURGE_LIFETIME_SEC;
  });
}

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::_deleteMatchingEntriesAtBeginningParallel(std::function<bool (const CacheEntry<Key, Value> &)> matches) {
  // Twice the number of cores, so we use full CPU even if half the threads are doing I/O
  const unsigned int numThreads = 2 * (std::max)(1u, std::thread::hardware_concurrency());
  std::vector<std::future<void>> waitHandles;
  for (unsigned int i = 0; i < numThreads; ++i) {
    waitHandles.push_back(std::async(std::launch::async, [this, matches] {
        _deleteMatchingEntriesAtBeginning(matches);
    }));
  }
  for (auto & waitHandle : waitHandles) {
    waitHandle.wait();
  }
};

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::_deleteMatchingEntriesAtBeginning(std::function<bool (const CacheEntry<Key, Value> &)> matches) {
  while (_deleteMatchingEntryAtBeginning(matches)) {}
}

template<class Key, class Value, uint32_t MAX_ENTRIES>
bool Cache<Key, Value, MAX_ENTRIES>::_deleteMatchingEntryAtBeginning(std::function<bool (const CacheEntry<Key, Value> &)> matches) {
  // This function can be called in parallel by multiple threads and will then cause the Value destructors
  // to be called in parallel. The call to _deleteEntry() releases the lock while the Value destructor is running.
  std::unique_lock<std::mutex> lock(_mutex);
  if (_cachedBlocks.size() > 0 && matches(*_cachedBlocks.peek())) {
    _deleteEntry(&lock);
    ASSERT(lock.owns_lock(), "Something strange happened with the lock. It should be locked again when we come back.");
    return true;
  } else {
    return false;
  }
};

template<class Key, class Value, uint32_t MAX_ENTRIES>
uint32_t Cache<Key, Value, MAX_ENTRIES>::size() const {
  std::unique_lock<std::mutex> lock(_mutex);
  return _cachedBlocks.size();
};

template<class Key, class Value, uint32_t MAX_ENTRIES>
void Cache<Key, Value, MAX_ENTRIES>::flush() {
  //TODO Test flush()
  return _deleteAllEntriesParallel();
};

}
}

#endif
