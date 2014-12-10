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
    return unique_ptr<DataInnerNode>(new DataInnerNode(std::move(node)));
  } else if (*node.MagicNumber() == node.magicNumberLeaf) {
    return unique_ptr<DataLeafNode>(new DataLeafNode(std::move(node)));
  } else {
    //TODO Better exception
    throw runtime_error("Invalid node magic number");
  }
}

unique_ptr<DataNode> DataNode::createNewInnerNode(unique_ptr<Block> block) {
  auto newNode = unique_ptr<DataInnerNode>(new DataInnerNode(std::move(block)));
  newNode->InitializeNewNode();
  return std::move(newNode);
}

unique_ptr<DataNode> DataNode::createNewLeafNode(unique_ptr<Block> block) {
  auto newNode = unique_ptr<DataLeafNode>(new DataLeafNode(std::move(block)));
  newNode->InitializeNewNode();
  return std::move(newNode);
}

}
}
