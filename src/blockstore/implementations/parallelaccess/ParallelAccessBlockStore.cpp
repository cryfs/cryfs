#include "BlockRef.h"
#include "ParallelAccessBlockStore.h"
#include "ParallelAccessBlockStoreAdapter.h"
#include <cassert>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/assert/assert.h>

using std::string;
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
  ASSERT(!_parallelAccessStore.isOpened(key), ("Key "+key.ToString()+"already exists").c_str());
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
  ASSERT(block_ref != none, "Block is not a BlockRef");
  return _parallelAccessStore.remove(key, std::move(*block_ref));
}

uint64_t ParallelAccessBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

uint64_t ParallelAccessBlockStore::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

uint64_t ParallelAccessBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

}
}
