#include "InMemoryBlock.h"
#include "InMemoryBlockStore.h"
#include <memory>
#include <cpp-utils/assert/assert.h>

using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::piecewise_construct;
using std::make_tuple;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

namespace blockstore {
namespace inmemory {

InMemoryBlockStore::InMemoryBlockStore()
 : _blocks() {}

optional<unique_ref<Block>> InMemoryBlockStore::tryCreate(const Key &key, Data data) {
  auto insert_result = _blocks.emplace(piecewise_construct, make_tuple(key.ToString()), make_tuple(key, std::move(data)));

  if (!insert_result.second) {
    return none;
  }

  //Return a pointer to the stored InMemoryBlock
  return optional<unique_ref<Block>>(make_unique_ref<InMemoryBlock>(insert_result.first->second));
}

optional<unique_ref<Block>> InMemoryBlockStore::load(const Key &key) {
  //Return a pointer to the stored InMemoryBlock
  try {
    return optional<unique_ref<Block>>(make_unique_ref<InMemoryBlock>(_blocks.at(key.ToString())));
  } catch (const std::out_of_range &e) {
    return none;
  }
}

void InMemoryBlockStore::remove(unique_ref<Block> block) {
  Key key = block->key();
  cpputils::destruct(std::move(block));
  int numRemoved = _blocks.erase(key.ToString());
  ASSERT(1==numRemoved, "Didn't find block to remove");
}

uint64_t InMemoryBlockStore::numBlocks() const {
  return _blocks.size();
}

}
}
