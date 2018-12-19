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

  bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
  bool remove(const BlockId &blockId) override;
  boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
  void store(const BlockId &blockId, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

  void flush();

private:
  // TODO Is a cache implementation with onEvict callback instead of destructor simpler?
  class CachedBlock final {
  public:
    CachedBlock(const CachingBlockStore2* blockStore, const BlockId &blockId, cpputils::Data data, bool isDirty);
    ~CachedBlock();

    const cpputils::Data& read() const;
    void write(cpputils::Data data);
    void markNotDirty() &&; // only on rvalue because the destructor should be called after calling markNotDirty(). It shouldn't be put back into the cache.
  private:
    const CachingBlockStore2* _blockStore;
    BlockId _blockId;
    cpputils::Data _data;
    bool _dirty;

    DISALLOW_COPY_AND_ASSIGN(CachedBlock);
  };

  boost::optional<cpputils::unique_ref<CachedBlock>> _loadFromCacheOrBaseStore(const BlockId &blockId) const;

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  friend class CachedBlock;

  // TODO Store CachedBlock directly, without unique_ref
  mutable std::mutex _cachedBlocksNotInBaseStoreMutex;
  mutable std::unordered_set<BlockId> _cachedBlocksNotInBaseStore;
  mutable Cache<BlockId, cpputils::unique_ref<CachedBlock>, 1000> _cache;

public:
  static constexpr double MAX_LIFETIME_SEC = decltype(_cache)::MAX_LIFETIME_SEC;

private:

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStore2);
};

}
}

#endif
