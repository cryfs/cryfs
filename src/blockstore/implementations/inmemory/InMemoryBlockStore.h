#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_INMEMORY_INMEMORYBLOCKSTORE_H_

#include <blockstore/interface/helpers/BlockStoreWithRandomKeys.h>
#include "fspp/utils/macros.h"

#include <mutex>
#include <map>

namespace blockstore {
namespace inmemory {
class InMemoryBlock;

class InMemoryBlockStore: public BlockStoreWithRandomKeys {
public:
  InMemoryBlockStore();

  std::unique_ptr<BlockWithKey> create(const std::string &key, size_t size) override;
  std::unique_ptr<Block> load(const std::string &key) override;

private:
  std::map<std::string, InMemoryBlock> _blocks;

  DISALLOW_COPY_AND_ASSIGN(InMemoryBlockStore);
};

}
}

#endif
