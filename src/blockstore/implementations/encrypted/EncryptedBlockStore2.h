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

  EncryptedBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const typename Cipher::EncryptionKey &encKey)
      : _baseBlockStore(std::move(baseBlockStore)), _encKey(encKey) {
  }

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override {
    cpputils::Data encrypted = _encrypt(key, data);
    return _baseBlockStore->tryCreate(key, encrypted);
  }

  boost::future<bool> remove(const Key &key) override {
    return _baseBlockStore->remove(key);
  }

  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override {
    auto loaded = _baseBlockStore->load(key);
    return loaded.then([this, key] (boost::future<boost::optional<cpputils::Data>> data_) {
      auto data = data_.get();
      if (boost::none == data) {
        return boost::optional<cpputils::Data>(boost::none);
      }
      return _tryDecrypt(key, *data);
    });
  }

  boost::future<void> store(const Key &key, const cpputils::Data &data) override {
    cpputils::Data encrypted = _encrypt(key, data);
    return _baseBlockStore->store(key, encrypted);
  }

  uint64_t numBlocks() const override {
    return _baseBlockStore->numBlocks();
  }

  uint64_t estimateNumFreeBytes() const override {
    return _baseBlockStore->estimateNumFreeBytes();
  }

  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override {
    uint64_t baseBlockSize = _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
    if (baseBlockSize <= Cipher::ciphertextSize(HEADER_LENGTH) + sizeof(FORMAT_VERSION_HEADER)) {
      return 0;
    }
    return Cipher::plaintextSize(baseBlockSize - sizeof(FORMAT_VERSION_HEADER)) - HEADER_LENGTH;
  }

  void forEachBlock(std::function<void (const Key &)> callback) const override {
    return _baseBlockStore->forEachBlock(std::move(callback));
  }

private:

  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;

  static constexpr unsigned int HEADER_LENGTH = Key::BINARY_LENGTH;

  cpputils::Data _encrypt(const Key &key, const cpputils::Data &data) const {
    cpputils::Data plaintextWithHeader = _prependKeyHeaderToData(key, data);
    cpputils::Data encrypted = Cipher::encrypt((byte*)plaintextWithHeader.data(), plaintextWithHeader.size(), _encKey);
    return _prependFormatHeaderToData(encrypted);
  }

  boost::optional<cpputils::Data> _tryDecrypt(const Key &key, const cpputils::Data &data) const {
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

  static cpputils::Data _prependKeyHeaderToData(const Key &key, const cpputils::Data &data) {
    cpputils::Data result(data.size() + Key::BINARY_LENGTH);
    std::memcpy(result.data(), key.data(), Key::BINARY_LENGTH);
    std::memcpy((uint8_t*)result.data() + Key::BINARY_LENGTH, data.data(), data.size());
    return result;
  }

  static bool _keyHeaderIsCorrect(const Key &key, const cpputils::Data &data) {
    return 0 == std::memcmp(key.data(), data.data(), Key::BINARY_LENGTH);
  }

  static cpputils::Data _prependFormatHeaderToData(const cpputils::Data &data) {
    cpputils::Data dataWithHeader(sizeof(FORMAT_VERSION_HEADER) + data.size());
    std::memcpy(dataWithHeader.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
    std::memcpy(dataWithHeader.dataOffset(sizeof(FORMAT_VERSION_HEADER)), data.data(), data.size());
    return dataWithHeader;
  }

  static cpputils::Data _removeKeyHeader(const cpputils::Data &data) {
    return data.copyAndRemovePrefix(Key::BINARY_LENGTH);
  }

  static cpputils::Data _checkAndRemoveFormatHeader(const cpputils::Data &data) {
    if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data()) != FORMAT_VERSION_HEADER) {
      throw std::runtime_error("The encrypted block has the wrong format. Was it created with a newer version of CryFS?");
    }
    return data.copyAndRemovePrefix(sizeof(FORMAT_VERSION_HEADER));
  }

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  typename Cipher::EncryptionKey _encKey;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStore2);
};

}
}

#endif
