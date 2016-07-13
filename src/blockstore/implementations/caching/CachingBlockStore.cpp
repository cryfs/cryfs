#include "CachedBlock.h"
#include "NewBlock.h"
#include "CachingBlockStore.h"
#include "../../interface/Block.h"

#include <algorithm>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/assert/assert.h>

using cpputils::dynamic_pointer_move;
using cpputils::Data;
using boost::optional;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::none;
using std::mutex;
using std::unique_lock;

namespace blockstore {
namespace caching {

CachingBlockStore::CachingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore)
  :_baseBlockStore(std::move(baseBlockStore)), _newBlocks(), _cache() {
}

Key CachingBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

optional<unique_ref<Block>> CachingBlockStore::tryCreate(const Key &key, Data data) {
  ASSERT(_cache.pop(key) == none, "Key already exists in cache");
  //TODO Shouldn't we return boost::none if the key already exists?
  //TODO Key can also already exist but not be in the cache right now.
  return unique_ref<Block>(make_unique_ref<CachedBlock>(make_unique_ref<NewBlock>(key, std::move(data), this), this));
}

optional<unique_ref<Block>> CachingBlockStore::load(const Key &key) {
  optional<unique_ref<Block>> optBlock = _cache.pop(key);
  //TODO an optional<> class with .getOrElse() would make this code simpler. boost::optional<>::value_or_eval didn't seem to work with unique_ptr members.
  if (optBlock != none) {
    return optional<unique_ref<Block>>(make_unique_ref<CachedBlock>(std::move(*optBlock), this));
  } else {
    auto block = _baseBlockStore->load(key);
    if (block == none) {
      return none;
    } else {
      return optional<unique_ref<Block>>(make_unique_ref<CachedBlock>(std::move(*block), this));
    }
  }
}

void CachingBlockStore::remove(cpputils::unique_ref<Block> block) {
  auto cached_block = dynamic_pointer_move<CachedBlock>(block);
  ASSERT(cached_block != none, "Passed block is not a CachedBlock");
  auto baseBlock = (*cached_block)->releaseBlock();
  auto baseNewBlock = dynamic_pointer_move<NewBlock>(baseBlock);
  if (baseNewBlock != none) {
    (*baseNewBlock)->remove();
  } else {
    _baseBlockStore->remove(std::move(baseBlock));
  }
}

void CachingBlockStore::remove(const Key &key) {
  auto fromCache = _cache.pop(key);
  if (fromCache != none) {
    remove(make_unique_ref<CachedBlock>(std::move(*fromCache), this));
  } else {
    _baseBlockStore->remove(key);
  }
}

uint64_t CachingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks() + _newBlocks.size();
}

uint64_t CachingBlockStore::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

void CachingBlockStore::release(unique_ref<Block> block) {
  Key key = block->key();
  _cache.push(key, std::move(block));
}

optional<unique_ref<Block>> CachingBlockStore::tryCreateInBaseStore(const Key &key, Data data) {
  return _baseBlockStore->tryCreate(key, std::move(data));
}

void CachingBlockStore::removeFromBaseStore(cpputils::unique_ref<Block> block) {
  _baseBlockStore->remove(std::move(block));
}

void CachingBlockStore::flush() {
  _cache.flush();
}

uint64_t CachingBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

void CachingBlockStore::forEachBlock(std::function<void (const Key &)> callback) const {
  _baseBlockStore->forEachBlock(callback);
  for (NewBlock *newBlock : _newBlocks) {
    callback(newBlock->key());
  }
}

void CachingBlockStore::registerNewBlock(NewBlock *newBlock) {
  unique_lock<mutex> lock(_newBlocksMutex);
  _newBlocks.insert(newBlock);
}

void CachingBlockStore::unregisterNewBlock(NewBlock *newBlock) {
  unique_lock<mutex> lock(_newBlocksMutex);
  _newBlocks.erase(newBlock);
}

}
}
