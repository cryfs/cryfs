#include "DataInnerNode.h"
#include "DataNodeStore.h"
#include <cpp-utils/assert/assert.h>

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
  ASSERT(depth() > 0, "Inner node can't have depth 0. Is this a leaf maybe?");
  if (node().FormatVersion() != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("This node format is not supported. Was it created with a newer version of CryFS?");
  }
}

DataInnerNode::~DataInnerNode() {
}

unique_ref<DataInnerNode> DataInnerNode::InitializeNewNode(unique_ref<Block> block, const DataNode &first_child) {
  DataNodeView node(std::move(block));
  node.setFormatVersion(DataNode::FORMAT_VERSION_HEADER);
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
  ASSERT(index < numChildren(), "Accessing child out of range");
  return ChildrenBegin()+index;
}

void DataInnerNode::addChild(const DataNode &child) {
  ASSERT(numChildren() < maxStoreableChildren(), "Adding more children than we can store");
  ASSERT(child.depth() == depth()-1, "The child that should be added has wrong depth");
  node().setSize(node().Size()+1);
  LastChild()->setKey(child.key());
}

void DataInnerNode::removeLastChild() {
  ASSERT(node().Size() > 1, "There is no child to remove");
  node().setSize(node().Size()-1);
}

uint32_t DataInnerNode::maxStoreableChildren() const {
  return node().layout().maxChildrenPerInnerNode();
}

}
}
}
