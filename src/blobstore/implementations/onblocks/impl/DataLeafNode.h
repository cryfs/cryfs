#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATALEAFNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATALEAFNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {

class DataLeafNode: public DataNode {
public:
  DataLeafNode(std::unique_ptr<blockstore::Block> block);
  virtual ~DataLeafNode();

  struct LeafHeader {
    NodeHeader nodeHeader;
  };

  void InitializeEmptyLeaf();
};

}
}

#endif
