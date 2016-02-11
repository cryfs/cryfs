#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>

namespace blockstore {
namespace inmemory {
class InMemoryBlockStore;

class InMemoryBlock final: public Block {
public:
  InMemoryBlock(const Key &key, cpputils::Data size);
  InMemoryBlock(const InMemoryBlock &rhs);
  ~InMemoryBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

private:
  std::shared_ptr<cpputils::Data> _data;
};

}
}

#endif
