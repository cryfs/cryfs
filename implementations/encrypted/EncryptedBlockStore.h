#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <messmer/cpp-utils/macros.h>
#include <messmer/cpp-utils/pointer/cast.h>
#include "EncryptedBlock.h"
#include <iostream>

namespace blockstore {
namespace encrypted {

template<class Cipher>
class EncryptedBlockStore: public BlockStore {
public:
  EncryptedBlockStore(std::unique_ptr<BlockStore> baseBlockStore, const typename Cipher::EncryptionKey &encKey);

  //TODO Are createKey() tests included in generic BlockStoreTest? If not, add it!
  Key createKey() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  std::unique_ptr<Block> load(const Key &key) override;
  void remove(std::unique_ptr<Block> block) override;
  uint64_t numBlocks() const override;

  //This function should only be used by test cases
  void __setKey(const typename Cipher::EncryptionKey &encKey);

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
boost::optional<cpputils::unique_ref<Block>> EncryptedBlockStore<Cipher>::tryCreate(const Key &key, cpputils::Data data) {
  //TODO Test that this returns boost::none when base blockstore returns nullptr  (for all pass-through-blockstores)
  //TODO Easier implementation? This is only so complicated because of the case EncryptedBlock -> Block
  auto result = EncryptedBlock<Cipher>::TryCreateNew(_baseBlockStore.get(), key, std::move(data), _encKey);
  if (result == boost::none) {
    return boost::none;
  }
  return cpputils::unique_ref<Block>(std::move(*result));
}

template<class Cipher>
std::unique_ptr<Block> EncryptedBlockStore<Cipher>::load(const Key &key) {
  auto block = _baseBlockStore->load(key);
  if (block.get() == nullptr) {
    //TODO Test this path (for all pass-through-blockstores)
    return nullptr;
  }
  return EncryptedBlock<Cipher>::TryDecrypt(std::move(block), _encKey);
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

template<class Cipher>
void EncryptedBlockStore<Cipher>::__setKey(const typename Cipher::EncryptionKey &encKey) {
  _encKey = encKey;
}

}
}

#endif
