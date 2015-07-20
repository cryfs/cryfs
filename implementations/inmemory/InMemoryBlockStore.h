#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE_H_

#include "../../interface/helpers/BlockStoreWithRandomKeys.h"
#include <messmer/cpp-utils/macros.h>

#include <mutex>
#include <map>

namespace blockstore {
namespace inmemory {
class InMemoryBlock;

class InMemoryBlockStore: public BlockStoreWithRandomKeys {
public:
  InMemoryBlockStore();

  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

private:
  std::map<std::string, InMemoryBlock> _blocks;

  DISALLOW_COPY_AND_ASSIGN(InMemoryBlockStore);
};

}
}

#endif
