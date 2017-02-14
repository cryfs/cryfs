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
    cpputils::Data encrypted = _encrypt(data);
    return _baseBlockStore->tryCreate(key, encrypted);
  }

  boost::future<bool> remove(const Key &key) override {
    return _baseBlockStore->remove(key);
  }

  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override {
    auto loaded = _baseBlockStore->load(key);
    return loaded.then([this] (boost::future<boost::optional<cpputils::Data>> data_) {
      auto data = data_.get();
      if (boost::none == data) {
        return boost::optional<cpputils::Data>(boost::none);
      }
      return _tryDecrypt(data);
    });
  }

  boost::future<void> store(const Key &key, const cpputils::Data &data) override {
    cpputils::Data encrypted = _encrypt(data);
    return _baseBlockStore->store(key, encrypted);
  }

private:

  cpputils::Data _encrypt(const Key &key, const cpputils::Data &data) {
    cpputils::Data plaintextWithHeader = _prependKeyHeaderToData(key, data);
    return Cipher::encrypt((byte*)plaintextWithHeader.data(), plaintextWithHeader.size(), _encKey);
  }

  boost::optional<cpputils::Data> _tryDecrypt(const Key &key, const cpputils::Data &data) {
    boost::optional<cpputils::Data> decrypted = Cipher::decrypt((byte*)data.data(), data.size(), _encKey);
    if (boost::none != decrypted && !_keyHeaderIsCorrect(key, *decrypted)) {
      // TODO Warning
      return boost::none;
    }
    return decrypted;
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

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  typename Cipher::EncryptionKey _encKey;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStore2);
};

}
}

#endif
