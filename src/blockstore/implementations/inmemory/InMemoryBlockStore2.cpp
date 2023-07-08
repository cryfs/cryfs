#include "InMemoryBlockStore2.h"
#include <memory>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/get_total_memory.h>

using std::string;
using std::mutex;
using std::make_pair;
using std::vector;
using cpputils::Data;
using boost::optional;

namespace blockstore {
namespace inmemory {

InMemoryBlockStore2::InMemoryBlockStore2()
 : _blocks() {}

bool InMemoryBlockStore2::tryCreate(const BlockId &blockId, const Data &data) {
  const std::unique_lock<std::mutex> lock(_mutex);
  return _tryCreate(blockId, data);
}

bool InMemoryBlockStore2::_tryCreate(const BlockId &blockId, const Data &data) {
  auto result = _blocks.insert(make_pair(blockId, data.copy()));
  return result.second; // Return if insertion was successful (i.e. blockId didn't exist yet)
}

bool InMemoryBlockStore2::remove(const BlockId &blockId) {
  const std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(blockId);
  if (found == _blocks.end()) {
    // BlockId not found
    return false;
  }

  _blocks.erase(found);
  return true;
}

optional<Data> InMemoryBlockStore2::load(const BlockId &blockId) const {
  const std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(blockId);
  if (found == _blocks.end()) {
    return boost::none;
  }
  return found->second.copy();
}

void InMemoryBlockStore2::store(const BlockId &blockId, const Data &data) {
  const std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(blockId);
  if (found == _blocks.end()) {
    const bool success = _tryCreate(blockId, data);
    if (!success) {
      throw std::runtime_error("Could neither save nor create the block in InMemoryBlockStore::store()");
    }
  } else {
    // TODO Would have better performance: found->second.overwriteWith(data)
    found->second = data.copy();
  }
}

uint64_t InMemoryBlockStore2::numBlocks() const {
  const std::unique_lock<std::mutex> lock(_mutex);
  return _blocks.size();
}

uint64_t InMemoryBlockStore2::estimateNumFreeBytes() const {
  return cpputils::system::get_total_memory();
}

uint64_t InMemoryBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return blockSize;
}

vector<BlockId> InMemoryBlockStore2::_allBlockIds() const {
  const std::unique_lock<std::mutex> lock(_mutex);
  vector<BlockId> result;
  result.reserve(_blocks.size());
  for (const auto &entry : _blocks) {
    result.push_back(entry.first);
  }
  return result;
}

void InMemoryBlockStore2::forEachBlock(std::function<void (const BlockId &)> callback) const {
  auto blockIds = _allBlockIds();
  for (const auto &blockId : blockIds) {
    callback(blockId);
  }
}

}
}
