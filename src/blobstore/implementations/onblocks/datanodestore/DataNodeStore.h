#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODESTORE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODESTORE_H_

#include <memory>
#include <cpp-utils/macros.h>
#include "DataNodeView.h"
#include <blockstore/utils/BlockId.h>

namespace blockstore{
class Block;
class BlockStore;
}

namespace blobstore {
namespace onblocks {
namespace datanodestore {
class DataNode;
class DataLeafNode;
class DataInnerNode;

class DataNodeStore final {
public:
  DataNodeStore(cpputils::unique_ref<blockstore::BlockStore> blockstore, uint64_t physicalBlocksizeBytes);
  ~DataNodeStore();

  static constexpr uint8_t MAX_DEPTH = 10;

  DataNodeLayout layout() const;

  boost::optional<cpputils::unique_ref<DataNode>> load(const blockstore::BlockId &blockId);
  static cpputils::unique_ref<DataNode> load(cpputils::unique_ref<blockstore::Block> block);

  cpputils::unique_ref<DataLeafNode> createNewLeafNode(cpputils::Data data);
  cpputils::unique_ref<DataInnerNode> createNewInnerNode(uint8_t depth, const std::vector<blockstore::BlockId> &children);

  cpputils::unique_ref<DataNode> createNewNodeAsCopyFrom(const DataNode &source);

  cpputils::unique_ref<DataNode> overwriteNodeWith(cpputils::unique_ref<DataNode> target, const DataNode &source);

  cpputils::unique_ref<DataLeafNode> overwriteLeaf(const blockstore::BlockId &blockId, cpputils::Data data);

  void remove(cpputils::unique_ref<DataNode> node);
  void remove(const blockstore::BlockId &blockId);
  void removeSubtree(uint8_t depth, const blockstore::BlockId &blockId);
  void removeSubtree(cpputils::unique_ref<DataNode> node);

  //TODO Test blocksizeBytes/numBlocks/estimateSpaceForNumBlocksLeft
  uint64_t virtualBlocksizeBytes() const;
  uint64_t numNodes() const;
  uint64_t estimateSpaceForNumNodesLeft() const;
  //TODO Test overwriteNodeWith(), createNodeAsCopyFrom(), removeSubtree()

  void forEachNode(std::function<void (const blockstore::BlockId& nodeId)> callback) const;

private:

  cpputils::unique_ref<blockstore::BlockStore> _blockstore;
  const DataNodeLayout _layout;

  DISALLOW_COPY_AND_ASSIGN(DataNodeStore);
};

}
}
}

#endif
