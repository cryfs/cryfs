#include "DataLeafNode.h"

using std::unique_ptr;
using blockstore::Block;
using blockstore::Data;
using blockstore::Key;

namespace blobstore {
namespace onblocks {
namespace datanodestore {

DataLeafNode::DataLeafNode(DataNodeView view, const Key &key)
: DataNode(std::move(view), key) {
  assert(numBytes() <= MAX_STORED_BYTES);
}

DataLeafNode::~DataLeafNode() {
}

void DataLeafNode::InitializeNewNode() {
  *node().Depth() = 0;
  *node().Size() = 0;
  //fillDataWithZeroes(); not needed, because a newly created block will be zeroed out. DataLeafNodeTest.SpaceIsZeroFilledWhenGrowing ensures this.
}

void *DataLeafNode::data() {
  return const_cast<void*>(const_cast<const DataLeafNode*>(this)->data());
}

const void *DataLeafNode::data() const {
  return node().DataBegin<uint8_t>();
}

uint32_t DataLeafNode::numBytes() const {
  return *node().Size();
}

void DataLeafNode::resize(uint32_t new_size) {
  assert(new_size <= MAX_STORED_BYTES);
  uint32_t old_size = *node().Size();
  if (new_size < old_size) {
    fillDataWithZeroesFromTo(new_size, old_size);
  }
  *node().Size() = new_size;
}

void DataLeafNode::fillDataWithZeroesFromTo(off_t begin, off_t end) {
  std::memset(node().DataBegin<unsigned char>()+begin, 0, end-begin);
}

}
}
}
