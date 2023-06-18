#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_TESTFAKE_FAKEBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_TESTFAKE_FAKEBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>

#include <cpp-utils/macros.h>

namespace blockstore {
namespace testfake {
class FakeBlockStore;

class FakeBlock final: public Block {
public:
  FakeBlock(FakeBlockStore *store, const BlockId &blockId, std::shared_ptr<cpputils::Data> data, bool dirty);
  ~FakeBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush();

  size_t size() const override;

  void resize(size_t newSize) override;

private:
  FakeBlockStore *_store;
  std::shared_ptr<cpputils::Data> _data;
  bool _dataChanged;

  DISALLOW_COPY_AND_ASSIGN(FakeBlock);
};

}
}

#endif
