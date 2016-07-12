#include "DataTree.h"

#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataInnerNode.h"
#include "../datanodestore/DataLeafNode.h"
#include "../utils/Math.h"

#include "impl/algorithms.h"

#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/pointer/optional_ownership_ptr.h>
#include <cmath>
#include <cpp-utils/assert/assert.h>
#include "impl/LeafTraverser.h"

using blockstore::Key;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;

using std::dynamic_pointer_cast;
using std::function;
using boost::shared_mutex;
using boost::shared_lock;
using boost::unique_lock;
using boost::none;
using std::vector;

using cpputils::dynamic_pointer_move;
using cpputils::optional_ownership_ptr;
using cpputils::WithOwnership;
using cpputils::WithoutOwnership;
using cpputils::unique_ref;
using cpputils::Data;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTree::DataTree(DataNodeStore *nodeStore, unique_ref<DataNode> rootNode)
  : _mutex(), _nodeStore(nodeStore), _rootNode(std::move(rootNode)), _numLeavesCache(none) {
}

DataTree::~DataTree() {
}

void DataTree::removeLastDataLeaf() {
  auto deletePosOrNull = algorithms::GetLowestRightBorderNodeWithMoreThanOneChildOrNull(_nodeStore, _rootNode.get());
  ASSERT(deletePosOrNull.get() != nullptr, "Tree has only one leaf, can't shrink it.");

  deleteLastChildSubtree(deletePosOrNull.get());

  ifRootHasOnlyOneChildReplaceRootWithItsChild();
}

void DataTree::ifRootHasOnlyOneChildReplaceRootWithItsChild() {
  DataInnerNode *rootNode = dynamic_cast<DataInnerNode*>(_rootNode.get());
  ASSERT(rootNode != nullptr, "RootNode is not an inner node");
  if (rootNode->numChildren() == 1) {
    auto child = _nodeStore->load(rootNode->getChild(0)->key());
    ASSERT(child != none, "Couldn't load first child of root node");
    _rootNode = _nodeStore->overwriteNodeWith(std::move(_rootNode), **child);
    _nodeStore->remove(std::move(*child));
  }
}

void DataTree::deleteLastChildSubtree(DataInnerNode *node) {
  auto lastChild = _nodeStore->load(node->LastChild()->key());
  ASSERT(lastChild != none, "Couldn't load last child");
  _nodeStore->removeSubtree(std::move(*lastChild));
  node->removeLastChild();
}

unique_ref<DataLeafNode> DataTree::addDataLeaf() {
  auto insertPosOrNull = algorithms::GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(_nodeStore, _rootNode.get());
  if (insertPosOrNull) {
    return addDataLeafAt(insertPosOrNull.get());
  } else {
    return addDataLeafToFullTree();
  }
}

unique_ref<DataLeafNode> DataTree::addDataLeafAt(DataInnerNode *insertPos) {
  auto new_leaf = _nodeStore->createNewLeafNode();
  auto chain = createChainOfInnerNodes(insertPos->depth()-1, new_leaf.get());
  insertPos->addChild(*chain);
  return new_leaf;
}

optional_ownership_ptr<DataNode> DataTree::createChainOfInnerNodes(unsigned int num, DataNode *child) {
  //TODO This function is implemented twice, once with optional_ownership_ptr, once with unique_ref. Redundancy!
  optional_ownership_ptr<DataNode> chain = cpputils::WithoutOwnership<DataNode>(child);
  for(unsigned int i=0; i<num; ++i) {
    auto newnode = _nodeStore->createNewInnerNode(*chain);
    chain = cpputils::WithOwnership<DataNode>(std::move(newnode));
  }
  return chain;
}

unique_ref<DataNode> DataTree::createChainOfInnerNodes(unsigned int num, unique_ref<DataNode> child) {
  unique_ref<DataNode> chain = std::move(child);
  for(unsigned int i=0; i<num; ++i) {
    chain = _nodeStore->createNewInnerNode(*chain);
  }
  return chain;
}

