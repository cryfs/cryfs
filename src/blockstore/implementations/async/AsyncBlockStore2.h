#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ASYNC_ASYNCBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ASYNC_ASYNCBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/fibers/AsyncThreadPoolExecutor.h>

namespace blockstore {
namespace async {

class AsyncBlockStore2 final: public BlockStore2 {
public:
  AsyncBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, size_t numExecutorThreads);

  bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
  bool remove(const BlockId &blockId) override;
  boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
  void store(const BlockId &blockId, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

private:
  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  mutable cpputils::AsyncThreadPoolExecutor _executor;

  DISALLOW_COPY_AND_ASSIGN(AsyncBlockStore2);
};

}
}

#endif
