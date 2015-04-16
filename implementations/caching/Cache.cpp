#include "Cache.h"

using std::unique_ptr;
using std::make_unique;
using std::mutex;
using std::lock_guard;
using std::pair;

namespace blockstore {
namespace caching {

constexpr uint32_t Cache::MAX_ENTRIES;

Cache::Cache(): _cachedBlocks() {
}

Cache::~Cache() {
}

unique_ptr<Block> Cache::pop(const Key &key) {
  lock_guard<mutex> lock(_mutex);
  auto found = _cachedBlocks.pop(key);
  if (found.get() == nullptr) {
    return nullptr;
  }
  auto block = found->releaseBlock();
  return block;
}

void Cache::push(unique_ptr<Block> block) {
  lock_guard<mutex> lock(_mutex);
  assert(_cachedBlocks.size() <= MAX_ENTRIES);
  if (_cachedBlocks.size() == MAX_ENTRIES) {
    _cachedBlocks.pop();
    assert(_cachedBlocks.size() == MAX_ENTRIES-1);
  }
  Key key = block->key();
  _cachedBlocks.push(key, make_unique<CacheEntry>(std::move(block)));
}

}
}
