#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_THREADSAFE_THREADSAFEBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_THREADSAFE_THREADSAFEBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/cast.h>
#include <iostream>
#include <cpp-utils/lock/LockPool.h>

namespace blockstore {
namespace threadsafe {

class ThreadsafeBlockStore final: public BlockStore {
public:
  ThreadsafeBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore);

  BlockId createBlockId() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const BlockId &blockId, cpputils::Data data) override;
  cpputils::unique_ref<Block> overwrite(const BlockId &blockId, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const BlockId &blockId) override;
  void remove(const BlockId &blockId) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

private:
  cpputils::unique_ref<BlockStore> _baseBlockStore;
  cpputils::LockPool<BlockId> _checkedOutBlocks;
  mutable std::mutex _structureMutex; // protects structure, i.e. which block ids exactly exist

  DISALLOW_COPY_AND_ASSIGN(ThreadsafeBlockStore);
};

}
}

#endif
