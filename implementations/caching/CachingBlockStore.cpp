#include "CachedBlock.h"
#include "NewBlock.h"
#include "CachingBlockStore.h"
#include "../../interface/Block.h"

#include <algorithm>
#include <messmer/cpp-utils/pointer/cast.h>

using cpputils::dynamic_pointer_move;
using cpputils::Data;
using boost::optional;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::none;

namespace blockstore {
namespace caching {

CachingBlockStore::CachingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore)
  :_baseBlockStore(std::move(baseBlockStore)), _cache(), _numNewBlocks(0) {
}

Key CachingBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

optional<unique_ref<Block>> CachingBlockStore::tryCreate(const Key &key, Data data) {
  //TODO Shouldn't we return boost::none if the key already exists?
  ++_numNewBlocks;
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
  assert(cached_block != none);
  auto baseBlock = (*cached_block)->releaseBlock();
  auto baseNewBlock = dynamic_pointer_move<NewBlock>(baseBlock);
  if (baseNewBlock != none) {
	if(!(*baseNewBlock)->alreadyExistsInBaseStore()) {
	  --_numNewBlocks;
	}
    (*baseNewBlock)->remove();
  } else {
    _baseBlockStore->remove(std::move(baseBlock));
  }
}

uint64_t CachingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks() + _numNewBlocks;
}

void CachingBlockStore::release(unique_ref<Block> block) {
  Key key = block->key();
  _cache.push(key, std::move(block));
}

optional<unique_ref<Block>> CachingBlockStore::tryCreateInBaseStore(const Key &key, Data data) {
  auto block = _baseBlockStore->tryCreate(key, std::move(data));
  if (block != none) {
	--_numNewBlocks;
  }
  return block;
}

void CachingBlockStore::removeFromBaseStore(cpputils::unique_ref<Block> block) {
  _baseBlockStore->remove(std::move(block));
}

}
}
