#include <blobstore/implementations/onblocks/impl/DataNodeStore.h>
#include "DataNode.h"

#include "DataInnerNode.h"
#include "DataLeafNode.h"

using blockstore::Block;

using std::unique_ptr;
using std::make_unique;
using std::runtime_error;

namespace blobstore {
namespace onblocks {

DataNode::DataNode(DataNodeView node, const Key &key, DataNodeStore *nodestorage)
: _key(key), _node(std::move(node)), _nodestorage(nodestorage) {
}

DataNode::~DataNode() {
}

DataNodeStore &DataNode::storage() {
  return const_cast<DataNodeStore&>(const_cast<const DataNode*>(this)->storage());
}

const DataNodeStore &DataNode::storage() const {
  return *_nodestorage;
}

DataNodeView &DataNode::node() {
  return const_cast<DataNodeView&>(const_cast<const DataNode*>(this)->node());
}

const DataNodeView &DataNode::node() const {
  return _node;
}

const Key &DataNode::key() const {
  return _key;
}

uint8_t DataNode::depth() const {
  return *_node.Depth();
}

}
}
