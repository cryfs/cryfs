#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>
#include "../../interface/BlockStore.h"
#include "KnownBlockVersions.h"

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
namespace versioncounting {
class VersionCountingBlockStore;

// TODO Is an implementation that doesn't keep an in-memory copy but just passes through write() calls to the underlying block store (including a write call to the version number each time) faster?

class VersionCountingBlock final: public Block {
public:
  static boost::optional<cpputils::unique_ref<VersionCountingBlock>> TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, KnownBlockVersions *knownBlockVersions);
  static boost::optional<cpputils::unique_ref<VersionCountingBlock>> TryLoad(cpputils::unique_ref<Block> baseBlock, KnownBlockVersions *knownBlockVersions);

  static uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize);

  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  VersionCountingBlock(cpputils::unique_ref<Block> baseBlock, cpputils::Data dataWithHeader, uint64_t version, KnownBlockVersions *knownBlockVersions);
  ~VersionCountingBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t count) override;
  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

  cpputils::unique_ref<Block> releaseBlock();

private:
  KnownBlockVersions *_knownBlockVersions;
  cpputils::unique_ref<Block> _baseBlock;
  cpputils::Data _dataWithHeader;
  uint64_t _version;
  bool _dataChanged;

  void _storeToBaseBlock();
  static cpputils::Data _prependHeaderToData(uint64_t version, cpputils::Data data);
  static void _checkFormatHeader(const cpputils::Data &data);
  static uint64_t _readVersion(const cpputils::Data &data);
  static bool _versionIsNondecreasing(const Key &key, uint64_t version, KnownBlockVersions *knownBlockVersions);

  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;
  static constexpr uint64_t VERSION_ZERO = 0;
  static constexpr unsigned int HEADER_LENGTH = sizeof(FORMAT_VERSION_HEADER) + sizeof(VERSION_ZERO);

  std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlock);
};


inline boost::optional<cpputils::unique_ref<VersionCountingBlock>> VersionCountingBlock::TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, KnownBlockVersions *knownBlockVersions) {
  cpputils::Data dataWithHeader = _prependHeaderToData(VERSION_ZERO, std::move(data));
  auto baseBlock = baseBlockStore->tryCreate(key, dataWithHeader.copy()); // TODO Copy necessary?
  if (baseBlock == boost::none) {
    //TODO Test this code branch
    return boost::none;
  }

  knownBlockVersions->updateVersion(key, VERSION_ZERO);
  return cpputils::make_unique_ref<VersionCountingBlock>(std::move(*baseBlock), std::move(dataWithHeader), VERSION_ZERO, knownBlockVersions);
}

inline cpputils::Data VersionCountingBlock::_prependHeaderToData(const uint64_t version, cpputils::Data data) {
  static_assert(HEADER_LENGTH == sizeof(FORMAT_VERSION_HEADER) + sizeof(version), "Wrong header length");
  cpputils::Data result(data.size() + HEADER_LENGTH);
  std::memcpy(result.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
  std::memcpy(result.dataOffset(sizeof(FORMAT_VERSION_HEADER)), &version, sizeof(version));
  std::memcpy((uint8_t*)result.dataOffset(HEADER_LENGTH), data.data(), data.size());
  return result;
}

inline boost::optional<cpputils::unique_ref<VersionCountingBlock>> VersionCountingBlock::TryLoad(cpputils::unique_ref<Block> baseBlock, KnownBlockVersions *knownBlockVersions) {
  cpputils::Data data(baseBlock->size());
  std::memcpy(data.data(), baseBlock->data(), data.size());
  _checkFormatHeader(data);
  uint64_t version = _readVersion(data);
  if(!_versionIsNondecreasing(baseBlock->key(), version, knownBlockVersions)) {
    //The stored key in the block data is incorrect - an attacker might have exchanged the contents with the encrypted data from a different block
    cpputils::logging::LOG(cpputils::logging::WARN) << "Decrypting block " << baseBlock->key().ToString() << " failed due to wrong version number. Was the block rolled back by an attacker?";
    return boost::none;
  }
  return cpputils::make_unique_ref<VersionCountingBlock>(std::move(baseBlock), std::move(data), version, knownBlockVersions);
}

inline void VersionCountingBlock::_checkFormatHeader(const cpputils::Data &data) {
  if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data()) != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("The versioned block has the wrong format. Was it created with a newer version of CryFS?");
  }
}

inline uint64_t VersionCountingBlock::_readVersion(const cpputils::Data &data) {
  uint64_t version;
  std::memcpy(&version, data.dataOffset(sizeof(FORMAT_VERSION_HEADER)), sizeof(version));
  return version;
}

inline bool VersionCountingBlock::_versionIsNondecreasing(const Key &key, uint64_t version, KnownBlockVersions *knownBlockVersions) {
  return knownBlockVersions->checkAndUpdateVersion(key, version);
}

inline VersionCountingBlock::VersionCountingBlock(cpputils::unique_ref<Block> baseBlock, cpputils::Data dataWithHeader, uint64_t version, KnownBlockVersions *knownBlockVersions)
    :Block(baseBlock->key()),
   _knownBlockVersions(knownBlockVersions),
   _baseBlock(std::move(baseBlock)),
   _dataWithHeader(std::move(dataWithHeader)),
   _version(version),
   _dataChanged(false),
   _mutex() {
}

inline VersionCountingBlock::~VersionCountingBlock() {
  std::unique_lock<std::mutex> lock(_mutex);
  _storeToBaseBlock();
}

inline const void *VersionCountingBlock::data() const {
  return (uint8_t*)_dataWithHeader.data() + HEADER_LENGTH;
}

inline void VersionCountingBlock::write(const void *source, uint64_t offset, uint64_t count) {
  ASSERT(offset <= size() && offset + count <= size(), "Write outside of valid area"); //Also check offset < size() because of possible overflow in the addition
  std::memcpy((uint8_t*)_dataWithHeader.data()+HEADER_LENGTH+offset, source, count);
  _dataChanged = true;
}

inline void VersionCountingBlock::flush() {
  std::unique_lock<std::mutex> lock(_mutex);
  _storeToBaseBlock();
  return _baseBlock->flush();
}

inline size_t VersionCountingBlock::size() const {
  return _dataWithHeader.size() - HEADER_LENGTH;
}

inline void VersionCountingBlock::resize(size_t newSize) {
  _dataWithHeader = cpputils::DataUtils::resize(std::move(_dataWithHeader), newSize + HEADER_LENGTH);
  _dataChanged = true;
}

inline void VersionCountingBlock::_storeToBaseBlock() {
  if (_dataChanged) {
    ++_version;
    std::memcpy(_dataWithHeader.dataOffset(sizeof(FORMAT_VERSION_HEADER)), &_version, sizeof(_version));
    _baseBlock->write(_dataWithHeader.data(), 0, _dataWithHeader.size());
    _knownBlockVersions->updateVersion(key(), _version);
    _dataChanged = false;
  }
}

inline cpputils::unique_ref<Block> VersionCountingBlock::releaseBlock() {
  std::unique_lock<std::mutex> lock(_mutex);
  _storeToBaseBlock();
  return std::move(_baseBlock);
}

inline uint64_t VersionCountingBlock::blockSizeFromPhysicalBlockSize(uint64_t blockSize) {
  if (blockSize <= HEADER_LENGTH) {
    return 0;
  }
  return blockSize - HEADER_LENGTH;
}

}
}

#endif
