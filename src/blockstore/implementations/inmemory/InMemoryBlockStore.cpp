#include "InMemoryBlock.h"
#include "InMemoryBlockStore.h"
#include <memory>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/get_total_memory.h>

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
  auto insert_result = _blocks.emplace(piecewise_construct, make_tuple(key), make_tuple(key, std::move(data)));

  if (!insert_result.second) {
    return none;
  }

  //Return a pointer to the stored InMemoryBlock
  return optional<unique_ref<Block>>(make_unique_ref<InMemoryBlock>(insert_result.first->second));
}

optional<unique_ref<Block>> InMemoryBlockStore::load(const Key &key) {
  //Return a pointer to the stored InMemoryBlock
  try {
    return optional<unique_ref<Block>>(make_unique_ref<InMemoryBlock>(_blocks.at(key)));
  } catch (const std::out_of_range &e) {
    return none;
  }
}

unique_ref<Block> InMemoryBlockStore::overwrite(const Key &key, Data data) {
  InMemoryBlock newBlock(key, std::move(data));
  auto insert_result = _blocks.emplace(key, newBlock);

  if (!insert_result.second) {
    // If block already exists, overwrite it.
    insert_result.first->second = newBlock;
  }

  //Return a pointer to the stored InMemoryBlock
  return make_unique_ref<InMemoryBlock>(insert_result.first->second);
}

void InMemoryBlockStore::remove(const Key &key) {
  int numRemoved = _blocks.erase(key);
  ASSERT(1==numRemoved, "Didn't find block to remove");
}

void InMemoryBlockStore::removeIfExists(const Key &key) {
  _blocks.erase(key);
}

uint64_t InMemoryBlockStore::numBlocks() const {
  return _blocks.size();
}

uint64_t InMemoryBlockStore::estimateNumFreeBytes() const {
  return cpputils::system::get_total_memory();
}

uint64_t InMemoryBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return blockSize;
}

void InMemoryBlockStore::forEachBlock(std::function<void (const Key &)> callback) const {
  for (const auto &entry : _blocks) {
    callback(entry.first);
  }
}

bool InMemoryBlockStore::exists(const Key &key) const {
  return _blocks.count(key) != 0;
}

}
}
