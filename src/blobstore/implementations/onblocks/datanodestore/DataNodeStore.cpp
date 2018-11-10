#include "DataInnerNode.h"
#include "DataLeafNode.h"
#include "DataNodeStore.h"
#include <blockstore/interface/BlockStore.h>
#include <blockstore/interface/Block.h>
#include <blockstore/utils/BlockStoreUtils.h>
#include <cpp-utils/assert/assert.h>

using blockstore::BlockStore;
using blockstore::Block;
using blockstore::BlockId;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using std::runtime_error;
using boost::optional;
using boost::none;
using std::vector;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataNodeStore::DataNodeStore(unique_ref<BlockStore> blockstore, uint64_t physicalBlocksizeBytes)
: _blockstore(std::move(blockstore)), _layout(_blockstore->blockSizeFromPhysicalBlockSize(physicalBlocksizeBytes)) {
}

DataNodeStore::~DataNodeStore() {
}

unique_ref<DataNode> DataNodeStore::load(unique_ref<Block> block) {
  DataNodeView node(std::move(block));

  if (node.Depth() == 0) {
    return make_unique_ref<DataLeafNode>(std::move(node));
  } else if (node.Depth() <= MAX_DEPTH) {
    return make_unique_ref<DataInnerNode>(std::move(node));
  } else {
    throw runtime_error("Tree is to deep. Data corruption?");
  }
}

unique_ref<DataInnerNode> DataNodeStore::createNewInnerNode(uint8_t depth, const vector<BlockId> &children) {
  ASSERT(children.size() >= 1, "Inner node must have at least one child");
  return DataInnerNode::CreateNewNode(_blockstore.get(), _layout, depth, children);
}

unique_ref<DataLeafNode> DataNodeStore::createNewLeafNode(Data data) {
  return DataLeafNode::CreateNewNode(_blockstore.get(), _layout, std::move(data));
}

unique_ref<DataLeafNode> DataNodeStore::overwriteLeaf(const BlockId &blockId, Data data) {
  return DataLeafNode::OverwriteNode(_blockstore.get(), _layout, blockId, std::move(data));
}

optional<unique_ref<DataNode>> DataNodeStore::load(const BlockId &blockId) {
  auto block = _blockstore->load(blockId);
  if (block == none) {
    return none;
  } else {
    ASSERT((*block)->size() == _layout.blocksizeBytes(), "Loading block of wrong size");
    return load(std::move(*block));
  }
}

unique_ref<DataNode> DataNodeStore::createNewNodeAsCopyFrom(const DataNode &source) {
  ASSERT(source.node().layout().blocksizeBytes() == _layout.blocksizeBytes(), "Source node has wrong layout. Is it from the same DataNodeStore?");
  auto newBlock = blockstore::utils::copyToNewBlock(_blockstore.get(), source.node().block());
  return load(std::move(newBlock));
}

unique_ref<DataNode> DataNodeStore::overwriteNodeWith(unique_ref<DataNode> target, const DataNode &source) {
  ASSERT(target->node().layout().blocksizeBytes() == _layout.blocksizeBytes(), "Target node has wrong layout. Is it from the same DataNodeStore?");
  ASSERT(source.node().layout().blocksizeBytes() == _layout.blocksizeBytes(), "Source node has wrong layout. Is it from the same DataNodeStore?");
  auto targetBlock = target->node().releaseBlock();
  cpputils::destruct(std::move(target)); // Call destructor
  blockstore::utils::copyTo(targetBlock.get(), source.node().block());
  return DataNodeStore::load(std::move(targetBlock));
}

void DataNodeStore::remove(unique_ref<DataNode> node) {
  BlockId blockId = node->blockId();
  cpputils::destruct(std::move(node));
  remove(blockId);
}

void DataNodeStore::remove(const BlockId &blockId) {
  _blockstore->remove(blockId);
}

void DataNodeStore::removeSubtree(unique_ref<DataNode> node) {
  auto leaf = dynamic_pointer_move<DataLeafNode>(node);
  if (leaf != none) {
    remove(std::move(*leaf));
    return;
  }

  auto inner = dynamic_pointer_move<DataInnerNode>(node);
  ASSERT(inner != none, "Is neither a leaf nor an inner node");
  for (uint32_t i = 0; i < (*inner)->numChildren(); ++i) {
    removeSubtree((*inner)->depth()-1, (*inner)->readChild(i).blockId());
  }
  remove(std::move(*inner));
}

void DataNodeStore::removeSubtree(uint8_t depth, const BlockId &blockId) {
  if (depth == 0) {
    remove(blockId);
  } else {
    auto node = load(blockId);
    ASSERT(node != none, "Node for removeSubtree not found");

    auto inner = dynamic_pointer_move<DataInnerNode>(*node);
    ASSERT(inner != none, "Is not an inner node, but depth was not zero");
    ASSERT((*inner)->depth() == depth, "Wrong depth given");
    for (uint32_t i = 0; i < (*inner)->numChildren(); ++i) {
      removeSubtree(depth-1, (*inner)->readChild(i).blockId());
    }
    remove(std::move(*inner));
  }
}

uint64_t DataNodeStore::numNodes() const {
  return _blockstore->numBlocks();
}

uint64_t DataNodeStore::estimateSpaceForNumNodesLeft() const {
  return _blockstore->estimateNumFreeBytes() / _layout.blocksizeBytes();
}

uint64_t DataNodeStore::virtualBlocksizeBytes() const {
  return _layout.blocksizeBytes();
}

DataNodeLayout DataNodeStore::layout() const {
  return _layout;
}

void DataNodeStore::forEachNode(std::function<void (const BlockId& nodeId)> callback) const {
  _blockstore->forEachBlock(std::move(callback));
}

}
}
}
