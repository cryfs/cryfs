#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHE_H_

#include "CacheEntry.h"
#include "QueueMap.h"
#include "../../interface/Block.h"
#include <memory>
#include <mutex>

namespace blockstore {
namespace caching {
class PeriodicTask;

//TODO Test

class Cache {
public:
  static constexpr uint32_t MAX_ENTRIES = 1000;
  //TODO Experiment with good values
  static constexpr double PURGE_LIFETIME_SEC = 0.5; //When an entry has this age, it will be purged from the cache
  static constexpr double PURGE_INTERVAL = 0.5; // With this interval, we check for entries to purge
  static constexpr double MAX_LIFETIME_SEC = PURGE_LIFETIME_SEC + PURGE_INTERVAL; // This is the oldest age an entry can reach (given purging works in an ideal world, i.e. with the ideal interval and in zero time)

  Cache();
  virtual ~Cache();

  void push(std::unique_ptr<Block> block);
  std::unique_ptr<Block> pop(const Key &key);

private:
  void _popOldEntries();

  mutable std::mutex _mutex;
  QueueMap<Key, CacheEntry> _cachedBlocks;
  std::unique_ptr<PeriodicTask> _timeoutFlusher;
};

}
}

#endif
