#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSBLOCKSTORE_H_

#include <parallelaccessstore/ParallelAccessStore.h>
#include "BlockRef.h"
#include "../../interface/BlockStore.h"
#include <cpp-utils/pointer/unique_ref.h>

namespace blockstore {
namespace parallelaccess {

//TODO Check that this blockstore allows parallel destructing of blocks (otherwise we won't encrypt blocks in parallel)
class ParallelAccessBlockStore final: public BlockStore {
public:
  explicit ParallelAccessBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore);

  BlockId createBlockId() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const BlockId &blockId, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const BlockId &blockId) override;
  cpputils::unique_ref<Block> overwrite(const BlockId &blockId, cpputils::Data data) override;
  void remove(const BlockId &blockId) override;
  void remove(cpputils::unique_ref<Block> node) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;
  void flushBlock(Block* block) override;

private:
  cpputils::unique_ref<BlockStore> _baseBlockStore;
  parallelaccessstore::ParallelAccessStore<Block, BlockRef, BlockId> _parallelAccessStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessBlockStore);
};

}
}

#endif
