#include "DataInnerNode.h"
#include "DataNodeStore.h"

using std::unique_ptr;
using blockstore::Block;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataInnerNode::DataInnerNode(DataNodeView view)
: DataNode(std::move(view)) {
  assert(depth() > 0);
}

DataInnerNode::~DataInnerNode() {
}

unique_ref<DataInnerNode> DataInnerNode::InitializeNewNode(unique_ptr<Block> block, const DataNode &first_child) {
  DataNodeView node(std::move(block));
  node.setDepth(first_child.depth() + 1);
  node.setSize(1);
  auto result = make_unique_ref<DataInnerNode>(std::move(node));
  result->ChildrenBegin()->setKey(first_child.key());
  return result;
}

uint32_t DataInnerNode::numChildren() const {
  return node().Size();
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
  return ChildrenBegin() + node().Size();
}

DataInnerNode::ChildEntry *DataInnerNode::LastChild() {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->LastChild());
}

const DataInnerNode::ChildEntry *DataInnerNode::LastChild() const {
  return getChild(numChildren()-1);
}

DataInnerNode::ChildEntry *DataInnerNode::getChild(unsigned int index) {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->getChild(index));
}

const DataInnerNode::ChildEntry *DataInnerNode::getChild(unsigned int index) const {
  assert(index < numChildren());
  return ChildrenBegin()+index;
}

void DataInnerNode::addChild(const DataNode &child) {
  assert(numChildren() < maxStoreableChildren());
  assert(child.depth() == depth()-1);
  node().setSize(node().Size()+1);
  LastChild()->setKey(child.key());
}

void DataInnerNode::removeLastChild() {
  assert(node().Size() > 1);
  node().setSize(node().Size()-1);
}

uint32_t DataInnerNode::maxStoreableChildren() const {
  return node().layout().maxChildrenPerInnerNode();
}

}
}
}
