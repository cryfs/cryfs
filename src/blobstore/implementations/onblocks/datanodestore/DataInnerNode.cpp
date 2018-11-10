#include "DataInnerNode.h"
#include "DataNodeStore.h"
#include <cpp-utils/assert/assert.h>

using blockstore::Block;
using blockstore::BlockStore;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blockstore::BlockId;
using std::vector;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataInnerNode::DataInnerNode(DataNodeView view)
: DataNode(std::move(view)) {
  ASSERT(depth() > 0, "Inner node can't have depth 0. Is this a leaf maybe?");
  if (node().FormatVersion() != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("This node format (" + std::to_string(node().FormatVersion()) + ") is not supported. Was it created with a newer version of CryFS?");
  }
}

DataInnerNode::~DataInnerNode() {
}

unique_ref<DataInnerNode> DataInnerNode::InitializeNewNode(unique_ref<Block> block, const DataNodeLayout &layout, uint8_t depth, const vector<BlockId> &children) {
  ASSERT(children.size() >= 1, "An inner node must have at least one child");
  Data data = _serializeChildren(children);

  return make_unique_ref<DataInnerNode>(DataNodeView::initialize(std::move(block), layout, DataNode::FORMAT_VERSION_HEADER, depth, children.size(), std::move(data)));
}

unique_ref<DataInnerNode> DataInnerNode::CreateNewNode(BlockStore *blockStore, const DataNodeLayout &layout, uint8_t depth, const vector<BlockId> &children) {
  ASSERT(children.size() >= 1, "An inner node must have at least one child");
  Data data = _serializeChildren(children);

  return make_unique_ref<DataInnerNode>(DataNodeView::create(blockStore, layout, DataNode::FORMAT_VERSION_HEADER, depth, children.size(), std::move(data)));
}

Data DataInnerNode::_serializeChildren(const vector<BlockId> &children) {
  Data data(sizeof(ChildEntry) * children.size());
  uint32_t i = 0;
  for (const BlockId &child : children) {
    child.ToBinary(data.dataOffset(i * BlockId::BINARY_LENGTH));
    ++i;
  }
  return data;
}

uint32_t DataInnerNode::numChildren() const {
  return node().Size();
}

DataInnerNode::ChildEntry DataInnerNode::readChild(unsigned int index) const {
  ASSERT(index < numChildren(), "Accessing child out of range");
  return ChildEntry(BlockId::FromBinary(static_cast<const uint8_t*>(node().data()) + index * sizeof(ChildEntry)));
}

void DataInnerNode::_writeChild(unsigned int index, const ChildEntry& child) {
  ASSERT(index < numChildren(), "Accessing child out of range");
  node().write(child.blockId().data().data(), index * sizeof(ChildEntry), sizeof(ChildEntry));
}

DataInnerNode::ChildEntry DataInnerNode::readLastChild() const {
  return readChild(numChildren() - 1);
}

void DataInnerNode::_writeLastChild(const ChildEntry& child) {
  _writeChild(numChildren() - 1, child);
}

void DataInnerNode::addChild(const DataNode &child) {
  ASSERT(numChildren() < maxStoreableChildren(), "Adding more children than we can store");
  ASSERT(child.depth() == depth()-1, "The child that should be added has wrong depth");
  node().setSize(node().Size()+1);
  _writeLastChild(ChildEntry(child.blockId()));
}

void DataInnerNode::removeLastChild() {
  ASSERT(node().Size() > 1, "There is no child to remove");
  _writeLastChild(ChildEntry(BlockId::Null()));
  node().setSize(node().Size()-1);
}

uint32_t DataInnerNode::maxStoreableChildren() const {
  return node().layout().maxChildrenPerInnerNode();
}

}
}
}
