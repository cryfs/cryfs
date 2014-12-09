#include "DataLeafNode.h"

using std::unique_ptr;
using blockstore::Block;

namespace blobstore {
namespace onblocks {

DataLeafNode::DataLeafNode(unique_ptr<Block> block)
: DataNode(std::move(block)) {
}

DataLeafNode::~DataLeafNode() {
}

void DataLeafNode::InitializeEmptyLeaf() {
  LeafHeader *header = (LeafHeader*)_block->data();
  header->nodeHeader.magicNumber = DataNode::magicNumberLeaf;
}

}
}
