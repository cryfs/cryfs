#include <blobstore/implementations/onblocks/datanodestore/DataInnerNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>


using std::unique_ptr;
using blockstore::Block;
using blockstore::Data;

namespace blobstore {
namespace onblocks {

DataInnerNode::DataInnerNode(DataNodeView view, const Key &key, DataNodeStore *nodestorage)
: DataNode(std::move(view), key, nodestorage) {
}

DataInnerNode::~DataInnerNode() {
}

void DataInnerNode::InitializeNewNode(const DataNode &first_child) {
  *node().Depth() = first_child.depth() + 1;
  *node().Size() = 1;
  first_child.key().ToBinary(ChildrenBegin()->key);
}

void DataInnerNode::read(off_t offset, size_t count, Data *result) const {
  assert(count <= result->size());
  const uint64_t end = offset + count;
  assert(end <= numBytesInThisNode());

  uint8_t *target = (uint8_t*)result->data();

  const ChildEntry *child = ChildContainingFirstByteAfterOffset(offset);
  uint32_t child_index = child-ChildrenBegin();
  uint64_t child_first_byte_index = maxNumBytesPerChild() * child_index;
  uint64_t next_child_first_byte_index = child_first_byte_index + maxNumBytesPerChild();
  off_t childrelative_offset = offset - child_first_byte_index;
  uint64_t already_read_bytes = readFromChild(child, childrelative_offset, count, target);
  while(next_child_first_byte_index < end) { //TODO Write a test case that breaks when we're having <= instead of < here
    ++child;
    already_read_bytes += readFromChild(child, 0, count, target + already_read_bytes);
  };
  assert(already_read_bytes == count);
}

uint64_t DataInnerNode::readFromChild(const ChildEntry *child, off_t inner_offset, size_t count, uint8_t *target) const {
  //TODO This only works for non-rightmost children
  uint64_t readable_bytes = std::min(count, maxNumBytesPerChild() - inner_offset);

  //TODO READ...

  return readable_bytes;
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildContainingFirstByteAfterOffset(off_t offset) const {
  uint8_t child_index = offset/maxNumBytesPerChild();
  return ChildrenBegin()+child_index;
}

uint32_t DataInnerNode::maxNumDataBlocksPerChild() const {
  return std::round(std::pow(MAX_STORED_CHILDREN, *node().Depth()));
}

uint64_t DataInnerNode::numBytesInThisNode() const {
  return numBytesInNonRightmostChildrenSum() + numBytesInRightmostChild();
}

uint64_t DataInnerNode::numBytesInNonRightmostChildrenSum() const {
  return maxNumBytesPerChild() * (numChildren()-1);
}

uint64_t DataInnerNode::numBytesInRightmostChild() const {
  Key rightmost_child_key = Key::FromBinary(RightmostChild()->key);
  auto rightmost_child = storage().load(rightmost_child_key);
  return rightmost_child->numBytesInThisNode();
}

uint32_t DataInnerNode::numChildren() const {
  return *node().Size();
}

//TODO This only works for non-rightmost children
uint64_t DataInnerNode::maxNumBytesPerChild() const {
  return maxNumDataBlocksPerChild() * DataNodeView::DATASIZE_BYTES;
}
DataInnerNode::ChildEntry *DataInnerNode::ChildrenBegin() {
  return const_cast<ChildEntry*>(const_cast<const DataInnerNode*>(this)->ChildrenBegin());
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildrenBegin() const {
  return node().DataBegin<ChildEntry>();
}

const DataInnerNode::ChildEntry *DataInnerNode::ChildrenEnd() const {
  return ChildrenBegin() + *node().Size();
}

const DataInnerNode::ChildEntry *DataInnerNode::RightmostChild() const{
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
