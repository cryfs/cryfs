#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_

#include "../../interface/Block.h"
#include "../../utils/Data.h"
#include "../../interface/BlockStore.h"

#include "messmer/cpp-utils/macros.h"
#include <memory>
#include <boost/optional.hpp>

namespace blockstore {
namespace encrypted {
template<class Cipher> class EncryptedBlockStore;

//TODO not only encryption, but also hmac

template<class Cipher>
class EncryptedBlock: public Block {
public:
  static std::unique_ptr<EncryptedBlock> TryCreateNew(BlockStore *baseBlockStore, const Key &key, Data data, const typename Cipher::EncryptionKey &encKey);
  static std::unique_ptr<EncryptedBlock> TryDecrypt(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &key);

  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  EncryptedBlock(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &key, Data plaintextData);
  virtual ~EncryptedBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;
  void flush() override;

  size_t size() const override;

  std::unique_ptr<Block> releaseBlock();

private:
  std::unique_ptr<Block> _baseBlock;
  Data _plaintextData;
  typename Cipher::EncryptionKey _encKey;
  bool _dataChanged;

  void _encryptToBaseBlock();

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlock);
};



template<class Cipher>
std::unique_ptr<EncryptedBlock<Cipher>> EncryptedBlock<Cipher>::TryCreateNew(BlockStore *baseBlockStore, const Key &key, Data data, const typename Cipher::EncryptionKey &encKey) {
  Data encrypted = Cipher::encrypt((byte*)data.data(), data.size(), encKey);
  auto baseBlock = baseBlockStore->tryCreate(key, std::move(encrypted));
  if (baseBlock.get() == nullptr) {
    //TODO Test this code branch
    return nullptr;
  }

  return std::make_unique<EncryptedBlock>(std::move(baseBlock), encKey, std::move(data));
}

template<class Cipher>
std::unique_ptr<EncryptedBlock<Cipher>> EncryptedBlock<Cipher>::TryDecrypt(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &encKey) {
  //TODO Change BlockStore so we can read their "class Data" objects instead of "void *data()", and then we can change the Cipher interface to take Data objects instead of "byte *" + size
  boost::optional<Data> plaintext = Cipher::decrypt((byte*)baseBlock->data(), baseBlock->size(), encKey);
  if(!plaintext) {
    return nullptr;
  }
  return std::make_unique<EncryptedBlock<Cipher>>(std::move(baseBlock), encKey, std::move(*plaintext));
}

template<class Cipher>
EncryptedBlock<Cipher>::EncryptedBlock(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &encKey, Data plaintextData)
    :Block(baseBlock->key()),
   _baseBlock(std::move(baseBlock)),
   _plaintextData(std::move(plaintextData)),
   _encKey(encKey),
   _dataChanged(false) {
}

template<class Cipher>
EncryptedBlock<Cipher>::~EncryptedBlock() {
  _encryptToBaseBlock();
}

template<class Cipher>
const void *EncryptedBlock<Cipher>::data() const {
  return _plaintextData.data();
}

template<class Cipher>
void EncryptedBlock<Cipher>::write(const void *source, uint64_t offset, uint64_t size) {
  assert(offset <= _plaintextData.size() && offset + size <= _plaintextData.size()); //Also check offset < _data->size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_plaintextData.data()+offset, source, size);
  _dataChanged = true;
}

template<class Cipher>
void EncryptedBlock<Cipher>::flush() {
  _encryptToBaseBlock();
  return _baseBlock->flush();
}

template<class Cipher>
size_t EncryptedBlock<Cipher>::size() const {
  return _plaintextData.size();
}

template<class Cipher>
void EncryptedBlock<Cipher>::_encryptToBaseBlock() {
  if (_dataChanged) {
    Data encrypted = Cipher::encrypt((byte*)_plaintextData.data(), _plaintextData.size(), _encKey);
    _baseBlock->write(encrypted.data(), 0, encrypted.size());
    _dataChanged = false;
  }
}

template<class Cipher>
std::unique_ptr<Block> EncryptedBlock<Cipher>::releaseBlock() {
  return std::move(_baseBlock);
}

}
}

#endif
