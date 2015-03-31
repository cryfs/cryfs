#include "CachedBlockRef.h"
#include "SynchronizedBlockStore.h"

using std::shared_ptr;
using std::make_unique;
using std::function;

namespace blockstore {
namespace synchronized {

CachedBlockRef::CachedBlockRef(Block *baseBlock, SynchronizedBlockStore *blockStore)
  //TODO We store key twice here - once in OpenBlock, once in the underlying baseBlock.
  //     Should we move that to make CachedBlockRef::key() call _baseBlock.key()?
  :Block(baseBlock->key()),
   _baseBlock(baseBlock),
   _blockStore(blockStore) {
}

CachedBlockRef::~CachedBlockRef() {
  _baseBlock->flush();
  _blockStore->release(_baseBlock);
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
