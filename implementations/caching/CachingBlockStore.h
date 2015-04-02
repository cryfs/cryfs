#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_CACHIN_CACHINGBLOCKSTORE_H_

#include "CachingStore.h"

#include "../../interface/BlockStore.h"
#include "CachedBlockRef.h"

namespace blockstore {
namespace caching {

class CachingBlockStore: public BlockStore, private CachingStore<Block, CachedBlockRef, Key> {
public:
  CachingBlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  std::unique_ptr<Block> create(size_t size) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

protected:
  const Key &getKey(const Block &block) const override;
  std::unique_ptr<Block> loadFromBaseStore(const Key &key) override;
  void removeFromBaseStore(std::unique_ptr<Block> block) override;

private:
  std::unique_ptr<BlockStore> _baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStore);
};

}
}

#endif
