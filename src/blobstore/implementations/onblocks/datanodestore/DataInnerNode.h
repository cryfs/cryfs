#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {

class DataInnerNode: public DataNode {
public:
  DataInnerNode(DataNodeView block, const blockstore::Key &key);
  virtual ~DataInnerNode();

  struct ChildEntry {
    uint8_t key[blockstore::Key::KEYLENGTH_BINARY];
  };

  static constexpr uint32_t MAX_STORED_CHILDREN = DataNodeView::DATASIZE_BYTES / sizeof(ChildEntry);

  void InitializeNewNode(const DataNode &first_child);

  ChildEntry *ChildrenBegin();
  ChildEntry *ChildrenEnd();
  const ChildEntry *ChildrenBegin() const;
  const ChildEntry *ChildrenEnd() const;

  const ChildEntry *RightmostExistingChild() const;

  uint32_t numChildren() const;
};

}
}
}

#endif
