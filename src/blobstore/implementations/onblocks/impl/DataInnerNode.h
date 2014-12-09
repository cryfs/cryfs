#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATAINNERNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATAINNERNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {

class DataInnerNode: public DataNode {
public:
  DataInnerNode(std::unique_ptr<blockstore::Block> block);
  virtual ~DataInnerNode();

  struct InnerNodeHeader {
    NodeHeader nodeHeader;
  };

  void InitializeEmptyInnerNode();
};

}
}

#endif
