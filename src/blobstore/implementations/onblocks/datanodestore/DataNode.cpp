#include "DataInnerNode.h"
#include "DataLeafNode.h"
#include "DataNode.h"
#include "DataNodeStore.h"
#include <blockstore/utils/BlockStoreUtils.h>

using blockstore::Block;
using blockstore::Key;

using std::runtime_error;
using cpputils::unique_ref;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

constexpr uint16_t DataNode::FORMAT_VERSION_HEADER;

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
  return _node.Depth();
}

unique_ref<DataInnerNode> DataNode::convertToNewInnerNode(unique_ref<DataNode> node, const DataNode &first_child) {
  Key key = node->key();
  auto block = node->_node.releaseBlock();
  blockstore::utils::fillWithZeroes(block.get());

  return DataInnerNode::InitializeNewNode(std::move(block), first_child);
}

void DataNode::flush() const {
  _node.flush();
}

}
}
}
