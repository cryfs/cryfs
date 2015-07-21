#include "BlockRef.h"
#include "ParallelAccessBlockStore.h"
#include "ParallelAccessBlockStoreAdapter.h"
#include <cassert>
#include <messmer/cpp-utils/pointer/cast.h>

using std::string;
using std::mutex;
using std::lock_guard;
using std::promise;
using cpputils::dynamic_pointer_move;
using cpputils::make_unique_ref;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

namespace blockstore {
namespace parallelaccess {

ParallelAccessBlockStore::ParallelAccessBlockStore(unique_ref<BlockStore> baseBlockStore)
 : _baseBlockStore(std::move(baseBlockStore)), _parallelAccessStore(make_unique_ref<ParallelAccessBlockStoreAdapter>(_baseBlockStore.get())) {
}

Key ParallelAccessBlockStore::createKey() {
  return _baseBlockStore->createKey();
}

optional<unique_ref<Block>> ParallelAccessBlockStore::tryCreate(const Key &key, cpputils::Data data) {
  auto block = _baseBlockStore->tryCreate(key, std::move(data));
  if (block == none) {
	//TODO Test this code branch
	return none;
  }
  return unique_ref<Block>(_parallelAccessStore.add(key, std::move(*block)));
}

optional<unique_ref<Block>> ParallelAccessBlockStore::load(const Key &key) {
  auto block = _parallelAccessStore.load(key);
  if (block == none) {
    return none;
  }
  return unique_ref<Block>(std::move(*block));
}


void ParallelAccessBlockStore::remove(unique_ref<Block> block) {
  Key key = block->key();
  auto block_ref = dynamic_pointer_move<BlockRef>(block);
  assert(block_ref != none);
  return _parallelAccessStore.remove(key, std::move(*block_ref));
}

uint64_t ParallelAccessBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
