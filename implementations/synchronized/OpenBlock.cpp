#include <messmer/blockstore/implementations/synchronized/OpenBlock.h>
#include "SynchronizedBlockStore.h"

using std::unique_ptr;
using std::make_unique;
using std::function;

namespace blockstore {
namespace synchronized {

OpenBlock::OpenBlock(unique_ptr<Block> baseBlock, OpenBlockList *openBlockList)
  //TODO We store key twice here - once in OpenBlock, once in the underlying baseBlock.
  //     Should we move that to make OpenBlock::key() call _baseBlock.key()?
  :Block(baseBlock->key()),
   _baseBlock(std::move(baseBlock)),
   _openBlockList(openBlockList) {
}

OpenBlock::~OpenBlock() {
  _openBlockList->release(std::move(_baseBlock));
}

const void *OpenBlock::data() const {
  return _baseBlock->data();
}

void OpenBlock::write(const void *source, uint64_t offset, uint64_t size) {
  return _baseBlock->write(source, offset, size);
}

size_t OpenBlock::size() const {
  return _baseBlock->size();
}

void OpenBlock::flush() {
  return _baseBlock->flush();
}

}
}
