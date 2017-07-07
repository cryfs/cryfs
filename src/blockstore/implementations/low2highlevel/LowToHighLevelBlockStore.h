#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_LOWTOHIGHLEVEL_LOWTOHIGHLEVELBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_LOWTOHIGHLEVEL_LOWTOHIGHLEVELBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/cast.h>
#include <iostream>

// TODO Think each function through and make sure it's as performant
//      to use LowToHighLevelBlockStore<OnDiskBlockStore2> as to use
//      OnDiskBlockStore directly (i.e. no additional stores/loads from the disk)
//      (same for other base block stores)

namespace blockstore {
namespace lowtohighlevel {

class LowToHighLevelBlockStore final: public BlockStore {
public:
  LowToHighLevelBlockStore(cpputils::unique_ref<BlockStore2> baseBlockStore);

  Key createKey() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  cpputils::unique_ref<Block> overwrite(const blockstore::Key &key, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
  void remove(const Key &key) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:
  cpputils::unique_ref<BlockStore2> _baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(LowToHighLevelBlockStore);
};

}
}

#endif
