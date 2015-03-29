#include "OpenBlockList.h"

#include "OpenBlock.h"
#include <cassert>

using std::unique_ptr;
using std::make_unique;
using std::function;
using std::mutex;
using std::lock_guard;
using std::unique_lock;
using std::promise;
using std::future;

namespace blockstore {
namespace synchronized {

OpenBlockList::OpenBlockList() {
}

OpenBlockList::~OpenBlockList() {
}

unique_ptr<Block> OpenBlockList::insert(unique_ptr<Block> block) {
  lock_guard<mutex> lock(_mutex);
  auto insertResult = _openBlocks.insert(block->key());
  assert(insertResult.second == true);
  return make_unique<OpenBlock>(std::move(block), this);
}

unique_ptr<Block> OpenBlockList::acquire(const Key &key, function<unique_ptr<Block> ()> loader) {
  unique_lock<mutex> lock(_mutex);
  auto insertResult = _openBlocks.insert(key);
  auto blockWasNotOpenYet = insertResult.second;
  if (blockWasNotOpenYet) {
	lock.unlock();
	auto block = loader();
	if (block.get() == nullptr) {
	  return nullptr;
	}
	return make_unique<OpenBlock>(std::move(block), this);
  } else {
	auto blockFuture = _addPromiseForBlock(key);
	lock.unlock();
	return blockFuture.get();
  }
}

future<unique_ptr<Block>> OpenBlockList::_addPromiseForBlock(const Key &key) {
  auto insertResult = _wantedBlocks.emplace(key, promise<unique_ptr<Block>>());
  assert(insertResult.second == true);
  return insertResult.first->second.get_future();
}

void OpenBlockList::release(unique_ptr<Block> block) {
  lock_guard<mutex> lock(_mutex);
  auto foundWantedBlock = _wantedBlocks.find(block->key());
  if (foundWantedBlock != _wantedBlocks.end()) {
	foundWantedBlock->second.set_value(std::move(block));
  } else {
	_openBlocks.erase(block->key());
    auto foundBlockToClose = _blocksToClose.find(block->key());
    if (foundBlockToClose != _blocksToClose.end()) {
      foundBlockToClose->second.set_value(std::move(block));
    }
  }
}

void OpenBlockList::close(unique_ptr<Block> block, function<void (unique_ptr<Block>)> onClose) {
  unique_lock<mutex> lock(_mutex);
  auto insertResult = _blocksToClose.emplace(block->key(), promise<unique_ptr<Block>>());
  assert(insertResult.second == true);
  block.reset();
  lock.unlock();
  auto closedBlock = insertResult.first->second.get_future().get();
  onClose(std::move(closedBlock));
}

}
}
