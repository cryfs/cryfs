#include "DataInnerNode.h"
#include "DataLeafNode.h"
#include "DataNodeStore.h"
#include <blockstore/interface/BlockStore.h>
#include <blockstore/interface/Block.h>
#include <blockstore/utils/BlockStoreUtils.h>
#include <cpp-utils/assert/assert.h>

using blockstore::BlockStore;
using blockstore::Block;
using blockstore::Key;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::runtime_error;
using boost::optional;
using boost::none;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataNodeStore::DataNodeStore(unique_ref<BlockStore> blockstore, uint64_t physicalBlocksizeBytes)
: _blockstore(std::move(blockstore)), _layout(_blockstore->blockSizeFromPhysicalBlockSize(physicalBlocksizeBytes)) {
}

DataNodeStore::~DataNodeStore() {
}

unique_ref<DataNode> DataNodeStore::load(unique_ref<Block> block) {
  ASSERT(block->size() == _layout.blocksizeBytes(), "Loading block of wrong size");
  DataNodeView node(std::move(block));

  if (node.Depth() == 0) {
    return make_unique_ref<DataLeafNode>(std::move(node));
  } else if (node.Depth() <= MAX_DEPTH) {
    return make_unique_ref<DataInnerNode>(std::move(node));
  } else {
    throw runtime_error("Tree is to deep. Data corruption?");
  }
}

unique_ref<DataInnerNode> DataNodeStore::createNewInnerNode(const DataNode &first_child) {
  ASSERT(first_child.node().layout().blocksizeBytes() == _layout.blocksizeBytes(), "Source node has wrong layout. Is it from the same DataNodeStore?");
  //TODO Initialize block and then create it in the blockstore - this is more efficient than creating it and then writing to it
  auto block = _blockstore->create(Data(_layout.blocksizeBytes()).FillWithZeroes());
  return DataInnerNode::InitializeNewNode(std::move(block), first_child);
}

unique_ref<DataLeafNode> DataNodeStore::createNewLeafNode() {
  //TODO Initialize block and then create it in the blockstore - this is more efficient than creating it and then writing to it
  auto block = _blockstore->create(Data(_layout.blocksizeBytes()).FillWithZeroes());
  return DataLeafNode::InitializeNewNode(std::move(block));
}

optional<unique_ref<DataNode>> DataNodeStore::load(const Key &key) {
  auto block = _blockstore->load(key);
  if (block == none) {
    return none;
  } else {
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
  Key key = target->key();
  {
    auto targetBlock = target->node().releaseBlock();
    cpputils::destruct(std::move(target)); // Call destructor
    blockstore::utils::copyTo(targetBlock.get(), source.node().block());
  }
  auto loaded = load(key);
  ASSERT(loaded != none, "Couldn't load the target node after overwriting it");
  return std::move(*loaded);
}

void DataNodeStore::remove(unique_ref<DataNode> node) {
  auto block = node->node().releaseBlock();
  cpputils::destruct(std::move(node)); // Call destructor
  _blockstore->remove(std::move(block));
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

void DataNodeStore::removeSubtree(unique_ref<DataNode> node) {
  //TODO Make this faster by not loading the leaves but just deleting them. Can be recognized, because of the depth of their parents.
  DataInnerNode *inner = dynamic_cast<DataInnerNode*>(node.get());
  if (inner != nullptr) {
    for (uint32_t i = 0; i < inner->numChildren(); ++i) {
      auto child = load(inner->getChild(i)->key());
      ASSERT(child != none, "Couldn't load child node");
      removeSubtree(std::move(*child));
    }
  }
  remove(std::move(node));
}

DataNodeLayout DataNodeStore::layout() const {
  return _layout;
}

}
}
}
