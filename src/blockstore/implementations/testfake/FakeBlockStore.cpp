#include "FakeBlock.h"
#include "FakeBlockStore.h"
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/get_total_memory.h>

using std::make_shared;
using std::string;
using std::mutex;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

namespace blockstore {
namespace testfake {

FakeBlockStore::FakeBlockStore()
 : _blocks(), _used_dataregions_for_blocks(), _mutex() {}

BlockId FakeBlockStore::createBlockId() {
  return BlockId::Random();
}

optional<unique_ref<Block>> FakeBlockStore::tryCreate(const BlockId &blockId, Data data) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto insert_result = _blocks.emplace(blockId, std::move(data));

  if (!insert_result.second) {
    return none;
  }

  //Return a copy of the stored data
  return _load(blockId);
}

unique_ref<Block> FakeBlockStore::overwrite(const BlockId &blockId, Data data) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto insert_result = _blocks.emplace(blockId, data.copy());

  if (!insert_result.second) {
    // If block already exists, overwrite it.
    insert_result.first->second = std::move(data);
  }

  //Return a pointer to the stored FakeBlock
  auto loaded = _load(blockId);
  ASSERT(loaded != none, "Block was just created or written. Should exist.");
  return std::move(*loaded);
}

optional<unique_ref<Block>> FakeBlockStore::load(const BlockId &blockId) {
  std::unique_lock<std::mutex> lock(_mutex);
  return _load(blockId);
}

optional<unique_ref<Block>> FakeBlockStore::_load(const BlockId &blockId) {
  //Return a copy of the stored data
  try {
    return makeFakeBlockFromData(blockId, _blocks.at(blockId), false);
  } catch (const std::out_of_range &e) {
    return none;
  }
}

void FakeBlockStore::remove(const BlockId &blockId) {
  std::unique_lock<std::mutex> lock(_mutex);
  int numRemoved = _blocks.erase(blockId);
  ASSERT(numRemoved == 1, "Block not found");
}

unique_ref<Block> FakeBlockStore::makeFakeBlockFromData(const BlockId &blockId, const Data &data, bool dirty) {
  auto newdata = make_shared<Data>(data.copy());
  _used_dataregions_for_blocks.push_back(newdata);
  return make_unique_ref<FakeBlock>(this, blockId, newdata, dirty);
}

void FakeBlockStore::updateData(const BlockId &blockId, const Data &data) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(blockId);
  if (found == _blocks.end()) {
    auto insertResult = _blocks.emplace(blockId, data.copy());
    ASSERT(true == insertResult.second, "Inserting didn't work");
    found = insertResult.first;
  }
  Data &stored_data = found->second;
  stored_data = data.copy();
}

uint64_t FakeBlockStore::numBlocks() const {
  std::unique_lock<std::mutex> lock(_mutex);
  return _blocks.size();
}

uint64_t FakeBlockStore::estimateNumFreeBytes() const {
  return cpputils::system::get_total_memory();
}

uint64_t FakeBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return blockSize;
}

void FakeBlockStore::forEachBlock(std::function<void (const BlockId &)> callback) const {
  for (const auto &entry : _blocks) {
    callback(entry.first);
  }
}

void FakeBlockStore::flushBlock(Block* block) {
  FakeBlock* fakeBlock = dynamic_cast<FakeBlock*>(block);
  ASSERT(fakeBlock != nullptr, "flushBlock got a block from the wrong block store");
  fakeBlock->flush();
}

}
}
