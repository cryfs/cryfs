#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_LOWTOHIGHLEVEL_LOWTOHIGHLEVELBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_LOWTOHIGHLEVEL_LOWTOHIGHLEVELBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>
#include "../../interface/BlockStore.h"
#include "../../interface/BlockStore2.h"

#include <cpp-utils/macros.h>
#include <memory>
#include <iostream>
#include <boost/optional.hpp>
#include <cpp-utils/crypto/symmetric/Cipher.h>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/data/DataUtils.h>
#include <mutex>
#include <cpp-utils/logging/logging.h>
#include "LowToHighLevelBlockStore.h"

namespace blockstore {
namespace lowtohighlevel {

class LowToHighLevelBlock final: public Block {
public:
  static boost::optional<cpputils::unique_ref<LowToHighLevelBlock>> TryCreateNew(BlockStore2 *baseBlockStore, const BlockId &blockId, cpputils::Data data);
  static cpputils::unique_ref<LowToHighLevelBlock> Overwrite(BlockStore2 *baseBlockStore, const BlockId &blockId, cpputils::Data data);
  static boost::optional<cpputils::unique_ref<LowToHighLevelBlock>> Load(BlockStore2 *baseBlockStore, const BlockId &blockId);

  LowToHighLevelBlock(const BlockId &blockId, cpputils::Data data, BlockStore2 *baseBlockStore);
  ~LowToHighLevelBlock() override;

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t count) override;
  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

private:
  BlockStore2 *_baseBlockStore;
  cpputils::Data _data;
  bool _dataChanged;
  std::mutex _mutex;

  void _storeToBaseBlock();

  DISALLOW_COPY_AND_ASSIGN(LowToHighLevelBlock);
};

}
}

#endif
