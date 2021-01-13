#include <blockstore/interface/BlockStore2.h>
#include "IntegrityBlockStore2.h"
#include "KnownBlockVersions.h"
#include <cpp-utils/data/SerializationHelper.h>
#include <cpp-utils/process/SignalCatcher.h>
#include <cpp-utils/io/ProgressBar.h>

using cpputils::Data;
using cpputils::unique_ref;
using cpputils::serialize;
using cpputils::deserialize;
using cpputils::SignalCatcher;
using std::string;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace blockstore {
namespace integrity {

#ifndef CRYFS_NO_COMPATIBILITY
constexpr uint16_t IntegrityBlockStore2::FORMAT_VERSION_HEADER_OLD;
#endif
constexpr uint16_t IntegrityBlockStore2::FORMAT_VERSION_HEADER;
constexpr uint64_t IntegrityBlockStore2::VERSION_ZERO;
constexpr unsigned int IntegrityBlockStore2::ID_HEADER_OFFSET;
constexpr unsigned int IntegrityBlockStore2::CLIENTID_HEADER_OFFSET;
constexpr unsigned int IntegrityBlockStore2::VERSION_HEADER_OFFSET;
constexpr unsigned int IntegrityBlockStore2::HEADER_LENGTH;

Data IntegrityBlockStore2::_prependHeaderToData(const BlockId& blockId, uint32_t myClientId, uint64_t version, const Data &data) {
  static_assert(HEADER_LENGTH == sizeof(FORMAT_VERSION_HEADER) + BlockId::BINARY_LENGTH + sizeof(myClientId) + sizeof(version), "Wrong header length");
  Data result(data.size() + HEADER_LENGTH);
  serialize<uint16_t>(result.dataOffset(0), FORMAT_VERSION_HEADER);
  blockId.ToBinary(result.dataOffset(ID_HEADER_OFFSET));
  serialize<uint32_t>(result.dataOffset(CLIENTID_HEADER_OFFSET), myClientId);
  serialize<uint64_t>(result.dataOffset(VERSION_HEADER_OFFSET), version);
  std::memcpy(result.dataOffset(HEADER_LENGTH), data.data(), data.size());
  return result;
}

bool IntegrityBlockStore2::_checkHeader(const BlockId &blockId, const Data &data) const {
  _checkFormatHeader(data);
  return _checkIdHeader(blockId, data) && _checkVersionHeader(blockId, data);
}

void IntegrityBlockStore2::_checkFormatHeader(const Data &data) const {
  if (FORMAT_VERSION_HEADER != _readFormatHeader(data)) {
    throw std::runtime_error("The versioned block has the wrong format. Was it created with a newer version of CryFS?");
  }
}

bool IntegrityBlockStore2::_checkVersionHeader(const BlockId &blockId, const Data &data) const {
  uint32_t clientId = _readClientId(data);
  uint64_t version = _readVersion(data);

  if(!_knownBlockVersions.checkAndUpdateVersion(clientId, blockId, version)) {
    integrityViolationDetected("The block version number is too low. Did an attacker try to roll back the block or to re-introduce a deleted block?");
    return false;
  }

  return true;
}

bool IntegrityBlockStore2::_checkIdHeader(const BlockId &expectedBlockId, const Data &data) const {
  // The obvious reason for this is to prevent adversaries from renaming blocks, but storing the block id in this way also
  // makes the authenticated cipher more robust, see https://libsodium.gitbook.io/doc/secret-key_cryptography/aead#robustness
  BlockId actualBlockId = _readBlockId(data);
  if (expectedBlockId != actualBlockId) {
    integrityViolationDetected("The block id is wrong. Did an attacker try to rename some blocks?");
    return false;
  }

  return true;
}

uint16_t IntegrityBlockStore2::_readFormatHeader(const Data &data) {
  return deserialize<uint16_t>(data.data());
}

uint32_t IntegrityBlockStore2::_readClientId(const Data &data) {
  return deserialize<uint32_t>(data.dataOffset(CLIENTID_HEADER_OFFSET));
}

BlockId IntegrityBlockStore2::_readBlockId(const Data &data) {
  return BlockId::FromBinary(data.dataOffset(ID_HEADER_OFFSET));
}

uint64_t IntegrityBlockStore2::_readVersion(const Data &data) {
  return deserialize<uint64_t>(data.dataOffset(VERSION_HEADER_OFFSET));
}

Data IntegrityBlockStore2::_removeHeader(const Data &data) {
  return data.copyAndRemovePrefix(HEADER_LENGTH);
}

void IntegrityBlockStore2::integrityViolationDetected(const string &reason) const {
  if (_allowIntegrityViolations) {
    LOG(WARN, "Integrity violation (but integrity checks are disabled): {}", reason);
    return;
  }
  _knownBlockVersions.setIntegrityViolationOnPreviousRun(true);
  _onIntegrityViolation();
}

IntegrityBlockStore2::IntegrityBlockStore2(unique_ref<BlockStore2> baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void ()> onIntegrityViolation)
: _baseBlockStore(std::move(baseBlockStore)), _knownBlockVersions(integrityFilePath, myClientId), _allowIntegrityViolations(allowIntegrityViolations), _missingBlockIsIntegrityViolation(missingBlockIsIntegrityViolation), _onIntegrityViolation(std::move(onIntegrityViolation)) {
  if (_knownBlockVersions.integrityViolationOnPreviousRun()) {
    throw IntegrityViolationOnPreviousRun(_knownBlockVersions.path());
  }
}

bool IntegrityBlockStore2::tryCreate(const BlockId &blockId, const Data &data) {
  uint64_t version = _knownBlockVersions.incrementVersion(blockId);
  Data dataWithHeader = _prependHeaderToData(blockId, _knownBlockVersions.myClientId(), version, data);
  return _baseBlockStore->tryCreate(blockId, dataWithHeader);
}

bool IntegrityBlockStore2::remove(const BlockId &blockId) {
  _knownBlockVersions.markBlockAsDeleted(blockId);
  return _baseBlockStore->remove(blockId);
}

optional<Data> IntegrityBlockStore2::load(const BlockId &blockId) const {
  auto loaded = _baseBlockStore->load(blockId);
  if (none == loaded) {
    if (_missingBlockIsIntegrityViolation && _knownBlockVersions.blockShouldExist(blockId)) {
      integrityViolationDetected("A block that should exist wasn't found. Did an attacker delete it?");
    }
    return optional<Data>(none);
  }
#ifndef CRYFS_NO_COMPATIBILITY
  if (FORMAT_VERSION_HEADER_OLD == _readFormatHeader(*loaded)) {
    Data migrated = _migrateBlock(blockId, *loaded);
    if (!_checkHeader(blockId, migrated) && !_allowIntegrityViolations) {
      return optional<Data>(none);
    }
    Data content = _removeHeader(migrated);
    const_cast<IntegrityBlockStore2*>(this)->store(blockId, content);
    return optional<Data>(_removeHeader(migrated));
  }
#endif
  if (!_checkHeader(blockId, *loaded) && !_allowIntegrityViolations) {
    return optional<Data>(none);
  }
  return optional<Data>(_removeHeader(*loaded));
}

#ifndef CRYFS_NO_COMPATIBILITY
Data IntegrityBlockStore2::_migrateBlock(const BlockId &blockId, const Data &data) {
  Data migrated(data.size() + BlockId::BINARY_LENGTH);
  serialize<uint16_t>(migrated.dataOffset(0), FORMAT_VERSION_HEADER);
  blockId.ToBinary(migrated.dataOffset(ID_HEADER_OFFSET));
  std::memcpy(migrated.dataOffset(ID_HEADER_OFFSET + BlockId::BINARY_LENGTH), data.dataOffset(sizeof(FORMAT_VERSION_HEADER)), data.size() - sizeof(FORMAT_VERSION_HEADER));
  ASSERT(migrated.size() == sizeof(FORMAT_VERSION_HEADER) + BlockId::BINARY_LENGTH + (data.size() - sizeof(FORMAT_VERSION_HEADER)), "Wrong offset computation");
  return migrated;
}
#endif

void IntegrityBlockStore2::store(const BlockId &blockId, const Data &data) {
  uint64_t version = _knownBlockVersions.incrementVersion(blockId);
  Data dataWithHeader = _prependHeaderToData(blockId, _knownBlockVersions.myClientId(), version, data);
  return _baseBlockStore->store(blockId, dataWithHeader);
}

uint64_t IntegrityBlockStore2::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

uint64_t IntegrityBlockStore2::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

uint64_t IntegrityBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  uint64_t baseBlockSize = _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
  if (baseBlockSize <= HEADER_LENGTH) {
    return 0;
  }
  return baseBlockSize - HEADER_LENGTH;
}

void IntegrityBlockStore2::forEachBlock(std::function<void (const BlockId &)> callback) const {
  if (!_missingBlockIsIntegrityViolation) {
    return _baseBlockStore->forEachBlock(std::move(callback));
  }

  std::unordered_set<blockstore::BlockId> existingBlocks = _knownBlockVersions.existingBlocks();
  _baseBlockStore->forEachBlock([&existingBlocks, callback] (const BlockId &blockId) {
    callback(blockId);

    auto found = existingBlocks.find(blockId);
    if (found != existingBlocks.end()) {
      existingBlocks.erase(found);
    }
  });
  if (!existingBlocks.empty()) {
    integrityViolationDetected("A block that should have existed wasn't found.");
  }
}

#ifndef CRYFS_NO_COMPATIBILITY
void IntegrityBlockStore2::migrateFromBlockstoreWithoutVersionNumbers(BlockStore2 *baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId) {
  SignalCatcher signalCatcher;

  KnownBlockVersions knownBlockVersions(integrityFilePath, myClientId);
  uint64_t numProcessedBlocks = 0;
  cpputils::ProgressBar progressbar("Migrating file system for integrity features. This can take a while...", baseBlockStore->numBlocks());
  baseBlockStore->forEachBlock([&] (const BlockId &blockId) {
    if (signalCatcher.signal_occurred()) {
      throw std::runtime_error("Caught signal");
    }
    migrateBlockFromBlockstoreWithoutVersionNumbers(baseBlockStore, blockId, &knownBlockVersions);
    progressbar.update(++numProcessedBlocks);
  });
}

void IntegrityBlockStore2::migrateBlockFromBlockstoreWithoutVersionNumbers(blockstore::BlockStore2* baseBlockStore, const blockstore::BlockId& blockId, KnownBlockVersions *knownBlockVersions) {
  auto data_ = baseBlockStore->load(blockId);
  if (data_ == boost::none) {
    LOG(WARN, "Block not found, but was returned from forEachBlock before");
    return;
  }
  if (0 != _readFormatHeader(*data_)) {
      // already migrated
      return;
  }

  uint64_t version = knownBlockVersions->incrementVersion(blockId);
  cpputils::Data data = std::move(*data_);
  cpputils::Data dataWithHeader = _prependHeaderToData(blockId, knownBlockVersions->myClientId(), version, data);
  baseBlockStore->store(blockId, dataWithHeader);
}
#endif

}
}
