#include "FakeBlock.h"
#include "FakeBlockStore.h"

using std::unique_ptr;
using std::make_unique;
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
  cpputils::to_unique_ptr(std::move(block)).reset(); // Call destructor
  int numRemoved = _blocks.erase(key.ToString());
  assert(numRemoved == 1);
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
    assert(true == insertResult.second);
    found = insertResult.first;
  }
  Data &stored_data = found->second;
  assert(data.size() == stored_data.size());
  std::memcpy(stored_data.data(), data.data(), data.size());
}

uint64_t FakeBlockStore::numBlocks() const {
  return _blocks.size();
}

}
}
