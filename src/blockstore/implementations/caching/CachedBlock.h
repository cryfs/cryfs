#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCK2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCK2_H_

#include "../../interface/Block.h"
#include "BaseBlockWrapper.h"

#include <cpp-utils/pointer/unique_ref.h>

namespace blockstore {
namespace caching {
class CachingBlockStore;

class CachedBlock final: public Block {
public:
  CachedBlock(BaseBlockWrapper baseBlock, CachingBlockStore *blockStore);
  ~CachedBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;
  void flush() override;

  size_t size() const override;

  void resize(size_t newSize) override;

  BaseBlockWrapper releaseBaseBlockWrapper();

private:
  CachingBlockStore *_blockStore;
  BaseBlockWrapper _baseBlock;

  DISALLOW_COPY_AND_ASSIGN(CachedBlock);
};

}
}

#endif
