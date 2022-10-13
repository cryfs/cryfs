#include "DataInnerNode.h"
#include "DataLeafNode.h"
#include "DataNode.h"
#include "DataNodeStore.h"
#include <blockstore/utils/BlockStoreUtils.h>

using blockstore::BlockId;

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

const BlockId &DataNode::blockId() const {
  return _node.blockId();
}

uint8_t DataNode::depth() const {
  return _node.Depth();
}

unique_ref<DataInnerNode> DataNode::convertToNewInnerNode(unique_ref<DataNode> node, const DataNodeLayout &layout, const DataNode &first_child) {
  auto block = node->_node.releaseBlock();
  blockstore::utils::fillWithZeroes(block.get());

  return DataInnerNode::InitializeNewNode(std::move(block), layout, first_child.depth()+1, {first_child.blockId()});
}

}
}
}
