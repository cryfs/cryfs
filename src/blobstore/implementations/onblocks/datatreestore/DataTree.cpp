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

using fspp::dynamic_pointer_move;

namespace blobstore {
namespace onblocks {
namespace datatreestore {

DataTree::DataTree(DataNodeStore *nodeStore, unique_ptr<DataNode> rootNode)
  : _nodeStore(nodeStore), _rootNode(std::move(rootNode)) {
}

DataTree::~DataTree() {
}

void DataTree::addDataLeaf() {
  auto insertPosOrNull = LowestRightBorderNodeWithLessThanKChildrenOrNull();
  if (insertPosOrNull) {
    addDataLeafAt(insertPosOrNull.get());
  } else {
    addDataLeafToFullTree();
  }
}

unique_ptr<DataInnerNode> DataTree::LowestRightBorderNodeWithLessThanKChildrenOrNull() {
  const DataInnerNode *root_inner_node = dynamic_cast<const DataInnerNode*>(_rootNode.get());
  if (nullptr == root_inner_node) {
    //Root is not an inner node but a leaf
    return nullptr;
  }

  unique_ptr<DataNode> currentNode = _nodeStore->load(root_inner_node->LastChild()->key());
  unique_ptr<DataInnerNode> result(nullptr);
  while(auto currentInnerNode = dynamic_pointer_move<DataInnerNode>(currentNode)) {
    Key rightmostChildKey = currentInnerNode->LastChild()->key();
    if (currentInnerNode->numChildren() < DataInnerNode::MAX_STORED_CHILDREN) {
      result = std::move(currentInnerNode);
    }
    currentNode = _nodeStore->load(rightmostChildKey);
  }

  return result;
}

unique_ptr<DataLeafNode> DataTree::addDataLeafAt(DataInnerNode *insertPos) {
  auto new_leaf = _nodeStore->createNewLeafNode();
  if (insertPos->depth() == 1) {
    insertPos->addChild(*new_leaf);
  } else {
    auto chain = createChainOfInnerNodes(insertPos->depth()-1, *new_leaf);
    insertPos->addChild(*chain);
  }
  return new_leaf;
}

unique_ptr<DataInnerNode> DataTree::createChainOfInnerNodes(unsigned int num, const DataLeafNode &leaf) {
  assert(num > 0);
  unique_ptr<DataInnerNode> chain = _nodeStore->createNewInnerNode(leaf);
  for(unsigned int i=1; i<num; ++i) {
    chain = _nodeStore->createNewInnerNode(*chain);
  }
  return chain;
}

unique_ptr<DataLeafNode> DataTree::addDataLeafToFullTree() {
  //TODO
  //auto copyOfOldRoot = copyNode(*_rootNode);
  //_rootNode->InitializeNewInnerNode(*copyOfOldRoot);
  //addDataLeafAt(_rootNode.get());
  assert(false);
  return nullptr;
}

unique_ptr<DataNode> DataTree::copyNode(const DataNode &source) {
  //TODO
  assert(false);
  return nullptr;
}



}
}
}
