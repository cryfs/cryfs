#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_

#include "../../interface/Block.h"
#include <messmer/cpp-utils/data/Data.h>

namespace blockstore {
namespace inmemory {
class InMemoryBlockStore;

class InMemoryBlock: public Block {
public:
  InMemoryBlock(const Key &key, cpputils::Data size);
  InMemoryBlock(const InMemoryBlock &rhs);
  virtual ~InMemoryBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;

private:
  std::shared_ptr<cpputils::Data> _data;
};

}
}

#endif
