#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHE_H_

#include "../../interface/Block.h"
#include "CacheEntry.h"
#include <memory>
#include <mutex>

namespace blockstore {
namespace caching2 {

class Cache {
public:
  static constexpr uint32_t MAX_ENTRIES = 1000;

  Cache();
  virtual ~Cache();

  void push(std::unique_ptr<Block> block);
  std::unique_ptr<Block> pop(const Key &key);

private:
  mutable std::mutex _mutex;
  std::map<Key, CacheEntry> _cachedBlocks;

  void deleteOldestEntry();
};

}
}

#endif
