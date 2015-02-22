#include "DataInnerNode.h"
#include "DataNodeStore.h"

using std::unique_ptr;
using std::make_unique;
using blockstore::Block;
using blockstore::Data;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

constexpr uint32_t DataInnerNode::MAX_STORED_CHILDREN;

DataInnerNode::DataInnerNode(DataNodeView view)
: DataNode(std::move(view)) {
  assert(depth() > 0);
}

DataInnerNode::~DataInnerNode() {
}

unique_ptr<DataInnerNode> DataInnerNode::InitializeNewNode(unique_ptr<Block> block, const DataNode &first_child) {
  DataNodeView node(std::move(block));
  *node.Depth() = first_child.depth() + 1;
  *node.Size() = 1;
  auto result = make_unique<DataInnerNode>(std::move(node));
  result->ChildrenBegin()->setKey(first_child.key());
  return result;
}

uint8_t DataInnerNode::depth() const {
  return *node().Depth();
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

void DataInnerNode::removeLastChild() {
  assert(*node().Size() > 1);
  *node().Size() -= 1;
}

}
}
}
