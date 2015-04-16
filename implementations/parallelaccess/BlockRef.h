#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCKREF_H_
#define BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHEDBLOCKREF_H_

#include <messmer/parallelaccessstore/ParallelAccessStore.h>
#include "../../interface/Block.h"
#include "messmer/cpp-utils/macros.h"
#include <memory>

namespace blockstore {
namespace parallelaccess {
class ParallelAccessBlockStore;

class BlockRef: public Block, public parallelaccessstore::ParallelAccessStore<Block, BlockRef, Key>::ResourceRefBase {
public:
  //TODO Unneccessarily storing Key twice here (in parent class and in _baseBlock).
  BlockRef(Block *baseBlock): Block(baseBlock->key()), _baseBlock(baseBlock) {}

  const void *data() const override {
	return _baseBlock->data();
  }

  void write(const void *source, uint64_t offset, uint64_t size) override {
	return _baseBlock->write(source, offset, size);
  }

  void flush() override {
	return _baseBlock->flush();
  }

  size_t size() const override {
	return _baseBlock->size();
  }

private:
  Block *_baseBlock;

  DISALLOW_COPY_AND_ASSIGN(BlockRef);
};

}
}

#endif
