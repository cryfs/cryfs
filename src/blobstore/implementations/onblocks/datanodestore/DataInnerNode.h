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
  static cpputils::unique_ref<DataInnerNode> InitializeNewNode(cpputils::unique_ref<blockstore::Block> block, const DataNodeLayout &layout, uint8_t depth, const std::vector<blockstore::BlockId> &children);
  static cpputils::unique_ref<DataInnerNode> CreateNewNode(blockstore::BlockStore *blockStore, const DataNodeLayout &layout, uint8_t depth, const std::vector<blockstore::BlockId> &children);

  using ChildEntry = DataInnerNode_ChildEntry;

  DataInnerNode(DataNodeView block);
  ~DataInnerNode() override;

  uint32_t maxStoreableChildren() const;

  ChildEntry readChild(unsigned int index) const;
  ChildEntry readLastChild() const;

  uint32_t numChildren() const;

  void addChild(const DataNode &child_blockId);

  void removeLastChild();

private:
  void _writeChild(unsigned int index, const ChildEntry& child);
  void _writeLastChild(const ChildEntry& child);
  static cpputils::Data _serializeChildren(const std::vector<blockstore::BlockId> &children);

  DISALLOW_COPY_AND_ASSIGN(DataInnerNode);
};

}
}
}

#endif
