#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_H_

#include "DataNode.h"
#include "DataInnerNode_ChildEntry.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {

class DataInnerNode final: public DataNode {
public:
  static cpputils::unique_ref<DataInnerNode> InitializeNewNode(cpputils::unique_ref<blockstore::Block> block, const DataNode &first_child_key);

  DataInnerNode(DataNodeView block);
  ~DataInnerNode();

  using ChildEntry = DataInnerNode_ChildEntry;

  uint32_t maxStoreableChildren() const;

  ChildEntry *getChild(unsigned int index);
  const ChildEntry *getChild(unsigned int index) const;

  uint32_t numChildren() const;

  void addChild(const DataNode &child_key);

  void removeLastChild();

  ChildEntry *LastChild();
  const ChildEntry *LastChild() const;

private:

  ChildEntry *ChildrenBegin();
  ChildEntry *ChildrenEnd();
  const ChildEntry *ChildrenBegin() const;
  const ChildEntry *ChildrenEnd() const;

  DISALLOW_COPY_AND_ASSIGN(DataInnerNode);
};

}
}
}

#endif
