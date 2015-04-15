#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_

#include "../../interface/Block.h"
#include "../../utils/Data.h"

#include "messmer/cpp-utils/macros.h"

namespace blockstore {
namespace testfake {
class FakeBlockStore;

class FakeBlock: public Block {
public:
  FakeBlock(FakeBlockStore *store, const Key &key, std::shared_ptr<Data> data, bool dirty);
  virtual ~FakeBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;

private:
  FakeBlockStore *_store;
  std::shared_ptr<Data> _data;
  bool _dataChanged;

  DISALLOW_COPY_AND_ASSIGN(FakeBlock);
};

}
}

#endif
