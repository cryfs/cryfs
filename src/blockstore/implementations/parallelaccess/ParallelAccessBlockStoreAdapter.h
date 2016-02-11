#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSBLOCKSTOREADAPTER_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSBLOCKSTOREADAPTER_H_

#include <cpp-utils/macros.h>
#include <parallelaccessstore/ParallelAccessStore.h>
#include "../../interface/BlockStore.h"

namespace blockstore {
namespace parallelaccess {

class ParallelAccessBlockStoreAdapter final: public parallelaccessstore::ParallelAccessBaseStore<Block, Key> {
public:
  explicit ParallelAccessBlockStoreAdapter(BlockStore *baseBlockStore)
    :_baseBlockStore(std::move(baseBlockStore)) {
  }

  boost::optional<cpputils::unique_ref<Block>> loadFromBaseStore(const Key &key) override {
	return _baseBlockStore->load(key);
  }

  void removeFromBaseStore(cpputils::unique_ref<Block> block) override {
	return _baseBlockStore->remove(std::move(block));
  }

private:
  BlockStore *_baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessBlockStoreAdapter);
};

}
}

#endif
