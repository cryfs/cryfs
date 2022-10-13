#include "BlockRef.h"
#include "ParallelAccessBlockStore.h"
#include "ParallelAccessBlockStoreAdapter.h"
#include <cassert>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/assert/assert.h>

using std::string;
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

BlockId ParallelAccessBlockStore::createBlockId() {
  return _baseBlockStore->createBlockId();
}

optional<unique_ref<Block>> ParallelAccessBlockStore::tryCreate(const BlockId &blockId, Data data) {
  if (_parallelAccessStore.isOpened(blockId)) {
    return none; // block already exists
  }
  auto block = _baseBlockStore->tryCreate(blockId, std::move(data));
  if (block == none) {
	//TODO Test this code branch
	return none;
  }
  return unique_ref<Block>(_parallelAccessStore.add(blockId, std::move(*block)));
}

optional<unique_ref<Block>> ParallelAccessBlockStore::load(const BlockId &blockId) {
  auto block = _parallelAccessStore.load(blockId);
  if (block == none) {
    return none;
  }
  return unique_ref<Block>(std::move(*block));
}

unique_ref<Block> ParallelAccessBlockStore::overwrite(const BlockId &blockId, Data data) {
  auto onExists = [&data] (BlockRef *block) {
      if (block->size() != data.size()) {
        block->resize(data.size());
      }
      block->write(data.data(), 0, data.size());
  };
  auto onAdd = [this, blockId, &data] {
      return _baseBlockStore->overwrite(blockId, data.copy()); // TODO Without copy?
  };
  return _parallelAccessStore.loadOrAdd(blockId, onExists, onAdd); // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
}

void ParallelAccessBlockStore::remove(unique_ref<Block> block) {
  BlockId blockId = block->blockId();
  auto block_ref = dynamic_pointer_move<BlockRef>(block);
  ASSERT(block_ref != none, "Block is not a BlockRef");
  return _parallelAccessStore.remove(blockId, std::move(*block_ref));
}

void ParallelAccessBlockStore::remove(const BlockId &blockId) {
  return _parallelAccessStore.remove(blockId);
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

void ParallelAccessBlockStore::forEachBlock(std::function<void (const BlockId &)> callback) const {
  return _baseBlockStore->forEachBlock(callback);
}

void ParallelAccessBlockStore::flushBlock(Block* block) {
  BlockRef* blockRef = dynamic_cast<BlockRef*>(block);
  ASSERT(blockRef != nullptr, "flushBlock got a block from the wrong block store");
  return _baseBlockStore->flushBlock(&*(blockRef->_baseBlock));
}

}
}
