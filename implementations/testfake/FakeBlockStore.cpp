#include "FakeBlock.h"
#include "FakeBlockStore.h"
#include <messmer/cpp-utils/assert/assert.h>

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
 : _blocks(), _used_dataregions_for_blocks() {}

optional<unique_ref<Block>> FakeBlockStore::tryCreate(const Key &key, Data data) {
  auto insert_result = _blocks.emplace(key.ToString(), std::move(data));

  if (!insert_result.second) {
    return none;
  }

  //Return a copy of the stored data
  return load(key);
}

optional<unique_ref<Block>> FakeBlockStore::load(const Key &key) {
  //Return a copy of the stored data
  string key_string = key.ToString();
  try {
    return makeFakeBlockFromData(key, _blocks.at(key_string), false);
  } catch (const std::out_of_range &e) {
    return none;
  }
}

void FakeBlockStore::remove(unique_ref<Block> block) {
  Key key = block->key();
  cpputils::destruct(std::move(block));
  int numRemoved = _blocks.erase(key.ToString());
  ASSERT(numRemoved == 1, "Block not found");
}

unique_ref<Block> FakeBlockStore::makeFakeBlockFromData(const Key &key, const Data &data, bool dirty) {
  auto newdata = make_shared<Data>(data.copy());
  _used_dataregions_for_blocks.push_back(newdata);
  return make_unique_ref<FakeBlock>(this, key, newdata, dirty);
}

void FakeBlockStore::updateData(const Key &key, const Data &data) {
  auto found = _blocks.find(key.ToString());
  if (found == _blocks.end()) {
    auto insertResult = _blocks.emplace(key.ToString(), data.copy());
    ASSERT(true == insertResult.second, "Inserting didn't work");
    found = insertResult.first;
  }
  Data &stored_data = found->second;
  ASSERT(data.size() == stored_data.size(), "Wrong data size in block");
  std::memcpy(stored_data.data(), data.data(), data.size());
}

uint64_t FakeBlockStore::numBlocks() const {
  return _blocks.size();
}

}
}
