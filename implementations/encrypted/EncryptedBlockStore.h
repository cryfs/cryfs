#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <messmer/cpp-utils/macros.h>
#include <messmer/cpp-utils/pointer.h>
#include "EncryptedBlock.h"

namespace blockstore {
namespace encrypted {

template<class Cipher>
class EncryptedBlockStore: public BlockStore {
public:
  EncryptedBlockStore(std::unique_ptr<BlockStore> baseBlockStore, const typename Cipher::EncryptionKey &encKey);

  Key createKey() override;
  std::unique_ptr<Block> tryCreate(const Key &key, Data data) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

private:
  std::unique_ptr<BlockStore> _baseBlockStore;
  typename Cipher::EncryptionKey _encKey;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStore);
};



template<class Cipher>
EncryptedBlockStore<Cipher>::EncryptedBlockStore(std::unique_ptr<BlockStore> baseBlockStore, const typename Cipher::EncryptionKey &encKey)
 : _baseBlockStore(std::move(baseBlockStore)), _encKey(encKey) {
}

template<class Cipher>
Key EncryptedBlockStore<Cipher>::createKey() {
  return _baseBlockStore->createKey();
}

template<class Cipher>
std::unique_ptr<Block> EncryptedBlockStore<Cipher>::tryCreate(const Key &key, Data data) {
  return EncryptedBlock<Cipher>::TryCreateNew(_baseBlockStore.get(), key, std::move(data), _encKey);
}

template<class Cipher>
std::unique_ptr<Block> EncryptedBlockStore<Cipher>::load(const Key &key) {
  auto block = _baseBlockStore->load(key);
  if (block.get() == nullptr) {
    return nullptr;
  }
  return std::make_unique<EncryptedBlock<Cipher>>(std::move(block), _encKey);
}

template<class Cipher>
void EncryptedBlockStore<Cipher>::remove(std::unique_ptr<Block> block) {
  auto baseBlock = cpputils::dynamic_pointer_move<EncryptedBlock<Cipher>>(block)->releaseBlock();
  return _baseBlockStore->remove(std::move(baseBlock));
}

template<class Cipher>
uint64_t EncryptedBlockStore<Cipher>::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

}
}

#endif