DataInnerNode* DataTree::increaseTreeDepth(unsigned int levels) {
  ASSERT(levels >= 1, "Parameter out of bounds: tried to increase tree depth by zero.");
  auto copyOfOldRoot = _nodeStore->createNewNodeAsCopyFrom(*_rootNode);
  auto chain = createChainOfInnerNodes(levels-1, copyOfOldRoot.get());
  auto newRootNode = DataNode::convertToNewInnerNode(std::move(_rootNode), *chain);
  DataInnerNode *result = newRootNode.get();
  _rootNode = std::move(newRootNode);
  return result;
}

unique_ref<DataLeafNode> DataTree::addDataLeafToFullTree() {
  DataInnerNode *rootNode = increaseTreeDepth(1);
  auto newLeaf = addDataLeafAt(rootNode);
  return newLeaf;
}

const Key &DataTree::key() const {
  return _rootNode->key();
}

void DataTree::flush() const {
  // By grabbing a lock, we ensure that all modifying functions don't run currently and are therefore flushed
  unique_lock<shared_mutex> lock(_mutex);
  // We also have to flush the root node
  _rootNode->flush();
}

unique_ref<DataNode> DataTree::releaseRootNode() {
  return std::move(_rootNode);
}

//TODO Test numLeaves(), for example also two configurations with same number of bytes but different number of leaves (last leaf has 0 bytes)
uint32_t DataTree::numLeaves() const {
    shared_lock<shared_mutex> lock(_mutex);
    return _numLeaves();
}

uint32_t DataTree::_numLeaves() const {
  if (_numLeavesCache == none) {
    _numLeavesCache = _computeNumLeaves(*_rootNode);
  }
  return *_numLeavesCache;
}

uint32_t DataTree::_forceComputeNumLeaves() const {
  _numLeavesCache = _computeNumLeaves(*_rootNode);
  return *_numLeavesCache;
}

uint32_t DataTree::_computeNumLeaves(const DataNode &node) const {
  const DataLeafNode *leaf = dynamic_cast<const DataLeafNode*>(&node);
  if (leaf != nullptr) {
    return 1;
  }

  const DataInnerNode &inner = dynamic_cast<const DataInnerNode&>(node);
  uint64_t numLeavesInLeftChildren = (inner.numChildren()-1) * leavesPerFullChild(inner);
  auto lastChild = _nodeStore->load(inner.LastChild()->key());
  ASSERT(lastChild != none, "Couldn't load last child");
  uint64_t numLeavesInRightChild = _computeNumLeaves(**lastChild);

  return numLeavesInLeftChildren + numLeavesInRightChild;
}

void DataTree::traverseLeaves(uint32_t beginIndex, uint32_t endIndex, std::function<void (uint32_t index, datanodestore::DataLeafNode* leaf)> onExistingLeaf, std::function<cpputils::Data (uint32_t index)> onCreateLeaf) {
  //TODO Can we allow multiple runs of traverseLeaves() in parallel?
  std::unique_lock<shared_mutex> lock(_mutex);
  ASSERT(beginIndex <= endIndex, "Invalid parameters");

  _rootNode = LeafTraverser(_nodeStore).traverseAndReturnRoot(std::move(_rootNode), beginIndex, endIndex, onExistingLeaf, onCreateLeaf);

  if (_numLeavesCache != none && *_numLeavesCache < endIndex) {
    _numLeavesCache = endIndex;
  }
}

uint32_t DataTree::leavesPerFullChild(const DataInnerNode &root) const {
  return utils::intPow(_nodeStore->layout().maxChildrenPerInnerNode(), (uint64_t)root.depth()-1);
}

uint64_t DataTree::numStoredBytes() const {
  shared_lock<shared_mutex> lock(_mutex);
  return _numStoredBytes();
}

uint64_t DataTree::_numStoredBytes() const {
  return _numStoredBytes(*_rootNode);
}

