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
 : _baseBlockStore(std::move(baseBlockStore)), _parallelAccessStore(make_unique<ParallelAccessBlockStoreAdapter>(_baseBlockStore.get())) {
}

Key ParallelAccessBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

unique_ptr<Block> ParallelAccessBlockStore::tryCreate(const Key &key, Data data) {
  auto block = _baseBlockStore->tryCreate(key, std::move(data));
  if (block.get() == nullptr) {
	//TODO Test this code branch
	return nullptr;
  }
  return _parallelAccessStore.add(key, std::move(block));
}

unique_ptr<Block> ParallelAccessBlockStore::load(const Key &key) {
  return _parallelAccessStore.load(key);
}


void ParallelAccessBlockStore::remove(unique_ptr<Block> block) {
  Key key = block->key();
  return _parallelAccessStore.remove(key, dynamic_pointer_move<BlockRef>(block));
}

uint64_t ParallelAccessBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
