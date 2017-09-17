#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREESTORE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_DATATREESTORE_H_

#include <memory>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <blockstore/utils/BlockId.h>
#include <boost/optional.hpp>
#include "../datanodestore/DataNodeStore.h"

namespace blobstore {
namespace onblocks {
namespace datatreestore {
class DataTree;

class DataTreeStore final {
public:
  DataTreeStore(cpputils::unique_ref<datanodestore::DataNodeStore> nodeStore);
  ~DataTreeStore();

  boost::optional<cpputils::unique_ref<DataTree>> load(const blockstore::BlockId &blockId);

  cpputils::unique_ref<DataTree> createNewTree();

  void remove(cpputils::unique_ref<DataTree> tree);
  void remove(const blockstore::BlockId &blockId);

  //TODO Test blocksizeBytes/numBlocks/estimateSpaceForNumBlocksLeft
  uint64_t virtualBlocksizeBytes() const;
  uint64_t numNodes() const;
  uint64_t estimateSpaceForNumNodesLeft() const;

private:
  cpputils::unique_ref<datanodestore::DataNodeStore> _nodeStore;

  DISALLOW_COPY_AND_ASSIGN(DataTreeStore);
};

inline uint64_t DataTreeStore::numNodes() const {
    return _nodeStore->numNodes();
}

inline uint64_t DataTreeStore::estimateSpaceForNumNodesLeft() const {
    return _nodeStore->estimateSpaceForNumNodesLeft();
}

inline uint64_t DataTreeStore::virtualBlocksizeBytes() const {
    return _nodeStore->virtualBlocksizeBytes();
}

}
}
}

#endif
