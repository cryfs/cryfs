#include "DataNode.h"

#include "DataInnerNode.h"
#include "DataLeafNode.h"

using blockstore::Block;

using std::unique_ptr;
using std::make_unique;
using std::runtime_error;

namespace blobstore {
namespace onblocks {

DataNode::DataNode(DataNodeView node)
: _node(std::move(node)) {
}

DataNode::~DataNode() {
}

unique_ptr<DataNode> DataNode::load(unique_ptr<Block> block) {
  DataNodeView node(std::move(block));

  if (*node.MagicNumber() == node.magicNumberNodeWithChildren) {
    return make_unique<DataInnerNode>(std::move(node));
  } else if (*node.MagicNumber() == node.magicNumberLeaf) {
    return make_unique<DataLeafNode>(std::move(node));
  } else {
    //TODO Better exception
    throw runtime_error("Invalid node magic number");
  }
}

/*
unique_ptr<DataInnerNode> DataNodeView::initializeNewInnerNode(unique_ptr<Block> block) {
  auto newNode = make_unique<DataInnerNode>(std::move(block));
  newNode->InitializeEmptyInnerNode();
  return newNode;
}

unique_ptr<DataLeafNode> DataNodeView::initializeNewLeafNode(unique_ptr<Block> block) {
  auto newNode = make_unique<DataLeafNode>(std::move(block));
  newNode->InitializeEmptyLeaf();
  return newNode;
}*/

}
}
