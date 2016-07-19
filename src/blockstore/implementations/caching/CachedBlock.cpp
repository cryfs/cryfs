#include "CachedBlock.h"
#include "CachingBlockStore.h"

using cpputils::unique_ref;

namespace blockstore {
namespace caching {

CachedBlock::CachedBlock(BaseBlockWrapper baseBlock, CachingBlockStore *blockStore)
    :Block(baseBlock.key()),
     _blockStore(blockStore),
     _baseBlock(std::move(baseBlock)) {
}

CachedBlock::~CachedBlock() {
  _blockStore->returnToCache(std::move(_baseBlock));
}

const void *CachedBlock::data() const {
  return _baseBlock.data();
}

void CachedBlock::write(const void *source, uint64_t offset, uint64_t size) {
  return _baseBlock.write(source, offset, size);
}

void CachedBlock::flush() {
  return _baseBlock.flush();
}

size_t CachedBlock::size() const {
  return _baseBlock.size();
}

void CachedBlock::resize(size_t newSize) {
  return _baseBlock.resize(newSize);
}

}
}
