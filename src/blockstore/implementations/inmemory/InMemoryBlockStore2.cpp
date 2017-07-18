#include "InMemoryBlockStore2.h"
#include <memory>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/get_total_memory.h>

using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;
using std::piecewise_construct;
using std::make_tuple;
using std::make_pair;
using std::vector;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;
using boost::future;
using boost::make_ready_future;

namespace blockstore {
namespace inmemory {

InMemoryBlockStore2::InMemoryBlockStore2()
 : _blocks() {}

future<bool> InMemoryBlockStore2::tryCreate(const Key &key, const Data &data) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto result = _blocks.insert(make_pair(key, data.copy()));
  return make_ready_future(result.second); // Return if insertion was successful (i.e. key didn't exist yet)
}

future<bool> InMemoryBlockStore2::remove(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(key);
  if (found == _blocks.end()) {
    // Key not found
    return make_ready_future(false);
  }

  _blocks.erase(found);
  return make_ready_future(true);
}

future<optional<Data>> InMemoryBlockStore2::load(const Key &key) const {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(key);
  if (found == _blocks.end()) {
    return make_ready_future(optional<Data>(none));
  }
  return make_ready_future(optional<Data>(found->second.copy()));
}

future<void> InMemoryBlockStore2::store(const Key &key, const Data &data) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _blocks.find(key);
  if (found == _blocks.end()) {
    return tryCreate(key, data).then([] (future<bool> success) {
      if (!success.get()) {
        throw std::runtime_error("Could neither save nor create the block in InMemoryBlockStore::store()");
      }
    });
  }
  // TODO Would have better performance: found->second.overwriteWith(data)
  found->second = data.copy();
  return make_ready_future();
}

uint64_t InMemoryBlockStore2::numBlocks() const {
  std::unique_lock<std::mutex> lock(_mutex);
  return _blocks.size();
}

uint64_t InMemoryBlockStore2::estimateNumFreeBytes() const {
  return cpputils::system::get_total_memory();
}

uint64_t InMemoryBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return blockSize;
}

vector<Key> InMemoryBlockStore2::_allBlockKeys() const {
  std::unique_lock<std::mutex> lock(_mutex);
  vector<Key> result;
  result.reserve(_blocks.size());
  for (const auto &entry : _blocks) {
    result.push_back(entry.first);
  }
  return result;
}

void InMemoryBlockStore2::forEachBlock(std::function<void (const Key &)> callback) const {
  auto keys = _allBlockKeys();
  for (const auto &key : keys) {
    callback(key);
  }
}

}
}
