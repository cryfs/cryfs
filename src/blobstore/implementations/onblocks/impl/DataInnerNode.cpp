#include "DataInnerNode.h"

using std::unique_ptr;
using blockstore::Block;
using blockstore::Data;

namespace blobstore {
namespace onblocks {

DataInnerNode::DataInnerNode(DataNodeView view)
: DataNode(std::move(view)) {
}

DataInnerNode::~DataInnerNode() {
}

void DataInnerNode::InitializeNewNode(const Key &first_child_key, const DataNodeView &first_child) {
  *_node.Depth() = *first_child.Depth() + 1;
  *_node.Size() = 1;
  first_child_key.ToBinary(ChildrenBegin()->key);
}

void DataInnerNode::read(off_t offset, size_t count, Data *result) const {
  assert(count <= result->size());
  const uint64_t end = offset + count;
  assert(end <= numBytesInThisNode());

  uint8_t *target = (uint8_t*)result->data();

  const ChildEntry *child = ChildContainingFirstByteAfterOffset(offset);
  off_t blockrelative_offset = offset - numBytesInLeftwardSiblings(child);
  uint64_t already_read_bytes = readFromChild(child, blockrelative_offset, count, target);
  while(numBytesInChildAndLeftwardSiblings(child) < end) {
    ++child;
    already_read_bytes += readFromChild(child, 0, count, target + already_read_bytes);
  };
  assert(already_read_bytes == count);
}

uint64_t DataInnerNode::readFromChild(const ChildEntry *child, off_t inner_offset, size_t count, uint8_t *target) const {
  uint64_t readable_bytes = std::min(count, numBytesInChild(child) - inner_offset);

  //TODO READ...

  return readable_bytes;
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildContainingFirstByteAfterOffset(off_t offset) const {
  uint32_t offset_blocks = offset / _node.BLOCKSIZE_BYTES;

  //TODO no binary search anymore
  return
    std::upper_bound(ChildrenBegin(), ChildrenEnd(), offset_blocks, [](uint32_t offset_blocks, const ChildEntry &child) {
      return false;//return offset_blocks < child.numBlocksInThisAndLeftwardNodes;
    });
}

uint64_t DataInnerNode::numBytesInThisNode() const {
  return numBytesInChildAndLeftwardSiblings(ChildrenLast());
}

uint64_t DataInnerNode::numBytesInChild(const ChildEntry *child) const {
  return numBytesInChildAndLeftwardSiblings(child) - numBytesInLeftwardSiblings(child);
}

uint64_t DataInnerNode::numBytesInLeftwardSiblings(const ChildEntry *child) const {
  if (child == ChildrenBegin()) {
    return 0;
  }
  return numBytesInChildAndLeftwardSiblings(child-1);
}

uint64_t DataInnerNode::numBytesInChildAndLeftwardSiblings(const ChildEntry *child) const {
  //TODO Rewrite
  //return (uint64_t)child->numBlocksInThisAndLeftwardNodes * _node.BLOCKSIZE_BYTES;
}

DataInnerNode::ChildEntry *DataInnerNode::ChildrenBegin() {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->ChildrenBegin());
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildrenBegin() const {
  return _node.DataBegin<ChildEntry>();
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildrenEnd() const {
  return ChildrenBegin() + *_node.Size();
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildrenLast() const{
  return ChildrenEnd()-1;
}

void DataInnerNode::write(off_t offset, size_t count, const Data &data) {
  //TODO Implement

}

void DataInnerNode::resize(uint64_t newsize_bytes) {
  //TODO Implement
}

}
}
