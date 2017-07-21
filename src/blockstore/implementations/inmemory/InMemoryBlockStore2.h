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

  bool tryCreate(const Key &key, const cpputils::Data &data) override;
  bool remove(const Key &key) override;
  boost::optional<cpputils::Data> load(const Key &key) const override;
  void store(const Key &key, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:
  std::vector<Key> _allBlockKeys() const;
  bool _tryCreate(const Key &key, const cpputils::Data &data);

  std::unordered_map<Key, cpputils::Data> _blocks;
  mutable std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(InMemoryBlockStore2);
};

}
}

#endif
