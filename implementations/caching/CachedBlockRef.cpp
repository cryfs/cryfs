#include <messmer/blockstore/implementations/caching/CachedBlockRef.h>
#include <messmer/blockstore/implementations/caching/CachingBlockStore.h>

using std::shared_ptr;
using std::make_unique;
using std::function;

namespace blockstore {
namespace caching {

CachedBlockRef::CachedBlockRef(Block *baseBlock)
  :Block(baseBlock->key()),
   _baseBlock(baseBlock) {
}

CachedBlockRef::~CachedBlockRef() {
  _baseBlock->flush();
}

const void *CachedBlockRef::data() const {
  return _baseBlock->data();
}

void CachedBlockRef::write(const void *source, uint64_t offset, uint64_t size) {
  return _baseBlock->write(source, offset, size);
}

size_t CachedBlockRef::size() const {
  return _baseBlock->size();
}

void CachedBlockRef::flush() {
  return _baseBlock->flush();
}

}
}
