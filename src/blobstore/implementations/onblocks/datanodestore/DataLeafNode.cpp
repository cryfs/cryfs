#include "DataLeafNode.h"
#include "DataInnerNode.h"
#include <cpp-utils/assert/assert.h>

using cpputils::Data;
using blockstore::BlockId;
using blockstore::BlockStore;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataLeafNode::DataLeafNode(DataNodeView view)
: DataNode(std::move(view)) {
  ASSERT(node().Depth() == 0, "Leaf node must have depth 0. Is it an inner node instead?");
  ASSERT(numBytes() <= maxStoreableBytes(), "Leaf says it stores more bytes than it has space for");
  if (node().FormatVersion() != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("This node format is not supported. Was it created with a newer version of CryFS?");
  }
}

DataLeafNode::~DataLeafNode() {
}

unique_ref<DataLeafNode> DataLeafNode::CreateNewNode(BlockStore *blockStore, const DataNodeLayout &layout, Data data) {
  ASSERT(data.size() <= layout.maxBytesPerLeaf(), "Data passed in is too large for one leaf.");
  const uint32_t size = data.size();
  return make_unique_ref<DataLeafNode>(DataNodeView::create(blockStore, layout, DataNode::FORMAT_VERSION_HEADER, 0, size, std::move(data)));
}

unique_ref<DataLeafNode> DataLeafNode::OverwriteNode(BlockStore *blockStore, const DataNodeLayout &layout, const BlockId &blockId, Data data) {
  ASSERT(data.size() == layout.maxBytesPerLeaf(), "Data passed in is too large for one leaf.");
  const uint32_t size = data.size();
  return make_unique_ref<DataLeafNode>(DataNodeView::overwrite(blockStore, layout, DataNode::FORMAT_VERSION_HEADER, 0, size, blockId, std::move(data)));
}

void DataLeafNode::read(void *target, uint64_t offset, uint64_t size) const {
  ASSERT(offset <= node().Size() && offset + size <= node().Size(), "Read out of valid area"); // Also check offset, because the addition could lead to overflows
  std::memcpy(target, static_cast<const uint8_t*>(node().data()) + offset, size);
}

void DataLeafNode::write(const void *source, uint64_t offset, uint64_t size) {
  ASSERT(offset <= node().Size() && offset + size <= node().Size(), "Write out of valid area"); // Also check offset, because the addition could lead to overflows
  node().write(source, offset, size);
}

uint32_t DataLeafNode::numBytes() const {
  return node().Size();
}

void DataLeafNode::resize(uint32_t new_size) {
  ASSERT(new_size <= maxStoreableBytes(), "Trying to resize to a size larger than the maximal size");
  const uint32_t old_size = node().Size();
  if (new_size < old_size) {
    fillDataWithZeroesFromTo(new_size, old_size);
  }
  node().setSize(new_size);
}

void DataLeafNode::fillDataWithZeroesFromTo(uint64_t begin, uint64_t end) {
  Data ZEROES(end-begin);
  ZEROES.FillWithZeroes();
  node().write(ZEROES.data(), begin, end-begin);
}

uint64_t DataLeafNode::maxStoreableBytes() const {
  return node().layout().maxBytesPerLeaf();
}

}
}
}
