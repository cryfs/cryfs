#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATAINNERNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATAINNERNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {

class DataInnerNode: public DataNode {
public:
  DataInnerNode(DataNodeView block);
  virtual ~DataInnerNode();

  struct ChildEntry {
    uint32_t numBlocksInThisAndLeftwardNodes;
    uint8_t key[Key::KEYLENGTH_BINARY];
  };

  static constexpr uint32_t MAX_STORED_CHILDREN = DataNodeView::DATASIZE_BYTES / sizeof(ChildEntry);

  void InitializeNewNode();

  void read(off_t offset, size_t count, blockstore::Data *result) override;
  void write(off_t offset, size_t count, const blockstore::Data &data) override;

  uint64_t numBytesInThisNode() override;
  void resize(uint64_t newsize_bytes) override;

private:

  ChildEntry *ChildrenBegin();
  ChildEntry *ChildrenEnd();
  ChildEntry *ChildrenLast();

  uint64_t readFromChild(const ChildEntry *child, off_t inner_offset, size_t count, uint8_t *target);

  ChildEntry *ChildContainingFirstByteAfterOffset(off_t offset);
  uint64_t numBytesInChildAndLeftwardSiblings(const ChildEntry *child);
  uint64_t numBytesInLeftwardSiblings(const ChildEntry *child);
  uint64_t numBytesInChild(const ChildEntry *child);
};

}
}

#endif
