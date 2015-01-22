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

DataNode::DataNode(DataNodeView node, const Key &key)
: _key(key), _node(std::move(node)) {
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
  return _key;
}

uint8_t DataNode::depth() const {
  return *node().Depth();
}


}
}
}
