#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>
#include "../../interface/BlockStore.h"

#include <cpp-utils/macros.h>
#include <memory>
#include <iostream>
#include <boost/optional.hpp>
#include <cpp-utils/crypto/symmetric/Cipher.h>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/data/DataUtils.h>
#include <mutex>
#include <cpp-utils/logging/logging.h>

namespace blockstore {
namespace encrypted {
template<class Cipher> class EncryptedBlockStore;

//TODO Test EncryptedBlock

//TODO Fix mutexes & locks (basically true for all blockstores)

template<class Cipher>
class EncryptedBlock final: public Block {
public:
  BOOST_CONCEPT_ASSERT((cpputils::CipherConcept<Cipher>));
  static boost::optional<cpputils::unique_ref<EncryptedBlock>> TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, const typename Cipher::EncryptionKey &encKey);
  static boost::optional<cpputils::unique_ref<EncryptedBlock>> TryDecrypt(cpputils::unique_ref<Block> baseBlock, const typename Cipher::EncryptionKey &key);

  static uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize);

  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  EncryptedBlock(cpputils::unique_ref<Block> baseBlock, const typename Cipher::EncryptionKey &key, cpputils::Data plaintextWithHeader);
  ~EncryptedBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t count) override;
  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

  cpputils::unique_ref<Block> releaseBlock();

private:
  cpputils::unique_ref<Block> _baseBlock; // TODO Do I need the ciphertext block in memory or is the key enough?
  cpputils::Data _plaintextWithHeader;
  typename Cipher::EncryptionKey _encKey;
  bool _dataChanged;

  static constexpr unsigned int HEADER_LENGTH = Key::BINARY_LENGTH;

  void _encryptToBaseBlock();
  static cpputils::Data _prependKeyHeaderToData(const Key &key, cpputils::Data data);
  static bool _keyHeaderIsCorrect(const Key &key, const cpputils::Data &data);
  static cpputils::Data _prependFormatHeader(const cpputils::Data &data);
  static void _checkFormatHeader(const void *data);

  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;

  std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlock);
};

template<class Cipher>
constexpr unsigned int EncryptedBlock<Cipher>::HEADER_LENGTH;

template<class Cipher>
constexpr uint16_t EncryptedBlock<Cipher>::FORMAT_VERSION_HEADER;


template<class Cipher>
boost::optional<cpputils::unique_ref<EncryptedBlock<Cipher>>> EncryptedBlock<Cipher>::TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, const typename Cipher::EncryptionKey &encKey) {
  //TODO Is it possible to avoid copying the whole plaintext data into plaintextWithHeader? Maybe an encrypt() object that has an .addData() function and concatenates all data for encryption? Maybe Crypto++ offers this functionality already.
  cpputils::Data plaintextWithHeader = _prependKeyHeaderToData(key, std::move(data));
  cpputils::Data encrypted = Cipher::encrypt((byte*)plaintextWithHeader.data(), plaintextWithHeader.size(), encKey);
  //TODO Avoid copying the whole encrypted block into a encryptedWithFormatHeader by creating a Data object with full size and then giving it as an encryption target to Cipher::encrypt()
  cpputils::Data encryptedWithFormatHeader = _prependFormatHeader(std::move(encrypted));
  auto baseBlock = baseBlockStore->tryCreate(key, std::move(encryptedWithFormatHeader));
  if (baseBlock == boost::none) {
    //TODO Test this code branch
    return boost::none;
  }

  return cpputils::make_unique_ref<EncryptedBlock>(std::move(*baseBlock), encKey, std::move(plaintextWithHeader));
}

