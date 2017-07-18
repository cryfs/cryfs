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
#include "IntegrityViolationError.h"
#include "VersionCountingBlockStore.h"

namespace blockstore {
namespace versioncounting {

// TODO Is an implementation that doesn't keep an in-memory copy but just passes through write() calls to the underlying block store (including a write call to the version number each time) faster?

class VersionCountingBlock final: public Block {
public:
  static boost::optional<cpputils::unique_ref<VersionCountingBlock>> TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, VersionCountingBlockStore *blockStore);
  static cpputils::unique_ref<VersionCountingBlock> Overwrite(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, VersionCountingBlockStore *blockStore);
  static cpputils::unique_ref<VersionCountingBlock> Load(cpputils::unique_ref<Block> baseBlock, VersionCountingBlockStore *blockStore);

  static uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize);

  //TODO Storing key twice (in parent class and in object pointed to). Once would be enough.
  VersionCountingBlock(cpputils::unique_ref<Block> baseBlock, cpputils::Data dataWithHeader, VersionCountingBlockStore *blockStore);
  ~VersionCountingBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t count) override;
  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

  cpputils::unique_ref<Block> releaseBlock();

private:
  VersionCountingBlockStore *_blockStore;
  cpputils::unique_ref<Block> _baseBlock;
  cpputils::Data _dataWithHeader;
  bool _dataChanged;
  std::mutex _mutex;

  void _storeToBaseBlock();
  static cpputils::Data _prependHeaderToData(uint32_t myClientId, uint64_t version, cpputils::Data data);
  static void _checkFormatHeader(const cpputils::Data &data);
  uint64_t _readVersion();
  uint32_t _readClientId();
  void _checkVersion();

  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;

  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlock);

public:
    static constexpr uint64_t VERSION_ZERO = 0;
    static constexpr unsigned int CLIENTID_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER);
    static constexpr unsigned int VERSION_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t);
    static constexpr unsigned int HEADER_LENGTH = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t) + sizeof(VERSION_ZERO);
};


inline boost::optional<cpputils::unique_ref<VersionCountingBlock>> VersionCountingBlock::TryCreateNew(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, VersionCountingBlockStore *blockStore) {
  uint64_t version = blockStore->knownBlockVersions()->incrementVersion(key);

  cpputils::Data dataWithHeader = _prependHeaderToData(blockStore->knownBlockVersions()->myClientId(), version, std::move(data));
  auto baseBlock = baseBlockStore->tryCreate(key, dataWithHeader.copy()); // TODO Copy necessary?
  if (baseBlock == boost::none) {
    //TODO Test this code branch
    return boost::none;
  }

  return cpputils::make_unique_ref<VersionCountingBlock>(std::move(*baseBlock), std::move(dataWithHeader), blockStore);
}

inline cpputils::unique_ref<VersionCountingBlock> VersionCountingBlock::Overwrite(BlockStore *baseBlockStore, const Key &key, cpputils::Data data, VersionCountingBlockStore *blockStore) {
  uint64_t version = blockStore->knownBlockVersions()->incrementVersion(key);

  cpputils::Data dataWithHeader = _prependHeaderToData(blockStore->knownBlockVersions()->myClientId(), version, std::move(data));
  auto baseBlock = baseBlockStore->overwrite(key, dataWithHeader.copy()); // TODO Copy necessary?
  return cpputils::make_unique_ref<VersionCountingBlock>(std::move(baseBlock), std::move(dataWithHeader), blockStore);
}

inline cpputils::Data VersionCountingBlock::_prependHeaderToData(uint32_t myClientId, uint64_t version, cpputils::Data data) {
  static_assert(HEADER_LENGTH == sizeof(FORMAT_VERSION_HEADER) + sizeof(myClientId) + sizeof(version), "Wrong header length");
  cpputils::Data result(data.size() + HEADER_LENGTH);
  std::memcpy(result.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
  std::memcpy(result.dataOffset(CLIENTID_HEADER_OFFSET), &myClientId, sizeof(myClientId));
  std::memcpy(result.dataOffset(VERSION_HEADER_OFFSET), &version, sizeof(version));
  std::memcpy((uint8_t*)result.dataOffset(HEADER_LENGTH), data.data(), data.size());
  return result;
}

inline cpputils::unique_ref<VersionCountingBlock> VersionCountingBlock::Load(cpputils::unique_ref<Block> baseBlock, VersionCountingBlockStore *blockStore) {
  cpputils::Data data(baseBlock->size());
  std::memcpy(data.data(), baseBlock->data(), data.size());
  _checkFormatHeader(data);
  auto block = cpputils::make_unique_ref<VersionCountingBlock>(std::move(baseBlock), std::move(data), blockStore);
  block->_checkVersion();
  return std::move(block);
}

inline void VersionCountingBlock::_checkVersion() {
  uint32_t lastClientId = _readClientId();
  uint64_t version = _readVersion();
  if(!_blockStore->knownBlockVersions()->checkAndUpdateVersion(lastClientId, key(), version)) {
    _blockStore->integrityViolationDetected("The block version number is too low. Did an attacker try to roll back the block or to re-introduce a deleted block?");
  }
}

inline void VersionCountingBlock::_checkFormatHeader(const cpputils::Data &data) {
  if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data()) != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("The versioned block has the wrong format. Was it created with a newer version of CryFS?");
  }
}

inline uint32_t VersionCountingBlock::_readClientId() {
  uint32_t clientId;
  std::memcpy(&clientId, _dataWithHeader.dataOffset(CLIENTID_HEADER_OFFSET), sizeof(clientId));
  return clientId;
}

inline uint64_t VersionCountingBlock::_readVersion() {
  uint64_t version;
  std::memcpy(&version, _dataWithHeader.dataOffset(VERSION_HEADER_OFFSET), sizeof(version));
  return version;
}

inline VersionCountingBlock::VersionCountingBlock(cpputils::unique_ref<Block> baseBlock, cpputils::Data dataWithHeader, VersionCountingBlockStore *blockStore)
    :Block(baseBlock->key()),
   _blockStore(blockStore),
   _baseBlock(std::move(baseBlock)),
   _dataWithHeader(std::move(dataWithHeader)),
   _dataChanged(false),
   _mutex() {
  if (_readVersion() == std::numeric_limits<uint64_t>::max()) {
    throw std::runtime_error("Version overflow when loading. This shouldn't happen because in case of a version number overflow, the block isn't stored at all.");
  }
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
    uint64_t version = _blockStore->knownBlockVersions()->incrementVersion(key());
    uint32_t myClientId = _blockStore->knownBlockVersions()->myClientId();
    std::memcpy(_dataWithHeader.dataOffset(CLIENTID_HEADER_OFFSET), &myClientId, sizeof(myClientId));
    std::memcpy(_dataWithHeader.dataOffset(VERSION_HEADER_OFFSET), &version, sizeof(version));
    if (_baseBlock->size() != _dataWithHeader.size()) {
      _baseBlock->resize(_dataWithHeader.size());
    }
    _baseBlock->write(_dataWithHeader.data(), 0, _dataWithHeader.size());
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
