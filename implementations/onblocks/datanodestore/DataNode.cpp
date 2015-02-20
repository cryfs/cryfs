#include "DataInnerNode.h"
#include "DataLeafNode.h"
#include "DataNode.h"
#include "DataNodeStore.h"

using blockstore::Block;
using blockstore::Key;

using std::unique_ptr;
using std::make_unique;
using std::runtime_error;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataNode::DataNode(DataNodeView node)
: _node(std::move(node)) {
}

DataNode::~DataNode() {
}

DataNodeView &DataNode::node() {
  return const_cast<DataNodeView&>(const_cast<const DataNode*>(this)->node());
}

const DataNodeView &DataNode::node() const {
  return _node;
}

const Key &DataNode::key() const {
  return _node.key();
}

uint8_t DataNode::depth() const {
  return *node().Depth();
}

unique_ptr<DataInnerNode> DataNode::convertToNewInnerNode(unique_ptr<DataNode> node, const DataNode &first_child) {
  Key key = node->key();
  auto block = node->_node.releaseBlock();
  std::memset(block->data(), 0, block->size());

  return DataInnerNode::InitializeNewNode(std::move(block), first_child);
}

void DataNode::flush() const {
  _node.flush();
}

}
}
}
