#include "Cache.h"

using std::unique_ptr;
using std::make_unique;
using std::mutex;
using std::lock_guard;
using std::pair;

namespace blockstore {
namespace caching2 {

constexpr uint32_t Cache::MAX_ENTRIES;

Cache::Cache(): _cachedBlocks() {
}

Cache::~Cache() {
}

unique_ptr<Block> Cache::pop(const Key &key) {
  lock_guard<mutex> lock(_mutex);
  auto found = _cachedBlocks.find(key);
  if (found == _cachedBlocks.end()) {
    return nullptr;
  }
  auto block = found->second.releaseBlock();
  _cachedBlocks.erase(found);
  return block;
}

void Cache::push(unique_ptr<Block> block) {
  lock_guard<mutex> lock(_mutex);
  if (_cachedBlocks.size() > MAX_ENTRIES) {
    deleteOldestEntry();
    assert(_cachedBlocks.size() == MAX_ENTRIES-1);
  }
  Key key = block->key();
  _cachedBlocks.emplace(key, std::move(block));
}

void Cache::deleteOldestEntry() {
  auto oldestEntry = std::min_element(_cachedBlocks.begin(), _cachedBlocks.end(), [] (const pair<const Key, CacheEntry> &lhs, const pair<const Key, CacheEntry> &rhs) {
    return lhs.second.ageSeconds() > rhs.second.ageSeconds();
  });
  //printf("Deleting age %f (vs %f)\n", oldestEntry->second.ageSeconds(), _cachedBlocks.begin()->second.ageSeconds());
  _cachedBlocks.erase(oldestEntry);
}

}
}
