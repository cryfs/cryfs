#include "DataTree.h"

#include "../datanodestore/DataNodeStore.h"
#include "../datanodestore/DataInnerNode.h"
#include "../datanodestore/DataLeafNode.h"
#include "../utils/Math.h"

#include "impl/algorithms.h"

#include "messmer/cpp-utils/pointer.h"
#include "messmer/cpp-utils/optional_ownership_ptr.h"
#include <cmath>

using blockstore::Key;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeLayout;

using std::unique_ptr;
using std::dynamic_pointer_cast;
using std::function;
using boost::shared_mutex;
using boost::shared_lock;
using boost::unique_lock;
using std::vector;

using cpputils::dynamic_pointer_move;
using cpputils::optional_ownership_ptr;
using cpputils::WithOwnership;
using cpputils::WithoutOwnership;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTree::DataTree(DataNodeStore *nodeStore, unique_ptr<DataNode> rootNode)
  : _mutex(), _nodeStore(nodeStore), _rootNode(std::move(rootNode)) {
}

DataTree::~DataTree() {
}

void DataTree::removeLastDataLeaf() {
  auto deletePosOrNull = algorithms::GetLowestRightBorderNodeWithMoreThanOneChildOrNull(_nodeStore, _rootNode.get());
  assert(deletePosOrNull.get() != nullptr); //TODO Correct exception (tree has only one leaf, can't shrink it)

  deleteLastChildSubtree(deletePosOrNull.get());

  ifRootHasOnlyOneChildReplaceRootWithItsChild();
}

void DataTree::ifRootHasOnlyOneChildReplaceRootWithItsChild() {
  DataInnerNode *rootNode = dynamic_cast<DataInnerNode*>(_rootNode.get());
  assert(rootNode != nullptr);
  if (rootNode->numChildren() == 1) {
    auto child = _nodeStore->load(rootNode->getChild(0)->key());
    _rootNode = _nodeStore->overwriteNodeWith(std::move(_rootNode), *child);
    _nodeStore->remove(std::move(child));
  }
}

void DataTree::deleteLastChildSubtree(DataInnerNode *node) {
  auto lastChild = _nodeStore->load(node->LastChild()->key());
  _nodeStore->removeSubtree(std::move(lastChild));
  node->removeLastChild();
}

unique_ptr<DataLeafNode> DataTree::addDataLeaf() {
  auto insertPosOrNull = algorithms::GetLowestInnerRightBorderNodeWithLessThanKChildrenOrNull(_nodeStore, _rootNode.get());
  if (insertPosOrNull) {
    return addDataLeafAt(insertPosOrNull.get());
  } else {
    return addDataLeafToFullTree();
  }
}

unique_ptr<DataLeafNode> DataTree::addDataLeafAt(DataInnerNode *insertPos) {
  auto new_leaf = _nodeStore->createNewLeafNode();
  auto chain = createChainOfInnerNodes(insertPos->depth()-1, new_leaf.get());
  insertPos->addChild(*chain);
  return new_leaf;
}

optional_ownership_ptr<DataNode> DataTree::createChainOfInnerNodes(unsigned int num, DataNode *child) {
  //TODO This function is implemented twice, once with optional_ownership_ptr, once with unique_ptr. Redundancy!
  optional_ownership_ptr<DataNode> chain = cpputils::WithoutOwnership<DataNode>(child);
  for(unsigned int i=0; i<num; ++i) {
    auto newnode = _nodeStore->createNewInnerNode(*chain);
    chain = cpputils::WithOwnership<DataNode>(std::move(newnode));
  }
  return chain;
}

unique_ptr<DataNode> DataTree::createChainOfInnerNodes(unsigned int num, unique_ptr<DataNode> child) {
  unique_ptr<DataNode> chain = std::move(child);
  for(unsigned int i=0; i<num; ++i) {
    chain = _nodeStore->createNewInnerNode(*chain);
  }
  return chain;
}

DataInnerNode* DataTree::increaseTreeDepth(unsigned int levels) {
  assert(levels >= 1);
  auto copyOfOldRoot = _nodeStore->createNewNodeAsCopyFrom(*_rootNode);
  auto chain = createChainOfInnerNodes(levels-1, copyOfOldRoot.get());
  auto newRootNode = DataNode::convertToNewInnerNode(std::move(_rootNode), *chain);
  DataInnerNode *result = newRootNode.get();
  _rootNode = std::move(newRootNode);
  return result;
}

