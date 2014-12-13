#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATAINNERNODE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_DATAINNERNODE_H_

#include "DataNode.h"

namespace blobstore {
namespace onblocks {

class DataInnerNode: public DataNode {
public:
  DataInnerNode(DataNodeView block, const Key &key, DataNodeStore *nodestorage);
  virtual ~DataInnerNode();

  struct ChildEntry {
    uint8_t key[Key::KEYLENGTH_BINARY];
  };

  static constexpr uint32_t MAX_STORED_CHILDREN = DataNodeView::DATASIZE_BYTES / sizeof(ChildEntry);

  void InitializeNewNode(const DataNode &first_child);

  void read(off_t offset, size_t count, blockstore::Data *result) const override;
  void write(off_t offset, size_t count, const blockstore::Data &data) override;

  uint64_t numBytesInThisNode() const override;
  void resize(uint64_t newsize_bytes) override;

private:
  ChildEntry *ChildrenBegin();
  const ChildEntry *ChildrenBegin() const;
  const ChildEntry *ChildrenEnd() const;
  const ChildEntry *RightmostChild() const;

  uint64_t readFromChild(const ChildEntry *child, off_t inner_offset, size_t count, uint8_t *target) const;

  uint32_t numChildren() const;
  uint32_t maxNumDataBlocksPerChild() const;
  uint64_t maxNumBytesPerChild() const;
  uint64_t numBytesInNonRightmostChildrenSum() const;
  uint64_t numBytesInRightmostChild() const;
  const ChildEntry *ChildContainingFirstByteAfterOffset(off_t offset) const;
};

}
}

#endif
