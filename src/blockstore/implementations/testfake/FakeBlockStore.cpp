#include "FakeBlock.h"
#include "FakeBlockStore.h"
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/get_total_memory.h>

using std::make_shared;
using std::string;
using std::mutex;
using std::lock_guard;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

namespace blockstore {
namespace testfake {

FakeBlockStore::FakeBlockStore()
 : _blocks(), _used_dataregions_for_blocks(), _mutex() {}

optional<unique_ref<Block>> FakeBlockStore::tryCreate(const Key &key, Data data) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto insert_result = _blocks.emplace(key, std::move(data));

  if (!insert_result.second) {
    return none;
  }

  //Return a copy of the stored data
  return _load(key);
}

optional<unique_ref<Block>> FakeBlockStore::load(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  return _load(key);
}

optional<unique_ref<Block>> FakeBlockStore::_load(const Key &key) {
  //Return a copy of the stored data
  try {
    return makeFakeBlockFromData(key, _blocks.at(key), false);
  } catch (const std::out_of_range &e) {
    return none;
  }
}

void FakeBlockStore::remove(unique_ref<Block> block) {
  Key key = block->key();
  cpputils::destruct(std::move(block));
  std::unique_lock<std::mutex> lock(_mutex);
  int numRemoved = _blocks.erase(key);
  ASSERT(numRemoved == 1, "Block not found");
}

unique_ref<Block> FakeBlockStore::makeFakeBlockFromData(const Key &key, const Data &data, bool dirty) {
  auto newdata = make_shared<Data>(data.copy());
  _used_dataregions_for_blocks.push_back(newdata);
  return make_unique_ref<FakeBlock>(this, key, newdata, dirty);
}

void FakeBlockStore::updateData(const Key &key, const Data &data) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(key);
  if (found == _blocks.end()) {
    auto insertResult = _blocks.emplace(key, data.copy());
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

}
}
