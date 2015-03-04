#include "DataLeafNode.h"
#include "DataInnerNode.h"

using std::unique_ptr;
using std::make_unique;
using blockstore::Block;
using blockstore::Data;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataLeafNode::DataLeafNode(DataNodeView view)
: DataNode(std::move(view)) {
  assert(node().Depth() == 0);
  assert(numBytes() <= maxStoreableBytes());
}

DataLeafNode::~DataLeafNode() {
}

unique_ptr<DataLeafNode> DataLeafNode::InitializeNewNode(unique_ptr<Block> block) {
  DataNodeView node(std::move(block));
  node.setDepth(0);
  node.setSize(0);
  //fillDataWithZeroes(); not needed, because a newly created block will be zeroed out. DataLeafNodeTest.SpaceIsZeroFilledWhenGrowing ensures this.
  return make_unique<DataLeafNode>(std::move(node));
}

void DataLeafNode::read(void *target, uint64_t offset, uint64_t size) const {
  assert(offset <= node().Size() && offset + size <= node().Size()); // Also check offset, because the addition could lead to overflows
  std::memcpy(target, (uint8_t*)node().data() + offset, size);
}

void DataLeafNode::write(const void *source, uint64_t offset, uint64_t size) {
  assert(offset <= node().Size() && offset + size <= node().Size()); // Also check offset, because the addition could lead to overflows
  node().write(source, offset, size);
}

uint32_t DataLeafNode::numBytes() const {
  return node().Size();
}

void DataLeafNode::resize(uint32_t new_size) {
  assert(new_size <= maxStoreableBytes());
  uint32_t old_size = node().Size();
  if (new_size < old_size) {
    fillDataWithZeroesFromTo(new_size, old_size);
  }
  node().setSize(new_size);
}

void DataLeafNode::fillDataWithZeroesFromTo(off_t begin, off_t end) {
  Data ZEROES(end-begin);
  ZEROES.FillWithZeroes();
  node().write(ZEROES.data(), begin, end-begin);
}

uint32_t DataLeafNode::maxStoreableBytes() const {
  return node().layout().maxBytesPerLeaf();
}

}
}
}
