#include "CachingBlockStore2.h"
#include <memory>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/get_total_memory.h>

using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::piecewise_construct;
using std::make_tuple;
using std::make_pair;
using std::vector;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;
using std::unique_lock;
using std::mutex;

namespace blockstore {
namespace caching {

CachingBlockStore2::CachedBlock::CachedBlock(const CachingBlockStore2* blockStore, const Key& key, cpputils::Data data, bool isDirty)
    : _blockStore(blockStore), _key(key), _data(std::move(data)), _dirty(isDirty) {
}

CachingBlockStore2::CachedBlock::~CachedBlock() {
  if (_dirty) {
    _blockStore->_baseBlockStore->store(_key, _data);
  }
  // remove it from the list of blocks not in the base store, if it's on it
  unique_lock<mutex> lock(_blockStore->_cachedBlocksNotInBaseStoreMutex);
  _blockStore->_cachedBlocksNotInBaseStore.erase(_key);
}

const Data& CachingBlockStore2::CachedBlock::read() const {
  return _data;
}

void CachingBlockStore2::CachedBlock::markNotDirty() && {
  _dirty = false; // Prevent writing it back into the base store
}

void CachingBlockStore2::CachedBlock::write(Data data) {
  _data = std::move(data);
  _dirty = true;
}

CachingBlockStore2::CachingBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore)
: _baseBlockStore(std::move(baseBlockStore)), _cachedBlocksNotInBaseStoreMutex(), _cachedBlocksNotInBaseStore(), _cache() {
}

bool CachingBlockStore2::tryCreate(const Key &key, const Data &data) {
  //TODO Check if block exists in base store? Performance hit? It's very unlikely it exists.
  auto popped = _cache.pop(key);
  if (popped != boost::none) {
    // entry already exists in cache
    _cache.push(key, std::move(*popped)); // push the just popped element back to the cache
    return false;
  } else {
    _cache.push(key, make_unique_ref<CachingBlockStore2::CachedBlock>(this, key, data.copy(), true));
    unique_lock<mutex> lock(_cachedBlocksNotInBaseStoreMutex);
    _cachedBlocksNotInBaseStore.insert(key);
    return true;
  }
}

bool CachingBlockStore2::remove(const Key &key) {
  // TODO Don't write-through but cache remove operations
  auto popped = _cache.pop(key);
  if (popped != boost::none) {
    // Remove from base store if it exists in the base store
    {
      unique_lock<mutex> lock(_cachedBlocksNotInBaseStoreMutex);
      if (_cachedBlocksNotInBaseStore.count(key) == 0) {
          const bool existedInBaseStore = _baseBlockStore->remove(key);
          if (!existedInBaseStore) {
              throw std::runtime_error("Tried to remove block. Block existed in cache and stated it exists in base store, but wasn't found there.");
          }
      }
    }
    // Don't write back the cached block when it is destructed
    std::move(**popped).markNotDirty();
    return true;
  } else {
    return _baseBlockStore->remove(key);
  }
}

optional<unique_ref<CachingBlockStore2::CachedBlock>> CachingBlockStore2::_loadFromCacheOrBaseStore(const Key &key) const {
  auto popped = _cache.pop(key);
  if (popped != boost::none) {
    return std::move(*popped);
  } else {
    auto loaded = _baseBlockStore->load(key);
    if (loaded == boost::none) {
      return boost::none;
    }
    return make_unique_ref<CachingBlockStore2::CachedBlock>(this, key, std::move(*loaded), false);
  }
}

optional<Data> CachingBlockStore2::load(const Key &key) const {
  auto loaded = _loadFromCacheOrBaseStore(key);
  if (loaded == boost::none) {
    // TODO Cache non-existence?
    return boost::none;
  }
  optional<Data> result = (*loaded)->read().copy();
  _cache.push(key, std::move(*loaded));
  return result;
}

void CachingBlockStore2::store(const Key &key, const Data &data) {
  auto popped = _cache.pop(key);
  if (popped != boost::none) {
    (*popped)->write(data.copy());
  } else {
    popped = make_unique_ref<CachingBlockStore2::CachedBlock>(this, key, data.copy(), false);
    // TODO Instead of storing it to the base store, we could just keep it dirty in the cache
    //      and (if it doesn't exist in base store yet) add it to _cachedBlocksNotInBaseStore
    _baseBlockStore->store(key, data);
  }
  _cache.push(key, std::move(*popped));
}

uint64_t CachingBlockStore2::numBlocks() const {
  uint64_t numInCacheButNotInBaseStore = 0;
  {
    unique_lock<mutex> lock(_cachedBlocksNotInBaseStoreMutex);
    numInCacheButNotInBaseStore = _cachedBlocksNotInBaseStore.size();
  }
  return _baseBlockStore->numBlocks() + numInCacheButNotInBaseStore;
}

uint64_t CachingBlockStore2::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

uint64_t CachingBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

void CachingBlockStore2::forEachBlock(std::function<void (const Key &)> callback) const {
  {
    unique_lock<mutex> lock(_cachedBlocksNotInBaseStoreMutex);
    for (const Key &key : _cachedBlocksNotInBaseStore) {
      callback(key);
    }
  }
  _baseBlockStore->forEachBlock(std::move(callback));
}

void CachingBlockStore2::flush() {
    _cache.flush();
}

}
}
