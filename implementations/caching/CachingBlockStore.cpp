#include "CachedBlock.h"
#include "NewBlock.h"
#include "CachingBlockStore.h"
#include "../../interface/Block.h"

#include <algorithm>
#include <messmer/cpp-utils/pointer/cast.h>

using std::unique_ptr;
using std::make_unique;
using cpputils::dynamic_pointer_move;
using cpputils::Data;
using boost::optional;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::none;

namespace blockstore {
namespace caching {

CachingBlockStore::CachingBlockStore(std::unique_ptr<BlockStore> baseBlockStore)
  :_baseBlockStore(std::move(baseBlockStore)), _cache(), _numNewBlocks(0) {
}

Key CachingBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

optional<unique_ref<Block>> CachingBlockStore::tryCreate(const Key &key, Data data) {
  //TODO Shouldn't we return boost::none if the key already exists?
  ++_numNewBlocks;
  return unique_ref<Block>(make_unique_ref<CachedBlock>(make_unique<NewBlock>(key, std::move(data), this), this));
}

unique_ptr<Block> CachingBlockStore::load(const Key &key) {
  boost::optional<unique_ptr<Block>> optBlock = _cache.pop(key);
  unique_ptr<Block> block;
  //TODO an optional<> class with .getOrElse() would make this code simpler. boost::optional<>::value_or_eval didn't seem to work with unique_ptr members.
  if (optBlock) {
    block = std::move(*optBlock);
  } else {
    block = _baseBlockStore->load(key);
    if (block.get() == nullptr) {
      return nullptr;
    }
  }
  return make_unique<CachedBlock>(std::move(block), this);
}

void CachingBlockStore::remove(std::unique_ptr<Block> block) {
  auto baseBlock = dynamic_pointer_move<CachedBlock>(block)->releaseBlock();
  auto baseNewBlock = dynamic_pointer_move<NewBlock>(baseBlock);
  if (baseNewBlock.get() != nullptr) {
	if(!baseNewBlock->alreadyExistsInBaseStore()) {
	  --_numNewBlocks;
	}
	baseNewBlock->remove();
  } else {
    _baseBlockStore->remove(std::move(baseBlock));
  }
}

uint64_t CachingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks() + _numNewBlocks;
}

void CachingBlockStore::release(unique_ptr<Block> block) {
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

void CachingBlockStore::removeFromBaseStore(std::unique_ptr<Block> block) {
  _baseBlockStore->remove(std::move(block));
}

}
}
