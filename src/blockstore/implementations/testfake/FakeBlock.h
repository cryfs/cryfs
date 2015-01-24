#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCK_H_

#include <blockstore/interface/Block.h>
#include <blockstore/utils/Data.h>

#include "fspp/utils/macros.h"

namespace blockstore {
namespace testfake {
class FakeBlockStore;

class FakeBlock: public Block {
public:
  FakeBlock(FakeBlockStore *store, const Key &key, std::shared_ptr<Data> data);
  virtual ~FakeBlock();

  void *data() override;
  const void *data() const override;

  void flush() override;

  size_t size() const override;

private:
  FakeBlockStore *_store;
  std::shared_ptr<Data> _data;

  DISALLOW_COPY_AND_ASSIGN(FakeBlock);
};

}
}

#endif
