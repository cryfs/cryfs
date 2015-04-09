#include "CachedBlockRef.h"
#include "CachingBlockStore.h"
#include <cassert>
#include <messmer/cpp-utils/pointer.h>

#include "CachingBlockStoreAdapter.h"

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::promise;
using cpputils::dynamic_pointer_move;

namespace blockstore {
namespace caching {

CachingBlockStore::CachingBlockStore(unique_ptr<BlockStore> baseBlockStore)
 : _baseBlockStore(std::move(baseBlockStore)), _cachingStore(make_unique<CachingBlockStoreAdapter>(_baseBlockStore.get())) {
}

unique_ptr<Block> CachingBlockStore::create(size_t size) {
  auto block = _baseBlockStore->create(size);
  Key key = block->key();
  return _cachingStore.add(key, std::move(block));
}

unique_ptr<Block> CachingBlockStore::load(const Key &key) {
  return _cachingStore.load(key);
}


void CachingBlockStore::remove(unique_ptr<Block> block) {
  Key key = block->key();
  return _cachingStore.remove(key, dynamic_pointer_move<CachedBlockRef>(block));
}

uint64_t CachingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
