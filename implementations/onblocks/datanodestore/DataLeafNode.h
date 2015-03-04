#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATALEAFNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATALEAFNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataInnerNode;

class DataLeafNode: public DataNode {
public:
  static std::unique_ptr<DataLeafNode> InitializeNewNode(std::unique_ptr<blockstore::Block> block);

  DataLeafNode(DataNodeView block);
  virtual ~DataLeafNode();

  uint32_t maxStoreableBytes() const;

  void read(void *target, uint64_t offset, uint64_t size) const;
  void write(const void *source, uint64_t offset, uint64_t size);

  uint32_t numBytes() const;

  void resize(uint32_t size);

private:
  void fillDataWithZeroesFromTo(off_t begin, off_t end);
};

}
}
}

#endif
