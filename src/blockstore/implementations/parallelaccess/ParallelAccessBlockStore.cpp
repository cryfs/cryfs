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
using cpputils::Data;
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

optional<unique_ref<Block>> ParallelAccessBlockStore::tryCreate(const Key &key, Data data) {
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

unique_ref<Block> ParallelAccessBlockStore::overwrite(const Key &key, Data data) {
  auto onExists = [&data] (BlockRef *block) {
      if (block->size() != data.size()) {
        block->resize(data.size());
      }
      block->write(data.data(), 0, data.size());
  };
  auto onAdd = [this, key, &data] {
      return _baseBlockStore->overwrite(key, data.copy()); // TODO Without copy?
  };
  return _parallelAccessStore.loadOrAdd(key, onExists, onAdd);
}

unique_ref<Block> ParallelAccessBlockStore::loadOrCreate(const Key &key, size_t size) {
  auto onExists = [size] (BlockRef *block) {
      if (block->size() != size) {
        block->resize(size);
      }
  };
  auto onAdd = [this, key, size] {
      return _baseBlockStore->loadOrCreate(key, size);
  };
  return _parallelAccessStore.loadOrAdd(key, onExists, onAdd);
}

void ParallelAccessBlockStore::remove(unique_ref<Block> block) {
  Key key = block->key();
  auto block_ref = dynamic_pointer_move<BlockRef>(block);
  ASSERT(block_ref != none, "Block is not a BlockRef");
  return _parallelAccessStore.remove(key, std::move(*block_ref));
}

void ParallelAccessBlockStore::remove(const Key &key) {
  return _parallelAccessStore.remove(key);
}

void ParallelAccessBlockStore::removeIfExists(const Key &key) {
  //TODO More efficient implementation without calling exists()
  //TODO Fix race condition when block is deleted between exists() and remove().
  if (exists(key)) {
    remove(key);
  }
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

void ParallelAccessBlockStore::forEachBlock(std::function<void (const Key &)> callback) const {
  return _baseBlockStore->forEachBlock(callback);
}

bool ParallelAccessBlockStore::exists(const Key &key) const {
  return _baseBlockStore->exists(key);
}

}
}
