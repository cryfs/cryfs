#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <unordered_map>

namespace blockstore {
namespace inmemory {

class InMemoryBlockStore2 final: public BlockStore2 {
public:
  InMemoryBlockStore2();

  bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
  bool remove(const BlockId &blockId) override;
  boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
  void store(const BlockId &blockId, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

private:
  std::vector<BlockId> _allBlockIds() const;
  bool _tryCreate(const BlockId &blockId, const cpputils::Data &data);

  std::unordered_map<BlockId, cpputils::Data> _blocks;
  mutable std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(InMemoryBlockStore2);
};

}
}

#endif
