#include "DataNode.h"

#include "DataInnerNode.h"
#include "DataLeafNode.h"

using blockstore::Block;

using std::unique_ptr;
using std::make_unique;
using std::runtime_error;

namespace blobstore {
namespace onblocks {

DataNode::DataNode(unique_ptr<Block> block)
: _block(std::move(block)) {
}

DataNode::~DataNode() {
}

void DataNode::flush() {
  _block->flush();
}

unique_ptr<DataNode> DataNode::load(unique_ptr<Block> block) {
  NodeHeader *header = (NodeHeader*)block->data();
  if (header->magicNumber == magicNumberInnerNode) {
    return make_unique<DataInnerNode>(std::move(block));
  } else if (header->magicNumber == magicNumberLeaf) {
    return make_unique<DataLeafNode>(std::move(block));
  } else {
    //TODO Better exception
    throw runtime_error("Invalid node magic number");
  }
}

unique_ptr<DataInnerNode> DataNode::initializeNewInnerNode(unique_ptr<Block> block) {
  auto newNode = make_unique<DataInnerNode>(std::move(block));
  newNode->InitializeEmptyInnerNode();
  return newNode;
}

unique_ptr<DataLeafNode> DataNode::initializeNewLeafNode(unique_ptr<Block> block) {
  auto newNode = make_unique<DataLeafNode>(std::move(block));
  newNode->InitializeEmptyLeaf();
  return newNode;
}

}
}
