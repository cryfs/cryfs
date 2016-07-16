#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_

#include "cache/Cache.h"
#include "../../interface/BlockStore.h"
#include <unordered_set>

namespace blockstore {
namespace caching {

class NewBlock;

//TODO Check that this blockstore allows parallel destructing of blocks (otherwise we won't encrypt blocks in parallel)
class CachingBlockStore final: public BlockStore {
public:
  explicit CachingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore);

  Key createKey() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  cpputils::unique_ref<Block> overwrite(const Key &key, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
  void remove(const Key &key) override;
  void remove(cpputils::unique_ref<Block> block) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

  void release(cpputils::unique_ref<Block> block);

  boost::optional<cpputils::unique_ref<Block>> tryCreateInBaseStore(const Key &key, cpputils::Data data);
  void removeFromBaseStore(cpputils::unique_ref<Block> block);
  void registerNewBlock(NewBlock *newBlock);
  void unregisterNewBlock(NewBlock *newBlock);

  void flush();

private:
  cpputils::unique_ref<BlockStore> _baseBlockStore;
  std::unordered_set<NewBlock*> _newBlocks; // List of all new blocks that aren't in the base store yet.
  Cache<Key, cpputils::unique_ref<Block>, 1000> _cache;
  std::mutex _newBlocksMutex;

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStore);
};

}
}

#endif
