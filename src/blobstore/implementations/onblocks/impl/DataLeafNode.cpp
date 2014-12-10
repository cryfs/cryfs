#include "DataLeafNode.h"

using std::unique_ptr;
using blockstore::Block;
using blockstore::Data;

namespace blobstore {
namespace onblocks {

DataLeafNode::DataLeafNode(DataNodeView view)
: DataNode(std::move(view)) {
  assert(numBytesInThisNode() <= MAX_STORED_BYTES);
}

DataLeafNode::~DataLeafNode() {
}

void DataLeafNode::read(off_t offset, size_t count, Data *result) {
  assert(count <= result->size());
  assert(offset+count <= numBytesInThisNode());
  std::memcpy(result->data(), _node.DataBegin<unsigned char>()+offset, count);
}

void DataLeafNode::write(off_t offset, size_t count, const Data &data) {
  assert(count <= data.size());
  assert(offset+count <= numBytesInThisNode());
  std::memcpy(_node.DataBegin<unsigned char>()+offset, data.data(), count);
}

void DataLeafNode::InitializeNewNode() {
  *_node.MagicNumber() = _node.magicNumberLeaf;
  *_node.Size() = 0;
  //fillDataWithZeroes(); not needed, because a newly created block will be zeroed out. DataLeafNodeTest.SpaceIsZeroFilledWhenGrowing ensures this.
}

void DataLeafNode::fillDataWithZeroesFromTo(off_t begin, off_t end) {
  std::memset(_node.DataBegin<unsigned char>()+begin, 0, end-begin);
}

uint64_t DataLeafNode::numBytesInThisNode() {
  return *_node.Size();
}

void DataLeafNode::resize(uint64_t newsize_bytes) {
  assert(newsize_bytes <= MAX_STORED_BYTES);

  if (newsize_bytes < *_node.Size()) {
    fillDataWithZeroesFromTo(newsize_bytes, *_node.Size());
  }

  *_node.Size() = newsize_bytes;
}

}
}
