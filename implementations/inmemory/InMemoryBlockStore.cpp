#include <messmer/blockstore/implementations/inmemory/InMemoryBlock.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <memory>

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::piecewise_construct;
using std::make_tuple;

namespace blockstore {
namespace inmemory {

InMemoryBlockStore::InMemoryBlockStore()
 : _blocks() {}

unique_ptr<Block> InMemoryBlockStore::create(const Key &key, size_t size) {
  auto insert_result = _blocks.emplace(piecewise_construct, make_tuple(key.ToString()), make_tuple(key, size));

  if (!insert_result.second) {
    return nullptr;
  }

  //Return a pointer to the stored InMemoryBlock
  return make_unique<InMemoryBlock>(insert_result.first->second);
}

unique_ptr<Block> InMemoryBlockStore::load(const Key &key) {
  //Return a pointer to the stored InMemoryBlock
  try {
    return make_unique<InMemoryBlock>(_blocks.at(key.ToString()));
  } catch (const std::out_of_range &e) {
    return nullptr;
  }
}

}
}
