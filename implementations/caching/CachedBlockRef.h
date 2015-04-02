#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCKREF_H_
#define BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCKREF_H_

#include "../../interface/Block.h"
#include "CachingStore.h"

#include "messmer/cpp-utils/macros.h"
#include <memory>

namespace blockstore {
namespace caching {
class CachingBlockStore;

class CachedBlockRef: public Block, public CachingStore<Block, CachedBlockRef, Key>::CachedResource {
public:
  CachedBlockRef(Block *baseBlock);
  virtual ~CachedBlockRef();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;

private:
  Block *_baseBlock;

  DISALLOW_COPY_AND_ASSIGN(CachedBlockRef);
};

}
}

#endif
