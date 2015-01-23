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
  DataLeafNode(DataNodeView block, const blockstore::Key &key);
  virtual ~DataLeafNode();

  static constexpr uint32_t MAX_STORED_BYTES = DataNodeView::DATASIZE_BYTES;

  void InitializeNewNode();

  void *data();
  const void *data() const;

  uint32_t numBytes() const;

  void resize(uint32_t size);

private:
  void fillDataWithZeroesFromTo(off_t begin, off_t end);
};

}
}
}

#endif
