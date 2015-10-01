#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_H_

#include "CacheEntry.h"
#include "QueueMap.h"
#include "PeriodicTask.h"
#include <memory>
#include <boost/optional.hpp>
#include <future>
#include <messmer/cpp-utils/assert/assert.h>
#include <messmer/cpp-utils/lock/LockPool.h>

namespace blockstore {
namespace caching {

template<class Key, class Value>
class Cache {
public:
  static constexpr uint32_t MAX_ENTRIES = 1000;
  //TODO Experiment with good values
  static constexpr double PURGE_LIFETIME_SEC = 0.5; //When an entry has this age, it will be purged from the cache
  static constexpr double PURGE_INTERVAL = 0.5; // With this interval, we check for entries to purge
  static constexpr double MAX_LIFETIME_SEC = PURGE_LIFETIME_SEC + PURGE_INTERVAL; // This is the oldest age an entry can reach (given purging works in an ideal world, i.e. with the ideal interval and in zero time)

  Cache();
  virtual ~Cache();

  void push(const Key &key, Value value);
  boost::optional<Value> pop(const Key &key);

private:
  void _makeSpaceForEntry(std::unique_lock<std::mutex> *lock);
  void _deleteEntry(std::unique_lock<std::mutex> *lock);
  void _deleteOldEntriesParallel();
  void _deleteOldEntries();
  bool _deleteOldEntry();

  mutable std::mutex _mutex;
  cpputils::LockPool<Key> _currentlyFlushingEntries;
  QueueMap<Key, CacheEntry<Key, Value>> _cachedBlocks;
  std::unique_ptr<PeriodicTask> _timeoutFlusher;
};

template<class Key, class Value> constexpr uint32_t Cache<Key, Value>::MAX_ENTRIES;
template<class Key, class Value> constexpr double Cache<Key, Value>::PURGE_LIFETIME_SEC;
template<class Key, class Value> constexpr double Cache<Key, Value>::PURGE_INTERVAL;
template<class Key, class Value> constexpr double Cache<Key, Value>::MAX_LIFETIME_SEC;

template<class Key, class Value>
Cache<Key, Value>::Cache(): _cachedBlocks(), _timeoutFlusher(nullptr) {
  //Don't initialize timeoutFlusher in the initializer list,
  //because it then might already call Cache::popOldEntries() before Cache is done constructing.
  _timeoutFlusher = std::make_unique<PeriodicTask>(std::bind(&Cache::_deleteOldEntriesParallel, this), PURGE_INTERVAL);
}

template<class Key, class Value>
Cache<Key, Value>::~Cache() {
}

template<class Key, class Value>
boost::optional<Value> Cache<Key, Value>::pop(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  _currentlyFlushingEntries.lock(key, &lock);

  auto found = _cachedBlocks.pop(key);
  if (!found) {
    return boost::none;
  }

  _currentlyFlushingEntries.release(key);
  return found->releaseValue();
}

template<class Key, class Value>
void Cache<Key, Value>::push(const Key &key, Value value) {
  std::unique_lock<std::mutex> lock(_mutex);
  ASSERT(_cachedBlocks.size() <= MAX_ENTRIES, "Cache too full");
  _makeSpaceForEntry(&lock);
  _cachedBlocks.push(key, CacheEntry<Key, Value>(std::move(value)));
}

template<class Key, class Value>
void Cache<Key, Value>::_makeSpaceForEntry(std::unique_lock<std::mutex> *lock) {
  // _deleteEntry releases the lock while the Value destructor is running.
  // So we can destruct multiple entries in parallel and also call pop() or push() while doing so.
  // However, if another thread calls push() before we get the lock back, the cache is full again.
  // That's why we need the while() loop here.
  while (_cachedBlocks.size() == MAX_ENTRIES) {
    _deleteEntry(lock);
  }
  ASSERT(_cachedBlocks.size() < MAX_ENTRIES, "Removing entry from cache didn't work");
};

template<class Key, class Value>
void Cache<Key, Value>::_deleteEntry(std::unique_lock<std::mutex> *lock) {
  auto key = _cachedBlocks.peekKey();
  ASSERT(key != boost::none, "There was no entry to delete");
  _currentlyFlushingEntries.lock(*key);
  auto value = _cachedBlocks.pop();
  // Call destructor outside of the unique_lock,
  // i.e. pop() and push() can be called here, except for pop() on the element in _currentlyFlushingEntries
  lock->unlock();
  value = boost::none; // Call destructor
  lock->lock();
  _currentlyFlushingEntries.release(*key);
};

template<class Key, class Value>
void Cache<Key, Value>::_deleteOldEntriesParallel() {
  unsigned int numThreads = std::max(1u, std::thread::hardware_concurrency());
  std::vector<std::future<void>> waitHandles;
  for (unsigned int i = 0; i < numThreads; ++i) {
    waitHandles.push_back(std::async(std::launch::async, [this] {
      _deleteOldEntries();
    }));
  }
  for (auto & waitHandle : waitHandles) {
    waitHandle.wait();
  }
};

template<class Key, class Value>
void Cache<Key, Value>::_deleteOldEntries() {
  while (_deleteOldEntry()) {}
}

template<class Key, class Value>
bool Cache<Key, Value>::_deleteOldEntry() {
  // This function can be called in parallel by multiple threads and will then cause the Value destructors
  // to be called in parallel. The call to _deleteEntry() releases the lock while the Value destructor is running.
  std::unique_lock<std::mutex> lock(_mutex);
  if (_cachedBlocks.size() > 0 && _cachedBlocks.peek()->ageSeconds() > PURGE_LIFETIME_SEC) {
    _deleteEntry(&lock);
    return true;
  } else {
    return false;
  }
};

}
}

#endif
