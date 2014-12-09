#include "DataInnerNode.h"

using std::unique_ptr;
using blockstore::Block;

namespace blobstore {
namespace onblocks {

DataInnerNode::DataInnerNode(unique_ptr<Block> block)
: DataNode(std::move(block)) {
}

DataInnerNode::~DataInnerNode() {
}

void DataInnerNode::InitializeEmptyInnerNode() {
  InnerNodeHeader* header = (InnerNodeHeader*)_block->data();
  header->nodeHeader.magicNumber = DataNode::magicNumberInnerNode;
}

}
}
