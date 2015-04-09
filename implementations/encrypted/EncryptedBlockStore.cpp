#include "EncryptedBlockStore.h"
#include "EncryptedBlock.h"
#include <messmer/cpp-utils/pointer.h>

using std::unique_ptr;
using std::make_unique;

namespace blockstore {
namespace encrypted {

EncryptedBlockStore::EncryptedBlockStore(unique_ptr<BlockStore> baseBlockStore, const EncryptionKey &encKey)
 : _baseBlockStore(std::move(baseBlockStore)), _encKey(encKey) {
}

unique_ptr<Block> EncryptedBlockStore::create(size_t size) {
  return make_unique<EncryptedBlock>(_baseBlockStore->create(size), _encKey);
}

unique_ptr<Block> EncryptedBlockStore::load(const Key &key) {
  auto block = _baseBlockStore->load(key);
  if (block.get() == nullptr) {
    return nullptr;
  }
  return make_unique<EncryptedBlock>(std::move(block), _encKey);
}

void EncryptedBlockStore::remove(unique_ptr<Block> block) {
  return _baseBlockStore->remove(std::move(block));
}

uint64_t EncryptedBlockStore::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}
