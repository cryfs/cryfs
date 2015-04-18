#include "CachedBlock.h"
#include "NewBlock.h"
#include "CachingBlockStore.h"
#include "../../interface/Block.h"

#include <algorithm>
#include <messmer/cpp-utils/pointer.h>

using std::unique_ptr;
using std::make_unique;
using cpputils::dynamic_pointer_move;

namespace blockstore {
namespace caching {

CachingBlockStore::CachingBlockStore(std::unique_ptr<BlockStore> baseBlockStore)
  :_baseBlockStore(std::move(baseBlockStore)), _cache(), _numNewBlocks(0) {
}

Key CachingBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

unique_ptr<Block> CachingBlockStore::tryCreate(const Key &key, Data data) {
  ++_numNewBlocks;
  return make_unique<CachedBlock>(make_unique<NewBlock>(key, std::move(data), this), this);
}

unique_ptr<Block> CachingBlockStore::load(const Key &key) {
  auto block = _cache.pop(key);
  if (block.get() != nullptr) {
    return make_unique<CachedBlock>(std::move(block), this);
  }
  block = _baseBlockStore->load(key);
  if (block.get() == nullptr) {
    return nullptr;
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
  //TODO Add number of NewBlock instances
  return _baseBlockStore->numBlocks() + _numNewBlocks;
}

void CachingBlockStore::release(unique_ptr<Block> block) {
  _cache.push(std::move(block));
}

std::unique_ptr<Block> CachingBlockStore::tryCreateInBaseStore(const Key &key, Data data) {
  auto block = _baseBlockStore->tryCreate(key, std::move(data));
  if (block.get() != nullptr) {
	--_numNewBlocks;
  }
  return block;
}

void CachingBlockStore::removeFromBaseStore(std::unique_ptr<Block> block) {
  _baseBlockStore->remove(std::move(block));
}

}
}