unique_ptr<DataLeafNode> DataTree::addDataLeafToFullTree() {
  DataInnerNode *rootNode = increaseTreeDepth(1);
  auto newLeaf = addDataLeafAt(rootNode);
  return newLeaf;
}

const Key &DataTree::key() const {
  return _rootNode->key();
}

void DataTree::flush() const {
  _rootNode->flush();
}

unique_ptr<DataNode> DataTree::releaseRootNode() {
  return std::move(_rootNode);
}

//TODO Test numLeaves(), for example also two configurations with same number of bytes but different number of leaves (last leaf has 0 bytes)
uint32_t DataTree::numLeaves() const {
  return _numLeaves(*_rootNode);
}

uint32_t DataTree::_numLeaves(const DataNode &node) const {
  const DataLeafNode *leaf = dynamic_cast<const DataLeafNode*>(&node);
  if (leaf != nullptr) {
    return 1;
  }

  const DataInnerNode &inner = dynamic_cast<const DataInnerNode&>(node);
  uint64_t numLeavesInLeftChildren = (inner.numChildren()-1) * leavesPerFullChild(inner);
  auto lastChild = _nodeStore->load(inner.LastChild()->key());
  uint64_t numLeavesInRightChild = _numLeaves(*lastChild);

  return numLeavesInLeftChildren + numLeavesInRightChild;
}

void DataTree::traverseLeaves(uint32_t beginIndex, uint32_t endIndex, function<void (DataLeafNode*, uint32_t)> func) {
  unique_lock<shared_mutex> lock(_mutex); //TODO Only lock when resizing
  assert(beginIndex <= endIndex);

  uint8_t neededTreeDepth = utils::ceilLog(_nodeStore->layout().maxChildrenPerInnerNode(), endIndex);
  uint32_t numLeaves = this->numLeaves();
  if (_rootNode->depth() < neededTreeDepth) {
    //TODO Test cases that actually increase it here by 0 level / 1 level / more than 1 level
    increaseTreeDepth(neededTreeDepth - _rootNode->depth());
  }

  if (numLeaves <= beginIndex) {
    //TODO Test cases with numLeaves < / >= beginIndex
    // There is a gap between the current size and the begin of the traversal
    return _traverseLeaves(_rootNode.get(), 0, numLeaves-1, endIndex, [beginIndex, numLeaves, &func, this](DataLeafNode* node, uint32_t index) {
      if (index >= beginIndex) {
        func(node, index);
      } else if (index == numLeaves - 1) {
        // It is the old last leaf - resize it to maximum
        node->resize(_nodeStore->layout().maxBytesPerLeaf());
      }
    });
  } else if (numLeaves < endIndex) {
    // We are starting traversal in the valid region, but traverse until after it (we grow new leaves)
    return _traverseLeaves(_rootNode.get(), 0, beginIndex, endIndex, [numLeaves, &func, this] (DataLeafNode *node, uint32_t index) {
      if (index == numLeaves - 1) {
        // It is the old last leaf  - resize it to maximum
        node->resize(_nodeStore->layout().maxBytesPerLeaf());
      }
      func(node, index);
    });
  } else {
    //We are traversing entierly inside the valid region
    _traverseLeaves(_rootNode.get(), 0, beginIndex, endIndex, func);
  }
}

void DataTree::_traverseLeaves(DataNode *root, uint32_t leafOffset, uint32_t beginIndex, uint32_t endIndex, function<void (DataLeafNode*, uint32_t)> func) {
  DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root);
  if (leaf != nullptr) {
    assert(beginIndex <= 1 && endIndex <= 1);
    if (beginIndex == 0 && endIndex == 1) {
      func(leaf, leafOffset);
    }
    return;
  }

  DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root);
  uint32_t leavesPerChild = leavesPerFullChild(*inner);
  uint32_t beginChild = beginIndex/leavesPerChild;
  uint32_t endChild = utils::ceilDivision(endIndex, leavesPerChild);
  vector<unique_ptr<DataNode>> children = getOrCreateChildren(inner, beginChild, endChild);

  for (uint32_t childIndex = beginChild; childIndex < endChild; ++childIndex) {
    uint32_t childOffset = childIndex * leavesPerChild;
    uint32_t localBeginIndex = utils::maxZeroSubtraction(beginIndex, childOffset);
    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
    auto child = std::move(children[childIndex-beginChild]);
    _traverseLeaves(child.get(), leafOffset + childOffset, localBeginIndex, localEndIndex, func);
  }
}

