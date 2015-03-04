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

using cpputils::dynamic_pointer_move;
using cpputils::optional_ownership_ptr;
using cpputils::WithOwnership;
using cpputils::WithoutOwnership;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTree::DataTree(DataNodeStore *nodeStore, unique_ptr<DataNode> rootNode)
  : _nodeStore(nodeStore), _rootNode(std::move(rootNode)) {
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

optional_ownership_ptr<DataNode> DataTree::createChainOfInnerNodes(unsigned int num, DataLeafNode *leaf) {
  optional_ownership_ptr<DataNode> chain = cpputils::WithoutOwnership<DataNode>(leaf);
  for(unsigned int i=0; i<num; ++i) {
    auto newnode = _nodeStore->createNewInnerNode(*chain);
    chain = cpputils::WithOwnership<DataNode>(std::move(newnode));
  }
  return chain;
}

unique_ptr<DataLeafNode> DataTree::addDataLeafToFullTree() {
  auto copyOfOldRoot = _nodeStore->createNewNodeAsCopyFrom(*_rootNode);
  auto newRootNode = DataNode::convertToNewInnerNode(std::move(_rootNode), *copyOfOldRoot);
  auto newLeaf = addDataLeafAt(newRootNode.get());
  _rootNode = std::move(newRootNode);
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

void DataTree::traverseLeaves(uint32_t beginIndex, uint32_t endIndex, function<void (DataLeafNode*, uint32_t)> func) {
  assert(beginIndex <= endIndex);
  //TODO assert(beginIndex <= numLeaves());
  //TODO assert(endIndex <= numLeaves());
  traverseLeaves(_rootNode.get(), 0, beginIndex, endIndex, func);
}

void DataTree::traverseLeaves(DataNode *root, uint32_t leafOffset, uint32_t beginIndex, uint32_t endIndex, function<void (DataLeafNode*, uint32_t)> func) {
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

  for (uint32_t childIndex = beginChild; childIndex < endChild; ++childIndex) {
    uint32_t childOffset = childIndex * leavesPerChild;
    uint32_t localBeginIndex = utils::maxZeroSubtraction(beginIndex, childOffset);
    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
    auto child = _nodeStore->load(inner->getChild(childIndex)->key());
    traverseLeaves(child.get(), leafOffset + childOffset, localBeginIndex, localEndIndex, func);
  }
}

uint32_t DataTree::leavesPerFullChild(const DataInnerNode &root) const {
  return utils::intPow(_nodeStore->layout().maxChildrenPerInnerNode(), root.depth()-1);
}

uint64_t DataTree::numStoredBytes() const {
  return numStoredBytes(*_rootNode);
}

uint64_t DataTree::numStoredBytes(const DataNode &root) const {
  const DataLeafNode *leaf = dynamic_cast<const DataLeafNode*>(&root);
  if (leaf != nullptr) {
    return leaf->numBytes();
  }

  const DataInnerNode &inner = dynamic_cast<const DataInnerNode&>(root);
  uint64_t numBytesInLeftChildren = (inner.numChildren()-1) * leavesPerFullChild(inner) * _nodeStore->layout().maxBytesPerLeaf();
  auto lastChild = _nodeStore->load(inner.LastChild()->key());
  uint64_t numBytesInRightChild = numStoredBytes(*lastChild);

  return numBytesInLeftChildren + numBytesInRightChild;
}

void DataTree::resizeNumBytes(uint64_t newNumBytes) {
  //TODO Faster implementation possible
  LastLeaf(_rootNode.get())->resize(_nodeStore->layout().maxBytesPerLeaf());
  uint64_t currentNumBytes = numStoredBytes();
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
