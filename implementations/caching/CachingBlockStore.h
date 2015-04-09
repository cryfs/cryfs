#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_

#include <messmer/cachingstore/CachingStore.h>

#include "../../interface/BlockStore.h"
#include "CachedBlockRef.h"

namespace blockstore {
namespace caching {

class CachingBlockStore: public BlockStore {
public:
  CachingBlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  std::unique_ptr<Block> create(size_t size) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

private:
  std::unique_ptr<BlockStore> _baseBlockStore;
  cachingstore::CachingStore<Block, CachedBlockRef, Key> _cachingStore;

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStore);
};

}
}

#endif