vector<unique_ptr<DataNode>> DataTree::getOrCreateChildren(DataInnerNode *node, uint32_t begin, uint32_t end) {
  vector<unique_ptr<DataNode>> children;
  children.reserve(end-begin);
  for (uint32_t childIndex = begin; childIndex < std::min(node->numChildren(), end); ++childIndex) {
    children.emplace_back(_nodeStore->load(node->getChild(childIndex)->key()));
  }
  for (uint32_t childIndex = node->numChildren(); childIndex < end; ++childIndex) {
    children.emplace_back(addChildTo(node));
  }
  assert(children.size() == end-begin);
  return children;
}

unique_ptr<DataNode> DataTree::addChildTo(DataInnerNode *node) {
  auto new_leaf = _nodeStore->createNewLeafNode();
  new_leaf->resize(_nodeStore->layout().maxBytesPerLeaf());
  auto chain = createChainOfInnerNodes(node->depth()-1, std::move(new_leaf));
  node->addChild(*chain);
  return std::move(chain);
}

uint32_t DataTree::leavesPerFullChild(const DataInnerNode &root) const {
  return utils::intPow(_nodeStore->layout().maxChildrenPerInnerNode(), root.depth()-1);
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
  uint64_t numBytesInRightChild = _numStoredBytes(*lastChild);

  return numBytesInLeftChildren + numBytesInRightChild;
}

void DataTree::resizeNumBytes(uint64_t newNumBytes) {
  boost::upgrade_lock<shared_mutex> lock(_mutex);
  {
    boost::upgrade_to_unique_lock<shared_mutex> exclusiveLock(lock);
    //TODO Faster implementation possible (no addDataLeaf()/removeLastDataLeaf() in a loop, but directly resizing)
    LastLeaf(_rootNode.get())->resize(_nodeStore->layout().maxBytesPerLeaf());
    uint64_t currentNumBytes = _numStoredBytes();
    assert(currentNumBytes % _nodeStore->layout().maxBytesPerLeaf() == 0);
    uint32_t currentNumLeaves = currentNumBytes / _nodeStore->layout().maxBytesPerLeaf();
    uint32_t newNumLeaves = std::max(1u, utils::ceilDivision(newNumBytes, _nodeStore->layout().maxBytesPerLeaf()));

    for(uint32_t i = currentNumLeaves; i < newNumLeaves; ++i) {
      addDataLeaf()->resize(_nodeStore->layout().maxBytesPerLeaf());
    }
    for(uint32_t i = currentNumLeaves; i > newNumLeaves; --i) {
      removeLastDataLeaf();
    }
    uint32_t newLastLeafSize = newNumBytes - (newNumLeaves-1)*_nodeStore->layout().maxBytesPerLeaf();
    LastLeaf(_rootNode.get())->resize(newLastLeafSize);
  }
  assert(newNumBytes == numStoredBytes());
}

optional_ownership_ptr<DataLeafNode> DataTree::LastLeaf(DataNode *root) {
  DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root);
  if (leaf != nullptr) {
    return WithoutOwnership(leaf);
  }

  DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root);
  return WithOwnership(LastLeaf(_nodeStore->load(inner->LastChild()->key())));
}

unique_ptr<DataLeafNode> DataTree::LastLeaf(unique_ptr<DataNode> root) {
  auto leaf = dynamic_pointer_move<DataLeafNode>(root);
  if (leaf.get() != nullptr) {
    return leaf;
  }
  auto inner = dynamic_pointer_move<DataInnerNode>(root);
  return LastLeaf(_nodeStore->load(inner->LastChild()->key()));
}

uint32_t DataTree::maxBytesPerLeaf() const {
  return _nodeStore->layout().maxBytesPerLeaf();
}

}
}
}
