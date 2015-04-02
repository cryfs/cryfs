#include <messmer/blockstore/implementations/caching/CachedBlockRef.h>
#include <messmer/blockstore/implementations/caching/CachingBlockStore.h>
#include <cassert>

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::promise;

namespace blockstore {
namespace caching {

CachingBlockStore::CachingBlockStore(unique_ptr<BlockStore> baseBlockStore)
 : _baseBlockStore(std::move(baseBlockStore)) {
}

unique_ptr<Block> CachingBlockStore::create(size_t size) {
  auto block = _baseBlockStore->create(size);
  Key key = block->key();
  return CachingStore::add(key, std::move(block));
}

unique_ptr<Block> CachingBlockStore::load(const Key &key) {
  return CachingStore::load(key);
}


void CachingBlockStore::remove(unique_ptr<Block> block) {
  Key key = block->key();
  return CachingStore::remove(key, std::move(block));
}

unique_ptr<Block> CachingBlockStore::loadFromBaseStore(const Key &key) {
  return _baseBlockStore->load(key);
}

void CachingBlockStore::removeFromBaseStore(unique_ptr<Block> block) {
  return _baseBlockStore->remove(std::move(block));
}

uint64_t CachingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