template<class Cipher>
cpputils::Data EncryptedBlock<Cipher>::_prependFormatHeader(const cpputils::Data &data) {
  cpputils::Data dataWithHeader(sizeof(FORMAT_VERSION_HEADER) + data.size());
  std::memcpy(dataWithHeader.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
  std::memcpy(dataWithHeader.dataOffset(sizeof(FORMAT_VERSION_HEADER)), data.data(), data.size());
  return dataWithHeader;
}

template<class Cipher>
boost::optional<cpputils::unique_ref<EncryptedBlock<Cipher>>> EncryptedBlock<Cipher>::TryDecrypt(cpputils::unique_ref<Block> baseBlock, const typename Cipher::EncryptionKey &encKey) {
  _checkFormatHeader(baseBlock->data());
  boost::optional<cpputils::Data> plaintextWithHeader = Cipher::decrypt((byte*)baseBlock->data() + sizeof(FORMAT_VERSION_HEADER), baseBlock->size() - sizeof(FORMAT_VERSION_HEADER), encKey);
  if(plaintextWithHeader == boost::none) {
    //Decryption failed (e.g. an authenticated cipher detected modifications to the ciphertext)
    cpputils::logging::LOG(cpputils::logging::WARN) << "Decrypting block " << baseBlock->key().ToString() << " failed. Was the block modified by an attacker?";
    return boost::none;
  }
  if(!_keyHeaderIsCorrect(baseBlock->key(), *plaintextWithHeader)) {
    //The stored key in the block data is incorrect - an attacker might have exchanged the contents with the encrypted data from a different block
    cpputils::logging::LOG(cpputils::logging::WARN) << "Decrypting block " << baseBlock->key().ToString() << " failed due to invalid block key. Was the block modified by an attacker?";
    return boost::none;
  }
  return cpputils::make_unique_ref<EncryptedBlock<Cipher>>(std::move(baseBlock), encKey, std::move(*plaintextWithHeader));
}

template<class Cipher>
void EncryptedBlock<Cipher>::_checkFormatHeader(const void *data) {
  if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data) != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("The encrypted block has the wrong format. Was it created with a newer version of CryFS?");
  }
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
EncryptedBlock<Cipher>::EncryptedBlock(cpputils::unique_ref<Block> baseBlock, const typename Cipher::EncryptionKey &encKey, cpputils::Data plaintextWithHeader)
    :Block(baseBlock->key()),
   _baseBlock(std::move(baseBlock)),
   _plaintextWithHeader(std::move(plaintextWithHeader)),
   _encKey(encKey),
   _dataChanged(false),
   _mutex() {
}

template<class Cipher>
EncryptedBlock<Cipher>::~EncryptedBlock() {
  std::unique_lock<std::mutex> lock(_mutex);
  _encryptToBaseBlock();
}

template<class Cipher>
const void *EncryptedBlock<Cipher>::data() const {
  return (uint8_t*)_plaintextWithHeader.data() + HEADER_LENGTH;
}

template<class Cipher>
void EncryptedBlock<Cipher>::write(const void *source, uint64_t offset, uint64_t count) {
  ASSERT(offset <= size() && offset + count <= size(), "Write outside of valid area"); //Also check offset < size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_plaintextWithHeader.data()+HEADER_LENGTH+offset, source, count);
  _dataChanged = true;
}

template<class Cipher>
void EncryptedBlock<Cipher>::flush() {
  std::unique_lock<std::mutex> lock(_mutex);
  _encryptToBaseBlock();
  return _baseBlock->flush();
}

template<class Cipher>
size_t EncryptedBlock<Cipher>::size() const {
  return _plaintextWithHeader.size() - HEADER_LENGTH;
}

template<class Cipher>
void EncryptedBlock<Cipher>::resize(size_t newSize) {
  _plaintextWithHeader = cpputils::DataUtils::resize(std::move(_plaintextWithHeader), newSize + HEADER_LENGTH);
  _dataChanged = true;
}

template<class Cipher>
void EncryptedBlock<Cipher>::_encryptToBaseBlock() {
  if (_dataChanged) {
    cpputils::Data encrypted = Cipher::encrypt((byte*)_plaintextWithHeader.data(), _plaintextWithHeader.size(), _encKey);
    if (_baseBlock->size() != sizeof(FORMAT_VERSION_HEADER) + encrypted.size()) {
      _baseBlock->resize(sizeof(FORMAT_VERSION_HEADER) + encrypted.size());
    }
    _baseBlock->write(&FORMAT_VERSION_HEADER, 0, sizeof(FORMAT_VERSION_HEADER));
    _baseBlock->write(encrypted.data(), sizeof(FORMAT_VERSION_HEADER), encrypted.size());
    _dataChanged = false;
  }
}

template<class Cipher>
cpputils::unique_ref<Block> EncryptedBlock<Cipher>::releaseBlock() {
  std::unique_lock<std::mutex> lock(_mutex);
  _encryptToBaseBlock();
  return std::move(_baseBlock);
}

template<class Cipher>
uint64_t EncryptedBlock<Cipher>::blockSizeFromPhysicalBlockSize(uint64_t blockSize) {
  if (blockSize <= Cipher::ciphertextSize(HEADER_LENGTH) + sizeof(FORMAT_VERSION_HEADER)) {
    return 0;
  }
  return Cipher::plaintextSize(blockSize - sizeof(FORMAT_VERSION_HEADER)) - HEADER_LENGTH;
}

}
}

#endif
