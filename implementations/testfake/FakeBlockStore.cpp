#include <messmer/blockstore/implementations/testfake/FakeBlock.h>
#include <messmer/blockstore/implementations/testfake/FakeBlockStore.h>

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
  auto insert_result = _blocks.emplace(key.ToString(), size);
  insert_result.first->second.FillWithZeroes();

  if (!insert_result.second) {
    return nullptr;
  }

  //Return a copy of the stored data
  return load(key);
}

unique_ptr<Block> FakeBlockStore::load(const Key &key) {
  //Return a copy of the stored data
  string key_string = key.ToString();
  try {
    return makeFakeBlockFromData(key, _blocks.at(key_string));
  } catch (const std::out_of_range &e) {
    return nullptr;
  }
}

void FakeBlockStore::remove(unique_ptr<Block> block) {
  Key key = block->key();
  block.reset();
  int numRemoved = _blocks.erase(key.ToString());
  assert(numRemoved == 1);
}

unique_ptr<Block> FakeBlockStore::makeFakeBlockFromData(const Key &key, const Data &data) {
  auto newdata = make_shared<Data>(data.copy());
  _used_dataregions_for_blocks.push_back(newdata);
  return make_unique<FakeBlock>(this, key, newdata);
}

void FakeBlockStore::updateData(const Key &key, const Data &data) {
  Data &stored_data = _blocks.at(key.ToString());
  assert(data.size() == stored_data.size());
  std::memcpy(stored_data.data(), data.data(), data.size());
}

uint64_t FakeBlockStore::numBlocks() const {
  return _blocks.size();
}

}
}
