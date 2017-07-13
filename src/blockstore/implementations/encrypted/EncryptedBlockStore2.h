#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/crypto/symmetric/Cipher.h>

namespace blockstore {
namespace encrypted {

//TODO Format version headers

template<class Cipher>
class EncryptedBlockStore2 final: public BlockStore2 {
public:
  BOOST_CONCEPT_ASSERT((cpputils::CipherConcept<Cipher>));

  EncryptedBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const typename Cipher::EncryptionKey &encKey);

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override;
  boost::future<bool> remove(const Key &key) override;
  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override;
  boost::future<void> store(const Key &key, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:

  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;
  static constexpr unsigned int HEADER_LENGTH = Key::BINARY_LENGTH;

  cpputils::Data _encrypt(const Key &key, const cpputils::Data &data) const;
  boost::optional<cpputils::Data> _tryDecrypt(const Key &key, const cpputils::Data &data) const;

  static cpputils::Data _prependKeyHeaderToData(const Key &key, const cpputils::Data &data);
  static bool _keyHeaderIsCorrect(const Key &key, const cpputils::Data &data);
  static cpputils::Data _prependFormatHeaderToData(const cpputils::Data &data);
  static cpputils::Data _removeKeyHeader(const cpputils::Data &data);
  static cpputils::Data _checkAndRemoveFormatHeader(const cpputils::Data &data);

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  typename Cipher::EncryptionKey _encKey;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStore2);
};

template<class Cipher>
inline EncryptedBlockStore2<Cipher>::EncryptedBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const typename Cipher::EncryptionKey &encKey)
: _baseBlockStore(std::move(baseBlockStore)), _encKey(encKey) {
}

template<class Cipher>
inline boost::future<bool> EncryptedBlockStore2<Cipher>::tryCreate(const Key &key, const cpputils::Data &data) {
  cpputils::Data encrypted = _encrypt(key, data);
  return _baseBlockStore->tryCreate(key, encrypted);
}

template<class Cipher>
inline boost::future<bool> EncryptedBlockStore2<Cipher>::remove(const Key &key) {
  return _baseBlockStore->remove(key);
}

template<class Cipher>
inline boost::future<boost::optional<cpputils::Data>> EncryptedBlockStore2<Cipher>::load(const Key &key) const {
  auto loaded = _baseBlockStore->load(key);
  return loaded.then([this, key] (boost::future<boost::optional<cpputils::Data>> data_) {
    auto data = data_.get();
    if (boost::none == data) {
      return boost::optional<cpputils::Data>(boost::none);
    }
    return _tryDecrypt(key, *data);
  });
}

template<class Cipher>
inline boost::future<void> EncryptedBlockStore2<Cipher>::store(const Key &key, const cpputils::Data &data) {
  cpputils::Data encrypted = _encrypt(key, data);
  return _baseBlockStore->store(key, encrypted);
}

template<class Cipher>
inline uint64_t EncryptedBlockStore2<Cipher>::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

template<class Cipher>
inline uint64_t EncryptedBlockStore2<Cipher>::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

template<class Cipher>
inline uint64_t EncryptedBlockStore2<Cipher>::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  uint64_t baseBlockSize = _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
  if (baseBlockSize <= Cipher::ciphertextSize(HEADER_LENGTH) + sizeof(FORMAT_VERSION_HEADER)) {
    return 0;
  }
  return Cipher::plaintextSize(baseBlockSize - sizeof(FORMAT_VERSION_HEADER)) - HEADER_LENGTH;
}

template<class Cipher>
inline void EncryptedBlockStore2<Cipher>::forEachBlock(std::function<void (const Key &)> callback) const {
  return _baseBlockStore->forEachBlock(std::move(callback));
}

template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_encrypt(const Key &key, const cpputils::Data &data) const {
  cpputils::Data plaintextWithHeader = _prependKeyHeaderToData(key, data);
  cpputils::Data encrypted = Cipher::encrypt((byte*)plaintextWithHeader.data(), plaintextWithHeader.size(), _encKey);
  return _prependFormatHeaderToData(encrypted);
}

template<class Cipher>
inline boost::optional<cpputils::Data> EncryptedBlockStore2<Cipher>::_tryDecrypt(const Key &key, const cpputils::Data &data) const {
  auto ciphertext = _checkAndRemoveFormatHeader(data);
  boost::optional<cpputils::Data> decrypted = Cipher::decrypt((byte*)ciphertext.data(), ciphertext.size(), _encKey);
  if (boost::none == decrypted) {
    // TODO Warning
    return boost::none;
  }
  if (!_keyHeaderIsCorrect(key, *decrypted)) {
    // TODO Warning
    return boost::none;
  }
  return _removeKeyHeader(*decrypted);
}

template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_prependKeyHeaderToData(const Key &key, const cpputils::Data &data) {
  cpputils::Data result(data.size() + Key::BINARY_LENGTH);
  std::memcpy(result.data(), key.data(), Key::BINARY_LENGTH);
  std::memcpy((uint8_t*)result.data() + Key::BINARY_LENGTH, data.data(), data.size());
  return result;
}

template<class Cipher>
inline bool EncryptedBlockStore2<Cipher>::_keyHeaderIsCorrect(const Key &key, const cpputils::Data &data) {
  return 0 == std::memcmp(key.data(), data.data(), Key::BINARY_LENGTH);
}

template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_prependFormatHeaderToData(const cpputils::Data &data) {
  cpputils::Data dataWithHeader(sizeof(FORMAT_VERSION_HEADER) + data.size());
  std::memcpy(dataWithHeader.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
  std::memcpy(dataWithHeader.dataOffset(sizeof(FORMAT_VERSION_HEADER)), data.data(), data.size());
  return dataWithHeader;
}

template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_removeKeyHeader(const cpputils::Data &data) {
  return data.copyAndRemovePrefix(Key::BINARY_LENGTH);
}

template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_checkAndRemoveFormatHeader(const cpputils::Data &data) {
  if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data()) != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("The encrypted block has the wrong format. Was it created with a newer version of CryFS?");
  }
  return data.copyAndRemovePrefix(sizeof(FORMAT_VERSION_HEADER));
}

}
}

#endif
