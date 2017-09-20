#include "AsyncBlockStore2.h"
#include <memory>
#include <cpp-utils/assert/assert.h>

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
using std::unique_lock;
using std::mutex;

namespace blockstore {
namespace async {

// TODO BaseStore needs to be threadsafe
// TODO For some functions here it doesn't make sense to run them in a separate thread pool
// TODO forEachBlock will run callbacks on a different thread. is that ok?
// TODO Should we parallelize forEachBlock somehow? Only as multiple fibers on one thread? Or multiple threads? In this class or somewhere else?

AsyncBlockStore2::AsyncBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, size_t numExecutorThreads)
: _baseBlockStore(std::move(baseBlockStore)), _executor(numExecutorThreads) {
}

bool AsyncBlockStore2::tryCreate(const BlockId &blockId, const Data &data) {
  return _executor.execute([this, &blockId, &data] () {
      return _baseBlockStore->tryCreate(blockId, data);
  });
}

bool AsyncBlockStore2::remove(const BlockId &blockId) {
  return _executor.execute([this, &blockId] () {
      return _baseBlockStore->remove(blockId);
  });
}

optional<Data> AsyncBlockStore2::load(const BlockId &blockId) const {
  return _executor.execute([this, &blockId] () {
      return _baseBlockStore->load(blockId);
  });
}

void AsyncBlockStore2::store(const BlockId &blockId, const Data &data) {
  return _executor.execute([this, &blockId, &data] () {
      return _baseBlockStore->store(blockId, data);
  });
}

uint64_t AsyncBlockStore2::numBlocks() const {
  return _executor.execute([this] () {
      return _baseBlockStore->numBlocks();
  });
}

uint64_t AsyncBlockStore2::estimateNumFreeBytes() const {
  return _executor.execute([this] () {
      return _baseBlockStore->estimateNumFreeBytes();
  });
}

uint64_t AsyncBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return _executor.execute([this, blockSize] () {
      return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
  });
}

void AsyncBlockStore2::forEachBlock(std::function<void (const BlockId &)> callback) const {
  return _executor.execute([this, &callback] () {
      return _baseBlockStore->forEachBlock(std::move(callback));
  });
}

}
}
