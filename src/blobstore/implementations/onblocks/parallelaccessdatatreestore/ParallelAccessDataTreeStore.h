#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTORE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_PARALLELACCESSDATATREESTORE_PARALLELACCESSDATATREESTORE_H_

#include <memory>
#include <cpp-utils/macros.h>
#include <blockstore/utils/BlockId.h>
#include <parallelaccessstore/ParallelAccessStore.h>
#include "../datatreestore/DataTreeStore.h"

namespace blobstore {
namespace onblocks {
namespace parallelaccessdatatreestore {
class DataTreeRef;

//TODO Test CachingDataTreeStore

class ParallelAccessDataTreeStore final {
public:
  ParallelAccessDataTreeStore(cpputils::unique_ref<datatreestore::DataTreeStore> dataTreeStore);
  ~ParallelAccessDataTreeStore();

  boost::optional<cpputils::unique_ref<DataTreeRef>> load(const blockstore::BlockId &blockId);

  cpputils::unique_ref<DataTreeRef> createNewTree();

  void remove(cpputils::unique_ref<DataTreeRef> tree);
  void remove(const blockstore::BlockId &blockId);

  //TODO Test blocksizeBytes/numBlocks/estimateSpaceForNumBlocksLeft
  uint64_t virtualBlocksizeBytes() const;
  uint64_t numNodes() const;
  uint64_t estimateSpaceForNumNodesLeft() const;

private:
  cpputils::unique_ref<datatreestore::DataTreeStore> _dataTreeStore;
  parallelaccessstore::ParallelAccessStore<datatreestore::DataTree, DataTreeRef, blockstore::BlockId> _parallelAccessStore;

  DISALLOW_COPY_AND_ASSIGN(ParallelAccessDataTreeStore);
};

inline uint64_t ParallelAccessDataTreeStore::virtualBlocksizeBytes() const {
    return _dataTreeStore->virtualBlocksizeBytes();
}

inline uint64_t ParallelAccessDataTreeStore::numNodes() const {
    return _dataTreeStore->numNodes();
}

inline uint64_t ParallelAccessDataTreeStore::estimateSpaceForNumNodesLeft() const {
    return _dataTreeStore->estimateSpaceForNumNodesLeft();
}

}
}
}

#endif
