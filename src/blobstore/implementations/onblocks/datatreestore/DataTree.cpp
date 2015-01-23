#include "DataTree.h"

#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"

#include "fspp/utils/pointer.h"

using blockstore::Key;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;

using std::unique_ptr;
using std::dynamic_pointer_cast;
using std::function;

using fspp::dynamic_pointer_move;
using fspp::ptr::optional_ownership_ptr;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTree::DataTree(DataNodeStore *nodeStore, unique_ptr<DataNode> rootNode)
  : _nodeStore(nodeStore), _rootNode(std::move(rootNode)) {
}

DataTree::~DataTree() {
}

unique_ptr<DataLeafNode> DataTree::addDataLeaf() {
  auto insertPosOrNull = lowestRightBorderNodeWithLessThanKChildrenOrNull();
  if (insertPosOrNull) {
    return addDataLeafAt(insertPosOrNull.get());
  } else {
    return addDataLeafToFullTree();
  }
}

optional_ownership_ptr<DataInnerNode> DataTree::lowestRightBorderNodeWithLessThanKChildrenOrNull() {
  optional_ownership_ptr<DataInnerNode> currentNode = fspp::ptr::WithoutOwnership(dynamic_cast<DataInnerNode*>(_rootNode.get()));
  optional_ownership_ptr<DataInnerNode> result = fspp::ptr::null<DataInnerNode>();
  for (unsigned int i=0; i < _rootNode->depth(); ++i) {
    auto lastChild = getLastChildAsInnerNode(*currentNode);
    if (currentNode->numChildren() < DataInnerNode::MAX_STORED_CHILDREN) {
      result = std::move(currentNode);
    }
    currentNode = std::move(lastChild);
  }

  return result;
}

unique_ptr<DataInnerNode> DataTree::getLastChildAsInnerNode(const DataInnerNode &node) {
  Key key = node.LastChild()->key();
  auto lastChild = _nodeStore->load(key);
  return dynamic_pointer_move<DataInnerNode>(lastChild);
}

unique_ptr<DataLeafNode> DataTree::addDataLeafAt(DataInnerNode *insertPos) {
  auto new_leaf = _nodeStore->createNewLeafNode();
  auto chain = createChainOfInnerNodes(insertPos->depth()-1, new_leaf.get());
  insertPos->addChild(*chain);
  return new_leaf;
}

optional_ownership_ptr<DataNode> DataTree::createChainOfInnerNodes(unsigned int num, DataLeafNode *leaf) {
  optional_ownership_ptr<DataNode> chain = fspp::ptr::WithoutOwnership<DataNode>(leaf);
  for(unsigned int i=0; i<num; ++i) {
    auto newnode = _nodeStore->createNewInnerNode(*chain);
    chain = fspp::ptr::WithOwnership<DataNode>(std::move(newnode));
  }
  return chain;
}

unique_ptr<DataLeafNode> DataTree::addDataLeafToFullTree() {
  auto copyOfOldRoot = copyNode(*_rootNode);
  auto newRootNode = DataNode::convertToNewInnerNode(std::move(_rootNode), *copyOfOldRoot);
  auto newLeaf = addDataLeafAt(newRootNode.get());
  _rootNode = std::move(newRootNode);
  return newLeaf;
}

unique_ptr<DataNode> DataTree::copyNode(const DataNode &source) {
  //TODO
  assert(false);
  return nullptr;
}



}
}
}
