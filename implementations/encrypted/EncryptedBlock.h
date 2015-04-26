#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_

#include "../../interface/Block.h"
#include <messmer/cpp-utils/data/Data.h>
#include "../../interface/BlockStore.h"

#include "messmer/cpp-utils/macros.h"
#include <memory>
#include <boost/optional.hpp>
#include "ciphers/Cipher.h"

namespace blockstore {
namespace encrypted {
template<class Cipher> class EncryptedBlockStore;

template<class Cipher>
class EncryptedBlock: public Block {
public:
  BOOST_CONCEPT_ASSERT((CipherConcept<Cipher>));
  static std::unique_ptr<EncryptedBlock> TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, const typename Cipher::EncryptionKey &encKey);
  static std::unique_ptr<EncryptedBlock> TryDecrypt(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &key);

  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  EncryptedBlock(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &key, cpputils::Data plaintextWithHeader);
  virtual ~EncryptedBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t count) override;
  void flush() override;

  size_t size() const override;

  std::unique_ptr<Block> releaseBlock();

private:
  std::unique_ptr<Block> _baseBlock;
  cpputils::Data _plaintextWithHeader;
  typename Cipher::EncryptionKey _encKey;
  bool _dataChanged;

  static constexpr unsigned int HEADER_LENGTH = Key::BINARY_LENGTH;

  void _encryptToBaseBlock();
  static cpputils::Data _prependKeyHeaderToData(const Key &key, cpputils::Data data);
  static bool _keyHeaderIsCorrect(const Key &key, const cpputils::Data &data);

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlock);
};

template<class Cipher>
constexpr unsigned int EncryptedBlock<Cipher>::HEADER_LENGTH;


template<class Cipher>
std::unique_ptr<EncryptedBlock<Cipher>> EncryptedBlock<Cipher>::TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, const typename Cipher::EncryptionKey &encKey) {
  cpputils::Data plaintextWithHeader = _prependKeyHeaderToData(key, std::move(data));
  cpputils::Data encrypted = Cipher::encrypt((byte*)plaintextWithHeader.data(), plaintextWithHeader.size(), encKey);
  auto baseBlock = baseBlockStore->tryCreate(key, std::move(encrypted));
  if (baseBlock.get() == nullptr) {
    //TODO Test this code branch
    return nullptr;
  }

  return std::make_unique<EncryptedBlock>(std::move(baseBlock), encKey, std::move(plaintextWithHeader));
}

template<class Cipher>
std::unique_ptr<EncryptedBlock<Cipher>> EncryptedBlock<Cipher>::TryDecrypt(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &encKey) {
  //TODO Change BlockStore so we can read their "class Data" objects instead of "void *data()", and then we can change the Cipher interface to take Data objects instead of "byte *" + size
  boost::optional<cpputils::Data> plaintextWithHeader = Cipher::decrypt((byte*)baseBlock->data(), baseBlock->size(), encKey);
  if(!plaintextWithHeader) {
    //Decryption failed (e.g. an authenticated cipher detected modifications to the ciphertext)
    //TODO Think about logging
    std::cerr << "Decrypting block " << baseBlock->key() << " failed. Was the block modified by an attacker?" << std::endl;
    return nullptr;
  }
  if(!_keyHeaderIsCorrect(baseBlock->key(), *plaintextWithHeader)) {
    //The stored key in the block data is incorrect - an attacker might have exchanged the contents with the encrypted data from a different block
    //TODO Think about logging
    std::cerr << "Decrypting block " << baseBlock->key() << " failed due to invalid block key. Was the block modified by an attacker?" << std::endl;
    return nullptr;
  }
  return std::make_unique<EncryptedBlock<Cipher>>(std::move(baseBlock), encKey, std::move(*plaintextWithHeader));
}

template<class Cipher>
cpputils::Data EncryptedBlock<Cipher>::_prependKeyHeaderToData(const Key &key, cpputils::Data data) {
  static_assert(HEADER_LENGTH >= Key::BINARY_LENGTH, "Key doesn't fit into the header");
  cpputils::Data result(data.size() + HEADER_LENGTH);
  std::memcpy(result.data(), key.data(), Key::BINARY_LENGTH);
  std::memcpy((uint8_t*)result.data() + Key::BINARY_LENGTH, data.data(), data.size());
  return result;
}

template<class Cipher>
bool EncryptedBlock<Cipher>::_keyHeaderIsCorrect(const Key &key, const cpputils::Data &data) {
  return 0 == std::memcmp(key.data(), data.data(), Key::BINARY_LENGTH);
}

template<class Cipher>
EncryptedBlock<Cipher>::EncryptedBlock(std::unique_ptr<Block> baseBlock, const typename Cipher::EncryptionKey &encKey, cpputils::Data plaintextWithHeader)
    :Block(baseBlock->key()),
   _baseBlock(std::move(baseBlock)),
   _plaintextWithHeader(std::move(plaintextWithHeader)),
   _encKey(encKey),
   _dataChanged(false) {
}

template<class Cipher>
EncryptedBlock<Cipher>::~EncryptedBlock() {
  _encryptToBaseBlock();
}

template<class Cipher>
const void *EncryptedBlock<Cipher>::data() const {
  return (uint8_t*)_plaintextWithHeader.data() + HEADER_LENGTH;
}

template<class Cipher>
void EncryptedBlock<Cipher>::write(const void *source, uint64_t offset, uint64_t count) {
  assert(offset <= size() && offset + count <= size()); //Also check offset < size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_plaintextWithHeader.data()+HEADER_LENGTH+offset, source, count);
  _dataChanged = true;
}

template<class Cipher>
void EncryptedBlock<Cipher>::flush() {
  _encryptToBaseBlock();
  return _baseBlock->flush();
}

template<class Cipher>
size_t EncryptedBlock<Cipher>::size() const {
  return _plaintextWithHeader.size() - HEADER_LENGTH;
}

template<class Cipher>
void EncryptedBlock<Cipher>::_encryptToBaseBlock() {
  if (_dataChanged) {
    cpputils::Data encrypted = Cipher::encrypt((byte*)_plaintextWithHeader.data(), _plaintextWithHeader.size(), _encKey);
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
