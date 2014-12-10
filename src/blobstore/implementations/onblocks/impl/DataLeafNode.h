#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATALEAFNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATALEAFNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {

class DataLeafNode: public DataNode {
public:
  DataLeafNode(DataNodeView block);
  virtual ~DataLeafNode();

  void read(off_t offset, size_t count, blockstore::Data *result) override;
  void write(off_t offset, size_t count, const blockstore::Data &data) override;

  void InitializeNewLeafNode();

  uint64_t numBytesInThisNode() override;

};

}
}

#endif
