#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTOREADAPTER_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTOREADAPTER_H_

#include <messmer/cpp-utils/macros.h>
#include <messmer/cachingstore/CachingStore.h>
#include "../../interface/BlockStore.h"

namespace blockstore {
namespace caching {

class CachingBlockStoreAdapter: public cachingstore::CachingBaseStore<Block, Key> {
public:
  CachingBlockStoreAdapter(BlockStore *baseBlockStore)
    :_baseBlockStore(std::move(baseBlockStore)) {
  }

  std::unique_ptr<Block> loadFromBaseStore(const Key &key) override {
	return _baseBlockStore->load(key);
  }

  void removeFromBaseStore(std::unique_ptr<Block> block) override {
	return _baseBlockStore->remove(std::move(block));
  }

private:
  BlockStore *_baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStoreAdapter);
};

}
}

#endif
