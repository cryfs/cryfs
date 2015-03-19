#include "SynchronizedBlockStore.h"

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::mutex;
using std::lock_guard;

namespace bf = boost::filesystem;

namespace blockstore {
namespace synchronized {

SynchronizedBlockStore::SynchronizedBlockStore(unique_ptr<BlockStore> baseBlockStore)
 : _baseBlockStore(std::move(baseBlockStore)), _mutex() {}

unique_ptr<Block> SynchronizedBlockStore::create(size_t size) {
  //TODO Does this need to be locked?
  lock_guard<mutex> lock(_mutex);
  return _baseBlockStore->create(size);
}

unique_ptr<Block> SynchronizedBlockStore::load(const Key &key) {
  //TODO Only load each block once and lock until old block not used anymore
  lock_guard<mutex> lock(_mutex);
  return _baseBlockStore->load(key);
}

void SynchronizedBlockStore::remove(unique_ptr<Block> block) {
  lock_guard<mutex> lock(_mutex);
  return _baseBlockStore->remove(std::move(block));
}

uint64_t SynchronizedBlockStore::numBlocks() const {
  //TODO Does this need to be locked?
  lock_guard<mutex> lock(_mutex);
  return _baseBlockStore->numBlocks();
}

}
}
