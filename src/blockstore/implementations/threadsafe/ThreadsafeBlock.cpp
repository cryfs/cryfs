#include "ThreadsafeBlock.h"

using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::MutexPoolLock;
using std::unique_lock;
using std::mutex;

namespace blockstore {
namespace threadsafe {

ThreadsafeBlock::ThreadsafeBlock(cpputils::unique_ref<Block> baseBlock, MutexPoolLock<BlockId> poolLock)
    :Block(baseBlock->blockId()),
     _poolLock(std::move(poolLock)),
     _baseBlock(std::move(baseBlock)),
     _mutex() {
}

const void *ThreadsafeBlock::data() const {
  // TODO Readers lock as long as they have the data pointer
  unique_lock<mutex> lock(_mutex);
  return _baseBlock->data();
}

void ThreadsafeBlock::write(const void *source, uint64_t offset, uint64_t count) {
  unique_lock<mutex> lock(_mutex);
  return _baseBlock->write(source, offset, count);
}

void ThreadsafeBlock::flush() {
  unique_lock<mutex> lock(_mutex);
  return _baseBlock->flush();
}

size_t ThreadsafeBlock::size() const {
  unique_lock<mutex> lock(_mutex);
  return _baseBlock->size();
}

void ThreadsafeBlock::resize(size_t newSize) {
  unique_lock<mutex> lock(_mutex);
  return _baseBlock->resize(newSize);
}

}
}
