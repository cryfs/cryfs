#include "CachedBlock.h"
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
  :_baseBlockStore(std::move(baseBlockStore)) {
}

unique_ptr<Block> CachingBlockStore::create(size_t size) {
  //TODO Also cache this and only write back in the destructor?
  //     When writing back is done efficiently in the base store (e.g. only one safe-to-disk, not one in the create() and then one in the save(), this is not supported by the current BlockStore interface),
  //     then the base store could actually directly create a block in the create() call, OnDiskBlockStore wouldn't have to avoid file creation in the create() call for performance reasons and I could also adapt the OnDiskBlockStore test cases and remove a lot of flush() calls there because then blocks are loadable directly after the create call() without a flush.
  //     Currently, OnDiskBlockStore doesn't create new blocks directly but only after they're destructed (performance reasons), but this means a newly created block can't be loaded directly.
  return make_unique<CachedBlock>(_baseBlockStore->create(size), this);
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
  return _baseBlockStore->remove(std::move(dynamic_pointer_move<CachedBlock>(block)->releaseBlock()));
}

uint64_t CachingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

void CachingBlockStore::release(unique_ptr<Block> block) {
  _cache.push(std::move(block));
}

}
}
