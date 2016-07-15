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
  : _mutex(), _nodeStore(nodeStore), _rootNode(std::move(rootNode)), _key(_rootNode->key()), _numLeavesCache(none) {
}

DataTree::~DataTree() {
}

const Key &DataTree::key() const {
  return _key;
}

void DataTree::flush() const {
  // By grabbing a lock, we ensure that all modifying functions don't run currently and are therefore flushed
  unique_lock<shared_mutex> lock(_mutex);
  // We also have to flush the root node
  _rootNode->flush();
}

unique_ref<DataNode> DataTree::releaseRootNode() {
  unique_lock<shared_mutex> lock(_mutex); // Lock ensures that the root node is currently set (traversing unsets it temporarily)
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
  unique_lock<shared_mutex> lock(_mutex); // Lock ensures that the root node is currently set (traversing unsets it temporarily)
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

void DataTree::traverseLeaves(uint32_t beginIndex, uint32_t endIndex, function<void (uint32_t index, LeafHandle leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf) {
  //TODO Can we allow multiple runs of traverseLeaves() in parallel? Also in parallel with resizeNumBytes()?
  std::unique_lock<shared_mutex> lock(_mutex);
  ASSERT(beginIndex <= endIndex, "Invalid parameters");

  auto onBacktrackFromSubtree = [] (DataInnerNode* /*node*/) {};

  _traverseLeaves(beginIndex, endIndex, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);

  if (_numLeavesCache != none && *_numLeavesCache < endIndex) {
    _numLeavesCache = endIndex;
  }
}

void DataTree::_traverseLeaves(uint32_t beginIndex, uint32_t endIndex,
    function<void (uint32_t index, LeafHandle leaf)> onExistingLeaf,
    function<Data (uint32_t index)> onCreateLeaf,
    function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
  _rootNode = LeafTraverser(_nodeStore).traverseAndReturnRoot(std::move(_rootNode), beginIndex, endIndex, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
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
  std::unique_lock<shared_mutex> lock(_mutex); // TODO Multiple ones in parallel? Also in parallel with traverseLeaves()?

  uint32_t newNumLeaves = std::max(UINT64_C(1), utils::ceilDivision(newNumBytes, _nodeStore->layout().maxBytesPerLeaf()));
  uint32_t newLastLeafSize = newNumBytes - (newNumLeaves-1) * _nodeStore->layout().maxBytesPerLeaf();
  uint32_t maxChildrenPerInnerNode = _nodeStore->layout().maxChildrenPerInnerNode();
  auto onExistingLeaf = [newLastLeafSize] (uint32_t /*index*/, LeafHandle leafHandle) {
      auto leaf = leafHandle.node();
      // This is only called, if the new last leaf was already existing
      if (leaf->numBytes() != newLastLeafSize) {
        leaf->resize(newLastLeafSize);
      }
  };
  auto onCreateLeaf = [newLastLeafSize] (uint32_t /*index*/) -> Data {
      // This is only called, if the new last leaf was not existing yet
      return Data(newLastLeafSize).FillWithZeroes();
  };
  auto onBacktrackFromSubtree = [this, newNumLeaves, maxChildrenPerInnerNode] (DataInnerNode* node) {
      // This is only called for the right border nodes of the new tree.
      // When growing size, the following is a no-op. When shrinking, we're deleting the children that aren't needed anymore.
      uint32_t maxLeavesPerChild = utils::intPow((uint64_t)maxChildrenPerInnerNode, ((uint64_t)node->depth()-1));
      uint32_t neededNodesOnChildLevel = utils::ceilDivision(newNumLeaves, maxLeavesPerChild);
      uint32_t neededSiblings = utils::ceilDivision(neededNodesOnChildLevel, maxChildrenPerInnerNode);
      uint32_t neededChildrenForRightBorderNode = neededNodesOnChildLevel - (neededSiblings-1) * maxChildrenPerInnerNode;
      ASSERT(neededChildrenForRightBorderNode <= node->numChildren(), "Node has too few children");
      // All children to the right of the new right-border-node are removed including their subtree.
      while(node->numChildren() > neededChildrenForRightBorderNode) {
        _nodeStore->removeSubtree(node->depth()-1, node->LastChild()->key());
        node->removeLastChild();
      }
  };

  _traverseLeaves(newNumLeaves - 1, newNumLeaves, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
  _numLeavesCache = newNumLeaves;
}

uint64_t DataTree::maxBytesPerLeaf() const {
  return _nodeStore->layout().maxBytesPerLeaf();
}

uint8_t DataTree::depth() const {
  return _rootNode->depth();
}

}
}
}
