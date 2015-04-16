#include "BlockRef.h"
#include "ParallelAccessBlockStore.h"
#include "ParallelAccessBlockStoreAdapter.h"
#include <cassert>
#include <messmer/cpp-utils/pointer.h>


using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::promise;
using cpputils::dynamic_pointer_move;

namespace blockstore {
namespace parallelaccess {

ParallelAccessBlockStore::ParallelAccessBlockStore(unique_ptr<BlockStore> baseBlockStore)
 : _baseBlockStore(std::move(baseBlockStore)), _cachingStore(make_unique<ParallelAccessBlockStoreAdapter>(_baseBlockStore.get())) {
}

unique_ptr<Block> ParallelAccessBlockStore::create(size_t size) {
  auto block = _baseBlockStore->create(size);
  Key key = block->key();
  return _cachingStore.add(key, std::move(block));
}

unique_ptr<Block> ParallelAccessBlockStore::load(const Key &key) {
  return _cachingStore.load(key);
}


void ParallelAccessBlockStore::remove(unique_ptr<Block> block) {
  Key key = block->key();
  return _cachingStore.remove(key, dynamic_pointer_move<BlockRef>(block));
}

uint64_t ParallelAccessBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
