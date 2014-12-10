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

void DataInnerNode::InitializeNewInnerNode() {
  *_node.MagicNumber() = _node.magicNumberNodeWithChildren;
  *_node.Size() = 0;
}

void DataInnerNode::read(off_t offset, size_t count, Data *result) {
  assert(count <= result->size());
  const uint64_t end = offset + count;
  assert(end <= numBytesInThisNode());

  uint8_t *target = (uint8_t*)result->data();

  ChildEntry *child = ChildContainingFirstByteAfterOffset(offset);
  off_t blockrelative_offset = offset - numBytesInLeftwardSiblings(child);
  uint64_t already_read_bytes = readFromChild(child, blockrelative_offset, count, target);
  while(numBytesInChildAndLeftwardSiblings(child) < end) {
    ++child;
    already_read_bytes += readFromChild(child, 0, count, target + already_read_bytes);
  };
  assert(already_read_bytes == count);
}

uint64_t DataInnerNode::readFromChild(const ChildEntry *child, off_t inner_offset, size_t count, uint8_t *target) {
  uint64_t readable_bytes = std::min(count, numBytesInChild(child) - inner_offset);

  //TODO READ...

  return readable_bytes;
}

DataInnerNode::ChildEntry *DataInnerNode::ChildContainingFirstByteAfterOffset(off_t offset) {
  uint32_t offset_blocks = offset / _node.BLOCKSIZE_BYTES;

  return
    std::upper_bound(ChildrenBegin(), ChildrenEnd(), offset_blocks, [](uint32_t offset_blocks, const ChildEntry &child) {
      return offset_blocks < child.numBlocksInThisAndLeftwardNodes;
    });
}

uint64_t DataInnerNode::numBytesInThisNode() {
  return numBytesInChildAndLeftwardSiblings(ChildrenLast());
}

uint64_t DataInnerNode::numBytesInChild(const ChildEntry *child) {
  return numBytesInChildAndLeftwardSiblings(child) - numBytesInLeftwardSiblings(child);
}

uint64_t DataInnerNode::numBytesInLeftwardSiblings(const ChildEntry *child) {
  if (child == ChildrenBegin()) {
    return 0;
  }
  return numBytesInChildAndLeftwardSiblings(child-1);
}

uint64_t DataInnerNode::numBytesInChildAndLeftwardSiblings(const ChildEntry *child) {
  return (uint64_t)child->numBlocksInThisAndLeftwardNodes * _node.BLOCKSIZE_BYTES;
}

DataInnerNode::ChildEntry *DataInnerNode::ChildrenBegin() {
  return _node.DataBegin<ChildEntry>();
}

DataInnerNode::ChildEntry *DataInnerNode::ChildrenEnd() {
  return ChildrenBegin() + *_node.Size();
}

DataInnerNode::ChildEntry *DataInnerNode::ChildrenLast() {
  return ChildrenEnd()-1;
}

void DataInnerNode::write(off_t offset, size_t count, const Data &data) {
  //assert(count <= data.size());
  //assert(offset+count <= _node->DATASIZE_BYTES);
  //std::memcpy(_node->DataBegin()+offset, result.data(), count);

}

}
}
