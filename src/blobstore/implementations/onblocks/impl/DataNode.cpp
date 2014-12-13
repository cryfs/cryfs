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

  if (*node.Depth() == 0) {
    return unique_ptr<DataLeafNode>(new DataLeafNode(std::move(node)));
  } else if (*node.Depth() < MAX_DEPTH) {
    return unique_ptr<DataInnerNode>(new DataInnerNode(std::move(node)));
  } else {
    throw runtime_error("Tree is to deep. Data corruption?");
  }
}

unique_ptr<DataNode> DataNode::createNewInnerNode(unique_ptr<Block> block, const Key &first_child_key, const DataNode &first_child) {
  auto newNode = unique_ptr<DataInnerNode>(new DataInnerNode(std::move(block)));
  newNode->InitializeNewNode(first_child_key, first_child._node);
  return std::move(newNode);
}

unique_ptr<DataNode> DataNode::createNewLeafNode(unique_ptr<Block> block) {
  auto newNode = unique_ptr<DataLeafNode>(new DataLeafNode(std::move(block)));
  newNode->InitializeNewNode();
  return std::move(newNode);
}

}
}
