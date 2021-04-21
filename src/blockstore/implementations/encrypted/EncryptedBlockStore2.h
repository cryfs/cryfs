#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTEDBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/crypto/symmetric/Cipher.h>
#include <cpp-utils/data/SerializationHelper.h>

namespace blockstore {
namespace encrypted {

//TODO Format version headers

template<class Cipher>
class EncryptedBlockStore2 final: public BlockStore2 {
public:
  BOOST_CONCEPT_ASSERT((cpputils::CipherConcept<Cipher>));

  EncryptedBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const typename Cipher::EncryptionKey &encKey);

  bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
  bool remove(const BlockId &blockId) override;
  boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
  void store(const BlockId &blockId, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

  //This function should only be used by test cases
  void _setKey(const typename Cipher::EncryptionKey &encKey);

private:

  // This header is prepended to blocks to allow future versions to have compatibility.
#ifndef CRYFS_NO_COMPATIBILITY
  static constexpr uint16_t FORMAT_VERSION_HEADER_OLD = 0;
#endif
  static constexpr uint16_t FORMAT_VERSION_HEADER = 1;

  cpputils::Data _encrypt(const cpputils::Data &data) const;
  boost::optional<cpputils::Data> _tryDecrypt(const BlockId &blockId, const cpputils::Data &data) const;

  static cpputils::Data _prependFormatHeaderToData(const cpputils::Data &data);
#ifndef CRYFS_NO_COMPATIBILITY
  static bool _blockIdHeaderIsCorrect(const BlockId &blockId, const cpputils::Data &data);
  static cpputils::Data _migrateBlock(const cpputils::Data &data);
#endif
  static void _checkFormatHeader(const cpputils::Data &data);
  static uint16_t _readFormatHeader(const cpputils::Data &data);

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  typename Cipher::EncryptionKey _encKey;

  DISALLOW_COPY_AND_ASSIGN(EncryptedBlockStore2);
};

#ifndef CRYFS_NO_COMPATIBILITY
template<class Cipher>
constexpr uint16_t EncryptedBlockStore2<Cipher>::FORMAT_VERSION_HEADER_OLD;
#endif

template<class Cipher>
constexpr uint16_t EncryptedBlockStore2<Cipher>::FORMAT_VERSION_HEADER;

template<class Cipher>
inline EncryptedBlockStore2<Cipher>::EncryptedBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const typename Cipher::EncryptionKey &encKey)
: _baseBlockStore(std::move(baseBlockStore)), _encKey(encKey) {
}

template<class Cipher>
inline bool EncryptedBlockStore2<Cipher>::tryCreate(const BlockId &blockId, const cpputils::Data &data) {
  cpputils::Data encrypted = _encrypt(data);
  return _baseBlockStore->tryCreate(blockId, encrypted);
}

template<class Cipher>
inline bool EncryptedBlockStore2<Cipher>::remove(const BlockId &blockId) {
  return _baseBlockStore->remove(blockId);
}

template<class Cipher>
inline boost::optional<cpputils::Data> EncryptedBlockStore2<Cipher>::load(const BlockId &blockId) const {
  auto loaded = _baseBlockStore->load(blockId);

  if (boost::none == loaded) {
    return boost::optional<cpputils::Data>(boost::none);
  }
  return _tryDecrypt(blockId, *loaded);
}

template<class Cipher>
inline void EncryptedBlockStore2<Cipher>::store(const BlockId &blockId, const cpputils::Data &data) {
  cpputils::Data encrypted = _encrypt(data);
  return _baseBlockStore->store(blockId, encrypted);
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
  if (baseBlockSize <= Cipher::ciphertextSize(0) + sizeof(FORMAT_VERSION_HEADER)) {
    return 0;
  }
  return Cipher::plaintextSize(baseBlockSize - sizeof(FORMAT_VERSION_HEADER));
}

template<class Cipher>
inline void EncryptedBlockStore2<Cipher>::forEachBlock(std::function<void (const BlockId &)> callback) const {
  return _baseBlockStore->forEachBlock(std::move(callback));
}

template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_encrypt(const cpputils::Data &data) const {
  cpputils::Data encrypted = Cipher::encrypt(static_cast<const CryptoPP::byte*>(data.data()), data.size(), _encKey);
  return _prependFormatHeaderToData(encrypted);
}

template<class Cipher>
inline boost::optional<cpputils::Data> EncryptedBlockStore2<Cipher>::_tryDecrypt(const BlockId &blockId, const cpputils::Data &data) const {
  _checkFormatHeader(data);
  boost::optional<cpputils::Data> decrypted = Cipher::decrypt(static_cast<const CryptoPP::byte*>(data.dataOffset(sizeof(FORMAT_VERSION_HEADER))), data.size() - sizeof(FORMAT_VERSION_HEADER), _encKey);
  if (decrypted == boost::none) {
    // TODO Log warning
    return boost::none;
  }

#ifndef CRYFS_NO_COMPATIBILITY
  if (FORMAT_VERSION_HEADER_OLD == _readFormatHeader(data)) {
    if (!_blockIdHeaderIsCorrect(blockId, *decrypted)) {
      return boost::none;
    }
    *decrypted = _migrateBlock(*decrypted);
    // no need to write migrated back to block store because
    // this migration happens in line with a migration in IntegrityBlockStore2
    // which then writes it back
  }
#endif
  return decrypted;
}

#ifndef CRYFS_NO_COMPATIBILITY
template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_migrateBlock(const cpputils::Data &data) {
  return data.copyAndRemovePrefix(BlockId::BINARY_LENGTH);
}

template<class Cipher>
inline bool EncryptedBlockStore2<Cipher>::_blockIdHeaderIsCorrect(const BlockId &blockId, const cpputils::Data &data) {
  return blockId == BlockId::FromBinary(data.data());
}
#endif

template<class Cipher>
inline cpputils::Data EncryptedBlockStore2<Cipher>::_prependFormatHeaderToData(const cpputils::Data &data) {
  cpputils::Data dataWithHeader(sizeof(FORMAT_VERSION_HEADER) + data.size());
  cpputils::serialize<uint16_t>(dataWithHeader.dataOffset(0), FORMAT_VERSION_HEADER);
  std::memcpy(dataWithHeader.dataOffset(sizeof(FORMAT_VERSION_HEADER)), data.data(), data.size());
  return dataWithHeader;
}

template<class Cipher>
inline void EncryptedBlockStore2<Cipher>::_checkFormatHeader(const cpputils::Data &data) {
  const uint16_t formatVersionHeader = _readFormatHeader(data);
#ifndef CRYFS_NO_COMPATIBILITY
  const bool formatVersionHeaderValid = formatVersionHeader == FORMAT_VERSION_HEADER || formatVersionHeader == FORMAT_VERSION_HEADER_OLD;
#else
  const bool formatVersionHeaderValid = formatVersionHeader == FORMAT_VERSION_HEADER;
#endif
  if (!formatVersionHeaderValid) {
    throw std::runtime_error("The encrypted block has the wrong format. Was it created with a newer version of CryFS?");
  }
}

template<class Cipher>
uint16_t EncryptedBlockStore2<Cipher>::_readFormatHeader(const cpputils::Data &data) {
  return cpputils::deserialize<uint16_t>(data.data());
}

template<class Cipher>
void EncryptedBlockStore2<Cipher>::_setKey(const typename Cipher::EncryptionKey &encKey) {
  _encKey = encKey;
}

}
}

#endif
