#include "EncryptedBlock.h"

namespace blockstore {
namespace encrypted {

EncryptedBlock::EncryptedBlock(std::unique_ptr<Block> baseBlock, const EncryptionKey &encKey)
  :Block(baseBlock->key()), _baseBlock(std::move(baseBlock)), _encKey(encKey) {
}

const void *EncryptedBlock::data() const {
  return _baseBlock->data();
}

void EncryptedBlock::write(const void *source, uint64_t offset, uint64_t size) {
  return _baseBlock->write(source, offset, size);
}

void EncryptedBlock::flush() {
  return _baseBlock->flush();
}

size_t EncryptedBlock::size() const {
  return _baseBlock->size();
}

}
}
