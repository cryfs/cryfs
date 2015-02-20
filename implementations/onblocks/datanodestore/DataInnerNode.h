#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATAINNERNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {
namespace datanodestore {

class DataInnerNode: public DataNode {
public:
  static std::unique_ptr<DataInnerNode> InitializeNewNode(std::unique_ptr<blockstore::Block> block, const DataNode &first_child_key);

  DataInnerNode(DataNodeView block);
  virtual ~DataInnerNode();

  struct ChildEntry {
  public:
    blockstore::Key key() const {
      return blockstore::Key::FromBinary(_keydata);
    }
  private:
    void setKey(const blockstore::Key &key) {
      key.ToBinary(_keydata);
    }
    friend class DataInnerNode;
    uint8_t _keydata[blockstore::Key::KEYLENGTH_BINARY];
    DISALLOW_COPY_AND_ASSIGN(ChildEntry);
  };

  static constexpr uint32_t MAX_STORED_CHILDREN = DataNodeView::DATASIZE_BYTES / sizeof(ChildEntry);

  uint8_t depth() const;

  ChildEntry *getChild(unsigned int index);
  const ChildEntry *getChild(unsigned int index) const;

  uint32_t numChildren() const;

  void addChild(const DataNode &child_key);

  ChildEntry *LastChild();
  const ChildEntry *LastChild() const;

private:

  ChildEntry *ChildrenBegin();
  ChildEntry *ChildrenEnd();
  const ChildEntry *ChildrenBegin() const;
  const ChildEntry *ChildrenEnd() const;
};

}
}
}

#endif
