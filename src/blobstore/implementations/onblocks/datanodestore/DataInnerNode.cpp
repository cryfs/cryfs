#include "DataInnerNode.h"
#include "DataNodeStore.h"


using std::unique_ptr;
using blockstore::Block;
using blockstore::Data;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataInnerNode::DataInnerNode(DataNodeView view)
: DataNode(std::move(view)) {
}

DataInnerNode::~DataInnerNode() {
}

void DataInnerNode::InitializeNewNode(const DataNode &first_child) {
  *node().Depth() = first_child.depth() + 1;
  *node().Size() = 1;
  ChildrenBegin()->setKey(first_child.key());
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

DataInnerNode::ChildEntry *DataInnerNode::LastChild() {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->LastChild());
}

const DataInnerNode::ChildEntry *DataInnerNode::LastChild() const {
  return ChildrenEnd()-1;
}

DataInnerNode::ChildEntry *DataInnerNode::getChild(unsigned int index) {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->getChild(index));
}

const DataInnerNode::ChildEntry *DataInnerNode::getChild(unsigned int index) const {
  assert(index < numChildren());
  return ChildrenBegin()+index;
}

void DataInnerNode::addChild(const DataNode &child) {
  assert(numChildren() < DataInnerNode::MAX_STORED_CHILDREN);
  assert(child.depth() == depth()-1);
  *node().Size() += 1;
  LastChild()->setKey(child.key());
}

}
}
}