uint64_t DataTree::_numStoredBytes(const DataNode &root) const {
  const DataLeafNode *leaf = dynamic_cast<const DataLeafNode*>(&root);
  if (leaf != nullptr) {
    return leaf->numBytes();
  }

  const DataInnerNode &inner = dynamic_cast<const DataInnerNode&>(root);
  uint64_t numBytesInLeftChildren = (inner.numChildren()-1) * leavesPerFullChild(inner) * _nodeStore->layout().maxBytesPerLeaf();
  auto lastChild = _nodeStore->load(inner.LastChild()->key());
  ASSERT(lastChild != none, "Couldn't load last child");
  uint64_t numBytesInRightChild = _numStoredBytes(**lastChild);

  return numBytesInLeftChildren + numBytesInRightChild;
}

void DataTree::resizeNumBytes(uint64_t newNumBytes) {
  //TODO Use LeafTraverser here. For growing, that should be trivial. Maybe there is even a way to make it for shrinking? Maybe a callback for inner nodes?
  //TODO Can we resize in parallel? Especially creating new blocks (i.e. encrypting them) is expensive and should be done in parallel.
  boost::upgrade_lock<shared_mutex> lock(_mutex);
  {
    boost::upgrade_to_unique_lock<shared_mutex> exclusiveLock(lock);
    //TODO Faster implementation possible (no addDataLeaf()/removeLastDataLeaf() in a loop, but directly resizing)
    LastLeaf(_rootNode.get())->resize(_nodeStore->layout().maxBytesPerLeaf());
    uint64_t currentNumBytes = _numStoredBytes();
    ASSERT(currentNumBytes % _nodeStore->layout().maxBytesPerLeaf() == 0, "The last leaf is not a max data leaf, although we just resized it to be one.");
    uint32_t currentNumLeaves = currentNumBytes / _nodeStore->layout().maxBytesPerLeaf();
    uint32_t newNumLeaves = std::max(UINT64_C(1), utils::ceilDivision(newNumBytes, _nodeStore->layout().maxBytesPerLeaf()));

    for(uint32_t i = currentNumLeaves; i < newNumLeaves; ++i) {
      addDataLeaf()->resize(_nodeStore->layout().maxBytesPerLeaf());
    }
    for(uint32_t i = currentNumLeaves; i > newNumLeaves; --i) {
      removeLastDataLeaf();
    }
    uint32_t newLastLeafSize = newNumBytes - (newNumLeaves-1)*_nodeStore->layout().maxBytesPerLeaf();
    LastLeaf(_rootNode.get())->resize(newLastLeafSize);

    _numLeavesCache = newNumLeaves;
  }
  ASSERT(newNumBytes == _numStoredBytes(), "We resized to the wrong number of bytes ("+std::to_string(numStoredBytes())+" instead of "+std::to_string(newNumBytes)+")");
}

optional_ownership_ptr<DataLeafNode> DataTree::LastLeaf(DataNode *root) {
  DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root);
  if (leaf != nullptr) {
    return WithoutOwnership(leaf);
  }

  DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root);
  auto lastChild = _nodeStore->load(inner->LastChild()->key());
  ASSERT(lastChild != none, "Couldn't load last child");
  return WithOwnership(LastLeaf(std::move(*lastChild)));
}

unique_ref<DataLeafNode> DataTree::LastLeaf(unique_ref<DataNode> root) {
  auto leaf = dynamic_pointer_move<DataLeafNode>(root);
  if (leaf != none) {
    return std::move(*leaf);
  }
  auto inner = dynamic_pointer_move<DataInnerNode>(root);
  ASSERT(inner != none, "Root node is neither a leaf nor an inner node");
  auto child = _nodeStore->load((*inner)->LastChild()->key());
  ASSERT(child != none, "Couldn't load last child");
  return LastLeaf(std::move(*child));
}

uint64_t DataTree::maxBytesPerLeaf() const {
  return _nodeStore->layout().maxBytesPerLeaf();
}

}
}
}
