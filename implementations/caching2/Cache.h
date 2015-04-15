#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHE_H_

#include "../../interface/Block.h"
#include "CacheEntry.h"
#include "QueueMap.h"
#include <memory>
#include <mutex>

namespace blockstore {
namespace caching2 {

//TODO Test
//TODO Also throw blocks out after a timeout

class Cache {
public:
  static constexpr uint32_t MAX_ENTRIES = 1000;

  Cache();
  virtual ~Cache();

  void push(std::unique_ptr<Block> block);
  std::unique_ptr<Block> pop(const Key &key);

private:
  mutable std::mutex _mutex;
  QueueMap<Key, CacheEntry> _cachedBlocks;
};

}
}

#endif
