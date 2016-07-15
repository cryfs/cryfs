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

unique_ref<DataInnerNode> DataNodeStore::createNewInnerNode(uint8_t depth, const vector<Key> &children) {
  ASSERT(children.size() >= 1, "Inner node must have at least one child");
  return DataInnerNode::CreateNewNode(_blockstore.get(), _layout, depth, children);
}

unique_ref<DataLeafNode> DataNodeStore::createNewLeafNode(Data data) {
  return DataLeafNode::CreateNewNode(_blockstore.get(), _layout, std::move(data));
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
  Key key = node->key();
  cpputils::destruct(std::move(node));
  remove(key);
}

void DataNodeStore::remove(const Key &key) {
  _blockstore->remove(key);
}


void DataNodeStore::removeSubtree(const Key &key) {
  auto node = load(key);
  ASSERT(node != none, "Node for removeSubtree not found");

  auto inner = dynamic_pointer_move<DataInnerNode>(*node);
  if (inner == none) {
    ASSERT((*node)->depth() == 0, "If it's not an inner node, it has to be a leaf.");
    remove(std::move(*node));
  } else {
    _removeSubtree(std::move(*inner));
  }
}

void DataNodeStore::_removeSubtree(cpputils::unique_ref<DataInnerNode> node) {
  if (node->depth() == 1) {
    for (uint32_t i = 0; i < node->numChildren(); ++i) {
      remove(node->getChild(i)->key());
    }
  } else {
    ASSERT(node->depth() > 1, "This if branch is only called when our children are inner nodes.");
    for (uint32_t i = 0; i < node->numChildren(); ++i) {
      auto child = load(node->getChild(i)->key());
      ASSERT(child != none, "Child not found");
      auto inner = dynamic_pointer_move<DataInnerNode>(*child);
      ASSERT(inner != none, "Expected inner node as child");
      _removeSubtree(std::move(*inner));
    }
  }
  remove(std::move(node));
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

}
}
}
