#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSBLOCKSTOREADAPTER_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_PARALLELACCESS_PARALLELACCESSBLOCKSTOREADAPTER_H_

#include <messmer/cpp-utils/macros.h>
#include <messmer/parallelaccessstore/ParallelAccessStore.h>
#include "../../interface/BlockStore.h"

namespace blockstore {
namespace parallelaccess {

class ParallelAccessBlockStoreAdapter: public parallelaccessstore::ParallelAccessBaseStore<Block, Key> {
public:
  ParallelAccessBlockStoreAdapter(BlockStore *baseBlockStore)
    :_baseBlockStore(std::move(baseBlockStore)) {
  }

  std::unique_ptr<Block> loadFromBaseStore(const Key &key) override {
	return _baseBlockStore->load(key);
  }

  void removeFromBaseStore(std::unique_ptr<Block> block) override {
	return _baseBlockStore->remove(std::move(block));
  }

private:
  BlockStore *_baseBlockStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessBlockStoreAdapter);
};

}
}

#endif
