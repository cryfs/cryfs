#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_

#include "../../interface/Block.h"
#include "../../utils/Data.h"

namespace blockstore {
namespace inmemory {
class InMemoryBlockStore;

class InMemoryBlock: public Block {
public:
  InMemoryBlock(const Key &key, Data size);
  InMemoryBlock(const InMemoryBlock &rhs);
  virtual ~InMemoryBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;

private:
  std::shared_ptr<Data> _data;
};

}
}

#endif
