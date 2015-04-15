#include "CachedBlock.h"
#include "Caching2BlockStore.h"

using std::unique_ptr;
using std::make_unique;

namespace blockstore {
namespace caching2 {

CachedBlock::CachedBlock(std::unique_ptr<Block> baseBlock, Caching2BlockStore *blockStore)
    :Block(baseBlock->key()),
     _blockStore(blockStore),
     _baseBlock(std::move(baseBlock)) {
}

CachedBlock::~CachedBlock() {
  if (_baseBlock.get() != nullptr) {
    _blockStore->release(std::move(_baseBlock));
  }
}

const void *CachedBlock::data() const {
  return _baseBlock->data();
}

void CachedBlock::write(const void *source, uint64_t offset, uint64_t size) {
  return _baseBlock->write(source, offset, size);
}

void CachedBlock::flush() {
  return _baseBlock->flush();
}

size_t CachedBlock::size() const {
  return _baseBlock->size();
}

unique_ptr<Block> CachedBlock::releaseBlock() {
  return std::move(_baseBlock);
}

}
}
