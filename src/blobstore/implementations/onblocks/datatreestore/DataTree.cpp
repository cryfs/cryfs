#include "DataTree.h"

#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datanodestore/DataInnerNode.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"

#include "impl/GetLowestRightBorderNodeWithLessThanKChildrenOrNull.h"

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
  auto insertPosOrNull = impl::GetLowestRightBorderNodeWithLessThanKChildrenOrNull::run(_nodeStore, _rootNode.get());
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
  optional_ownership_ptr<DataNode> chain = fspp::ptr::WithoutOwnership<DataNode>(leaf);
  for(unsigned int i=0; i<num; ++i) {
    auto newnode = _nodeStore->createNewInnerNode(*chain);
    chain = fspp::ptr::WithOwnership<DataNode>(std::move(newnode));
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



}
}
}
