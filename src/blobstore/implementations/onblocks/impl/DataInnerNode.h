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
    uint8_t key[Key::KEYLENGTH_BINARY];
  };

  static constexpr uint32_t MAX_STORED_CHILDREN = DataNodeView::DATASIZE_BYTES / sizeof(ChildEntry);

  void InitializeNewNode(const Key &first_child_key, const DataNodeView &first_child);

  void read(off_t offset, size_t count, blockstore::Data *result) const override;
  void write(off_t offset, size_t count, const blockstore::Data &data) override;

  uint64_t numBytesInThisNode() const override;
  void resize(uint64_t newsize_bytes) override;

private:
  ChildEntry *ChildrenBegin();
  const ChildEntry *ChildrenBegin() const;
  const ChildEntry *ChildrenEnd() const;
  const ChildEntry *ChildrenLast() const;

  uint64_t readFromChild(const ChildEntry *child, off_t inner_offset, size_t count, uint8_t *target) const;

  const ChildEntry *ChildContainingFirstByteAfterOffset(off_t offset) const;
  uint64_t numBytesInChildAndLeftwardSiblings(const ChildEntry *child) const;
  uint64_t numBytesInLeftwardSiblings(const ChildEntry *child) const;
  uint64_t numBytesInChild(const ChildEntry *child) const;
};

}
}

#endif
