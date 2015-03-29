#include "SynchronizedBlockStore.h"

using std::unique_ptr;
using std::make_unique;
using std::string;

namespace blockstore {
namespace synchronized {

SynchronizedBlockStore::SynchronizedBlockStore(unique_ptr<BlockStore> baseBlockStore)
 : _baseBlockStore(std::move(baseBlockStore)),
   _openBlockList() {
}

unique_ptr<Block> SynchronizedBlockStore::create(size_t size) {
  return _openBlockList.insert(_baseBlockStore->create(size));
}

unique_ptr<Block> SynchronizedBlockStore::load(const Key &key) {
  return _openBlockList.acquire(key, [this, key] {
    return _baseBlockStore->load(key);
  });
}

void SynchronizedBlockStore::remove(unique_ptr<Block> block) {
  _openBlockList.close(std::move(block), [this] (unique_ptr<Block> block) {
    _baseBlockStore->remove(std::move(block));
  });
}

uint64_t SynchronizedBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
