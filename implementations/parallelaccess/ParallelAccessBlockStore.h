#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_

#include <messmer/parallelaccessstore/ParallelAccessStore.h>
#include "BlockRef.h"
#include "../../interface/BlockStore.h"

namespace blockstore {
namespace parallelaccess {

//TODO Check that this blockstore allows parallel destructing of blocks (otherwise we won't encrypt blocks in parallel)
class ParallelAccessBlockStore: public BlockStore {
public:
  ParallelAccessBlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  Key createKey() override;
  std::unique_ptr<Block> tryCreate(const Key &key, Data data) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

private:
  std::unique_ptr<BlockStore> _baseBlockStore;
  parallelaccessstore::ParallelAccessStore<Block, BlockRef, Key> _parallelAccessStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessBlockStore);
};

}
}

#endif
