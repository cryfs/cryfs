#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_

#include "cache/Cache.h"
#include "../../interface/BlockStore.h"

namespace blockstore {
namespace caching {

//TODO Check that this blockstore allows parallel destructing of blocks (otherwise we won't encrypt blocks in parallel)
class CachingBlockStore: public BlockStore {
public:
  explicit CachingBlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  Key createKey() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

  void release(std::unique_ptr<Block> block);

  boost::optional<cpputils::unique_ref<Block>> tryCreateInBaseStore(const Key &key, cpputils::Data data);
  void removeFromBaseStore(std::unique_ptr<Block> block);

private:
  std::unique_ptr<BlockStore> _baseBlockStore;
  Cache<Key, std::unique_ptr<Block>> _cache;
  uint32_t _numNewBlocks;

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStore);
};

}
}

#endif
