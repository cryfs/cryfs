#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHINGBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_CACHING2_CACHINGBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include "Cache.h"

namespace blockstore {
namespace caching2 {

//TODO Rename this to CachingBlockStore and the other one to something else
//TODO Check that this blockstore allows parallel destructing of blocks (otherwise we won't encrypt blocks in parallel)
class Caching2BlockStore: public BlockStore {
public:
  Caching2BlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  std::unique_ptr<Block> create(size_t size) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

  void release(std::unique_ptr<Block> block);

private:
  std::unique_ptr<BlockStore> _baseBlockStore;
  Cache _cache;

  DISALLOW_COPY_AND_ASSIGN(Caching2BlockStore);
};

}
}

#endif
