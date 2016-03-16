#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/cast.h>
#include "EncryptedBlock.h"
#include <iostream>

namespace blockstore {
namespace encrypted {

template<class Cipher>
class EncryptedBlockStore final: public BlockStore {
public:
  EncryptedBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore, const typename Cipher::EncryptionKey &encKey);

  //TODO Are createKey() tests included in generic BlockStoreTest? If not, add it!
  Key createKey() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
  void remove(cpputils::unique_ref<Block> block) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;

  //This function should only be used by test cases
  void __setKey(const typename Cipher::EncryptionKey &encKey);

private:
  cpputils::unique_ref<BlockStore> _baseBlockStore;
  typename Cipher::EncryptionKey _encKey;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStore);
};



template<class Cipher>
EncryptedBlockStore<Cipher>::EncryptedBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore, const typename Cipher::EncryptionKey &encKey)
 : _baseBlockStore(std::move(baseBlockStore)), _encKey(encKey) {
}

template<class Cipher>
Key EncryptedBlockStore<Cipher>::createKey() {
  return _baseBlockStore->createKey();
}

template<class Cipher>
boost::optional<cpputils::unique_ref<Block>> EncryptedBlockStore<Cipher>::tryCreate(const Key &key, cpputils::Data data) {
  //TODO Test that this returns boost::none when base blockstore returns nullptr  (for all pass-through-blockstores)
  //TODO Easier implementation? This is only so complicated because of the cast EncryptedBlock -> Block
  auto result = EncryptedBlock<Cipher>::TryCreateNew(_baseBlockStore.get(), key, std::move(data), _encKey);
  if (result == boost::none) {
    return boost::none;
  }
  return cpputils::unique_ref<Block>(std::move(*result));
}

template<class Cipher>
boost::optional<cpputils::unique_ref<Block>> EncryptedBlockStore<Cipher>::load(const Key &key) {
  auto block = _baseBlockStore->load(key);
  if (block == boost::none) {
    //TODO Test this path (for all pass-through-blockstores)
    return boost::none;
  }
  return boost::optional<cpputils::unique_ref<Block>>(EncryptedBlock<Cipher>::TryDecrypt(std::move(*block), _encKey));
}

template<class Cipher>
void EncryptedBlockStore<Cipher>::remove(cpputils::unique_ref<Block> block) {
  auto encryptedBlock = cpputils::dynamic_pointer_move<EncryptedBlock<Cipher>>(block);
  ASSERT(encryptedBlock != boost::none, "Block is not an EncryptedBlock");
  auto baseBlock = (*encryptedBlock)->releaseBlock();
  return _baseBlockStore->remove(std::move(baseBlock));
}

template<class Cipher>
uint64_t EncryptedBlockStore<Cipher>::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

template<class Cipher>
uint64_t EncryptedBlockStore<Cipher>::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

template<class Cipher>
void EncryptedBlockStore<Cipher>::__setKey(const typename Cipher::EncryptionKey &encKey) {
  _encKey = encKey;
}

template<class Cipher>
uint64_t EncryptedBlockStore<Cipher>::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  return EncryptedBlock<Cipher>::blockSizeFromPhysicalBlockSize(_baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize));
}

}
}

#endif
