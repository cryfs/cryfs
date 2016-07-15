#include "DataInnerNode.h"
#include "DataNodeStore.h"
#include <cpp-utils/assert/assert.h>

using blockstore::Block;
using blockstore::BlockStore;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blockstore::Key;
using std::vector;

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

unique_ref<DataInnerNode> DataInnerNode::InitializeNewNode(unique_ref<Block> block, const DataNodeLayout &layout, uint8_t depth, const vector<Key> &children) {
  ASSERT(children.size() >= 1, "An inner node must have at least one child");
  Data data = _serializeChildren(children);

  return make_unique_ref<DataInnerNode>(DataNodeView::initialize(std::move(block), layout, DataNode::FORMAT_VERSION_HEADER, depth, children.size(), std::move(data)));
}

unique_ref<DataInnerNode> DataInnerNode::CreateNewNode(BlockStore *blockStore, const DataNodeLayout &layout, uint8_t depth, const vector<Key> &children) {
  ASSERT(children.size() >= 1, "An inner node must have at least one child");
  Data data = _serializeChildren(children);

  return make_unique_ref<DataInnerNode>(DataNodeView::create(blockStore, layout, DataNode::FORMAT_VERSION_HEADER, depth, children.size(), std::move(data)));
}

Data DataInnerNode::_serializeChildren(const vector<Key> &children) {
  Data data(sizeof(ChildEntry) * children.size());
  uint32_t i = 0;
  for (const Key &child : children) {
    reinterpret_cast<ChildEntry*>(data.data())[i++].setKey(child);
  }
  return data;
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
  LastChild()->setKey(Key::Null());
  node().setSize(node().Size()-1);
}

uint32_t DataInnerNode::maxStoreableChildren() const {
  return node().layout().maxChildrenPerInnerNode();
}

}
}
}
