#include "parallelaccessdatatreestore/DataTreeRef.h"
#include "BlobOnBlocks.h"

#include "datanodestore/DataLeafNode.h"
#include "datanodestore/DataNodeStore.h"
#include "utils/Math.h"
#include <cmath>
#include <cpp-utils/assert/assert.h>
#include "datatreestore/LeafHandle.h"

using cpputils::unique_ref;
using cpputils::Data;
using blockstore::BlockId;

namespace blobstore {
namespace onblocks {

using parallelaccessdatatreestore::DataTreeRef;

BlobOnBlocks::BlobOnBlocks(unique_ref<DataTreeRef> datatree)
: _datatree(std::move(datatree)) {
}

BlobOnBlocks::~BlobOnBlocks() {
} // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )

uint64_t BlobOnBlocks::size() const {
  return _datatree->numBytes();
}

void BlobOnBlocks::resize(uint64_t numBytes) {
  _datatree->resizeNumBytes(numBytes);
}

Data BlobOnBlocks::readAll() const {
  return _datatree->readAllBytes();
}

void BlobOnBlocks::read(void *target, uint64_t offset, uint64_t count) const {
  return _datatree->readBytes(target, offset, count);
}

uint64_t BlobOnBlocks::tryRead(void *target, uint64_t offset, uint64_t count) const {
  return _datatree->tryReadBytes(target, offset, count);
}

void BlobOnBlocks::write(const void *source, uint64_t offset, uint64_t count) {
  _datatree->writeBytes(source, offset, count);
}

void BlobOnBlocks::flush() {
  _datatree->flush();
}

uint32_t BlobOnBlocks::numNodes() const {
  return _datatree->numNodes();
}

const BlockId &BlobOnBlocks::blockId() const {
  return _datatree->blockId();
}

unique_ref<DataTreeRef> BlobOnBlocks::releaseTree() {
  return std::move(_datatree);
}

}
}
