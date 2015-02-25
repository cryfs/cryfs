#include "BlobOnBlocks.h"

#include "datatreestore/DataTree.h"
#include <cassert>

using std::unique_ptr;

namespace blobstore {
namespace onblocks {

using datatreestore::DataTree;

BlobOnBlocks::BlobOnBlocks(unique_ptr<DataTree> datatree)
: _datatree(std::move(datatree)) {

}

BlobOnBlocks::~BlobOnBlocks() {
}

size_t BlobOnBlocks::size() const {
  assert(false); //TODO Implement
  //return _rootnode->numBytesInThisNode();
}

void BlobOnBlocks::flush() const {
  _datatree->flush();
}

}
}
