#include <blobstore/implementations/onblocks/BlobOnBlocks.h>

#include "datanodestore/DataNode.h"

using std::unique_ptr;

namespace blobstore {
namespace onblocks {

using datanodestore::DataNode;

BlobOnBlocks::BlobOnBlocks(unique_ptr<DataNode> rootnode)
: _rootnode(std::move(rootnode)) {

}

BlobOnBlocks::~BlobOnBlocks() {
}

size_t BlobOnBlocks::size() const {
  return _rootnode->numBytesInThisNode();
}

}
}
