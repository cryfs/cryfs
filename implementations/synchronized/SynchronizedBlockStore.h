#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_SYNCHRONIZED_SYNCHRONIZEDBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_SYNCHRONIZED_SYNCHRONIZEDBLOCKSTORE_H_

#include <boost/filesystem.hpp>
#include "../../interface/BlockStore.h"

#include "messmer/cpp-utils/macros.h"

#include <mutex>
#include <memory>

namespace blockstore {
namespace synchronized {

class SynchronizedBlockStore: public BlockStore {
public:
  SynchronizedBlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  std::unique_ptr<Block> create(size_t size) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

private:
  std::unique_ptr<BlockStore> _baseBlockStore;
  mutable std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(SynchronizedBlockStore);
};

}
}

#endif
