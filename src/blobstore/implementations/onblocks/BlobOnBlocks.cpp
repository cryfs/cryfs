#include <blobstore/implementations/onblocks/BlobOnBlocks.h>

#include <blobstore/implementations/onblocks/datanodestore/DataNode.h>

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
  assert(false); //TODO Implement
  //return _rootnode->numBytesInThisNode();
}

void BlobOnBlocks::flush() const {
  _rootnode->flush();
}

}
}
