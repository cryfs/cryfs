#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include "../caching/cache/Cache.h"
#include <unordered_set>

namespace blockstore {
namespace caching {

class CachingBlockStore2 final: public BlockStore2 {
public:
  CachingBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore);

  bool tryCreate(const Key &key, const cpputils::Data &data) override;
  bool remove(const Key &key) override;
  boost::optional<cpputils::Data> load(const Key &key) const override;
  void store(const Key &key, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:
  // TODO Is a cache implementation with onEvict callback instead of destructor simpler?
  class CachedBlock final {
  public:
    CachedBlock(const CachingBlockStore2* blockStore, const Key& key, cpputils::Data data, bool isDirty);
    ~CachedBlock();

    const cpputils::Data& read() const;
    void write(cpputils::Data data);
    bool remove() &&; // only on rvalue because the destructor should be called after calling remove(). It shouldn't be put back into the cache.
  private:
    const CachingBlockStore2* _blockStore;
    Key _key;
    cpputils::Data _data;
    bool _dirty;

    DISALLOW_COPY_AND_ASSIGN(CachedBlock);
  };

  boost::optional<cpputils::unique_ref<CachedBlock>> _loadFromCacheOrBaseStore(const Key &key) const;

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  friend class CachedBlock;

  // TODO Store CachedBlock directly, without unique_ref
  mutable std::mutex _cachedBlocksNotInBaseStoreMutex;
  mutable std::unordered_set<Key> _cachedBlocksNotInBaseStore;
  mutable Cache<Key, cpputils::unique_ref<CachedBlock>, 1000> _cache;

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStore2);
};

}
}

#endif
