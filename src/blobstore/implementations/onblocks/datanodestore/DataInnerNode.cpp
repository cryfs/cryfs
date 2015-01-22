#include "DataInnerNode.h"
#include "DataNodeStore.h"


using std::unique_ptr;
using blockstore::Block;
using blockstore::Data;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataInnerNode::DataInnerNode(DataNodeView view, const Key &key)
: DataNode(std::move(view), key) {
}

DataInnerNode::~DataInnerNode() {
}

void DataInnerNode::InitializeNewNode(const DataNode &first_child) {
  *node().Depth() = first_child.depth() + 1;
  *node().Size() = 1;
  first_child.key().ToBinary(ChildrenBegin()->key);
}

uint32_t DataInnerNode::numChildren() const {
  return *node().Size();
}

DataInnerNode::ChildEntry *DataInnerNode::ChildrenBegin() {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->ChildrenBegin());
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildrenBegin() const {
  return node().DataBegin<ChildEntry>();
}

DataInnerNode::ChildEntry *DataInnerNode::ChildrenEnd() {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->ChildrenEnd());
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildrenEnd() const {
  return ChildrenBegin() + *node().Size();
}

const DataInnerNode::ChildEntry *DataInnerNode::RightmostExistingChild() const{
  return ChildrenEnd()-1;
}

}
}
}
