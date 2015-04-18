#include "Cache.h"
#include "PeriodicTask.h"

using std::unique_ptr;
using std::make_unique;
using std::mutex;
using std::lock_guard;
using std::pair;

namespace blockstore {
namespace caching {

constexpr uint32_t Cache::MAX_ENTRIES;
constexpr double Cache::PURGE_LIFETIME_SEC;
constexpr double Cache::PURGE_INTERVAL;
constexpr double Cache::MAX_LIFETIME_SEC;

Cache::Cache(): _cachedBlocks(), _timeoutFlusher(nullptr) {
  //Don't initialize timeoutFlusher in the initializer list,
  //because it then might already call Cache::popOldEntries() before Cache is done constructing
  _timeoutFlusher = make_unique<PeriodicTask>(std::bind(&Cache::_popOldEntries, this), PURGE_INTERVAL);
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

void Cache::_popOldEntries() {
  lock_guard<mutex> lock(_mutex);
  while(_cachedBlocks.size() > 0 && _cachedBlocks.peek().ageSeconds() > PURGE_LIFETIME_SEC) {
	double age = _cachedBlocks.peek().ageSeconds();
	printf("Removing block with age: %f\n", age);
	_cachedBlocks.pop();
  }
}

}
}
