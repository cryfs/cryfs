#include "FakeBlockStore.h"
#include "FakeBlock.h"

using std::unique_ptr;
using std::make_unique;
using std::make_shared;
using std::string;
using std::mutex;
using std::lock_guard;

namespace blockstore {
namespace testfake {

FakeBlockStore::FakeBlockStore()
 : _blocks(), _used_dataregions_for_blocks() {}

unique_ptr<Block> FakeBlockStore::create(const Key &key, size_t size) {
  string key_string = key.AsString();
  auto insert_result = _blocks.emplace(key_string, size);
  insert_result.first->second.FillWithZeroes();

  if (!insert_result.second) {
    return nullptr;
  }

  //Return a copy of the stored data
  _used_dataregions_for_blocks.push_back(make_shared<Data>(size));
  std::memcpy(_used_dataregions_for_blocks.back()->data(), insert_result.first->second.data(), size);
  return make_unique<FakeBlock>(this, key_string, _used_dataregions_for_blocks.back());
}

unique_ptr<Block> FakeBlockStore::load(const Key &key) {
  //Return a copy of the stored data
  string key_string = key.AsString();
  try {
    const Data &data = _blocks.at(key_string);
    _used_dataregions_for_blocks.push_back(make_shared<Data>(data.size()));
    std::memcpy(_used_dataregions_for_blocks.back()->data(), data.data(), data.size());
    return make_unique<FakeBlock>(this, key_string, _used_dataregions_for_blocks.back());
  } catch (const std::out_of_range &e) {
    return nullptr;
  }
}

void FakeBlockStore::updateData(const std::string &key, const Data &data) {
  Data &stored_data = _blocks.at(key);
  assert(data.size() == stored_data.size());
  std::memcpy(stored_data.data(), data.data(), data.size());
}

}
}
