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
 : _baseBlockStore(std::move(baseBlockStore)),
   _openBlocks() {
}

unique_ptr<Block> CachingBlockStore::create(size_t size) {
  auto block = _baseBlockStore->create(size);
  lock_guard<mutex> lock(_mutex);
  return _addOpenBlock(std::move(block));
}

unique_ptr<Block> CachingBlockStore::_addOpenBlock(unique_ptr<Block> block) {
  auto insertResult = _openBlocks.emplace(block->key(), std::move(block));
  assert(true == insertResult.second);
  return make_unique<CachedBlockRef>(insertResult.first->second.getReference(), this);
}

unique_ptr<Block> CachingBlockStore::load(const Key &key) {
  lock_guard<mutex> lock(_mutex);
  auto found = _openBlocks.find(key);
  if (found == _openBlocks.end()) {
	auto block = _baseBlockStore->load(key);
	if (block.get() == nullptr) {
	  return nullptr;
	}
	return _addOpenBlock(std::move(block));
  } else {
	return make_unique<CachedBlockRef>(found->second.getReference(), this);
  }
}

void CachingBlockStore::release(const Block *block) {
  lock_guard<mutex> lock(_mutex);
  Key key = block->key();
  auto found = _openBlocks.find(key);
  assert (found != _openBlocks.end());
  found->second.releaseReference();
  if (found->second.refCount == 0) {
	auto foundToRemove = _blocksToRemove.find(key);
	if (foundToRemove != _blocksToRemove.end()) {
	  foundToRemove->second.set_value(std::move(found->second.block));
	}
	_openBlocks.erase(found);
  }
}

void CachingBlockStore::remove(unique_ptr<Block> block) {
  auto insertResult = _blocksToRemove.emplace(block->key(), promise<unique_ptr<Block>>());
  assert(true == insertResult.second);
  block.reset();

  //Wait for last block user to release it
  auto blockToRemove = insertResult.first->second.get_future().get();

  _baseBlockStore->remove(std::move(blockToRemove));
}

uint64_t CachingBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
