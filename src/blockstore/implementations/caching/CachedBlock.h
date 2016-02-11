#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCK_H_

#include "../../interface/Block.h"

#include <cpp-utils/pointer/unique_ref.h>

namespace blockstore {
namespace caching {
class CachingBlockStore;

class CachedBlock final: public Block {
public:
  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  CachedBlock(cpputils::unique_ref<Block> baseBlock, CachingBlockStore *blockStore);
  ~CachedBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;
  void flush() override;

  size_t size() const override;

  void resize(size_t newSize) override;

  cpputils::unique_ref<Block> releaseBlock();

private:
  CachingBlockStore *_blockStore;
  cpputils::unique_ref<Block> _baseBlock;

  DISALLOW_COPY_AND_ASSIGN(CachedBlock);
};

}
}

#endif
