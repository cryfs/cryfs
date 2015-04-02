#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_SYNCHRONIZED_SYNCHRONIZEDBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_SYNCHRONIZED_SYNCHRONIZEDBLOCKSTORE_H_

#include "messmer/cpp-utils/macros.h"
#include <memory>
#include <mutex>
#include <map>
#include <future>

#include "../../interface/BlockStore.h"

namespace blockstore {
namespace caching {

class CachingBlockStore: public BlockStore {
public:
  CachingBlockStore(std::unique_ptr<BlockStore> baseBlockStore);

  std::unique_ptr<Block> create(size_t size) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

  void release(const Block *block);

private:
  struct OpenBlock {
	OpenBlock(std::unique_ptr<Block> block_): block(std::move(block_)), refCount(0) {}
	Block *getReference() {
	  ++refCount;
	  return block.get();
	}
	void releaseReference() {
	  --refCount;
	}
    std::unique_ptr<Block> block;
    uint32_t refCount;
  };
  std::unique_ptr<BlockStore> _baseBlockStore;
  std::map<Key, OpenBlock> _openBlocks;
  std::mutex _mutex;
  std::map<Key, std::promise<std::unique_ptr<Block>>> _blocksToRemove;

  std::unique_ptr<Block> _addOpenBlock(std::unique_ptr<Block> block);

  DISALLOW_COPY_AND_ASSIGN(CachingBlockStore);
};

}
}

#endif
