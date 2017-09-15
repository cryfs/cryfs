#include <blockstore/interface/BlockStore2.h>
#include "VersionCountingBlockStore2.h"
#include "KnownBlockVersions.h"

using cpputils::Data;
using cpputils::unique_ref;
using std::string;
using boost::optional;
using boost::none;
using namespace cpputils::logging;

namespace blockstore {
namespace versioncounting {

#ifndef CRYFS_NO_COMPATIBILITY
constexpr uint16_t VersionCountingBlockStore2::FORMAT_VERSION_HEADER_OLD;
#endif
constexpr uint16_t VersionCountingBlockStore2::FORMAT_VERSION_HEADER;
constexpr uint64_t VersionCountingBlockStore2::VERSION_ZERO;
constexpr unsigned int VersionCountingBlockStore2::ID_HEADER_OFFSET;
constexpr unsigned int VersionCountingBlockStore2::CLIENTID_HEADER_OFFSET;
constexpr unsigned int VersionCountingBlockStore2::VERSION_HEADER_OFFSET;
constexpr unsigned int VersionCountingBlockStore2::HEADER_LENGTH;

Data VersionCountingBlockStore2::_prependHeaderToData(const Key& key, uint32_t myClientId, uint64_t version, const Data &data) {
  static_assert(HEADER_LENGTH == sizeof(FORMAT_VERSION_HEADER) + Key::BINARY_LENGTH + sizeof(myClientId) + sizeof(version), "Wrong header length");
  Data result(data.size() + HEADER_LENGTH);
  std::memcpy(result.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
  std::memcpy(result.dataOffset(ID_HEADER_OFFSET), key.data(), Key::BINARY_LENGTH);
  std::memcpy(result.dataOffset(CLIENTID_HEADER_OFFSET), &myClientId, sizeof(myClientId));
  std::memcpy(result.dataOffset(VERSION_HEADER_OFFSET), &version, sizeof(version));
  std::memcpy((uint8_t*)result.dataOffset(HEADER_LENGTH), data.data(), data.size());
  return result;
}

void VersionCountingBlockStore2::_checkHeader(const Key &key, const Data &data) const {
  _checkFormatHeader(data);
  _checkIdHeader(key, data);
  _checkVersionHeader(key, data);
}

void VersionCountingBlockStore2::_checkFormatHeader(const Data &data) const {
  if (FORMAT_VERSION_HEADER != _readFormatHeader(data)) {
    throw std::runtime_error("The versioned block has the wrong format. Was it created with a newer version of CryFS?");
  }
}

void VersionCountingBlockStore2::_checkVersionHeader(const Key &key, const Data &data) const {
  uint32_t clientId = _readClientId(data);
  uint64_t version = _readVersion(data);

  if(!_knownBlockVersions.checkAndUpdateVersion(clientId, key, version)) {
    integrityViolationDetected("The block version number is too low. Did an attacker try to roll back the block or to re-introduce a deleted block?");
  }
}

void VersionCountingBlockStore2::_checkIdHeader(const Key &expectedKey, const Data &data) const {
  Key actualKey = _readBlockId(data);
  if (expectedKey != actualKey) {
    integrityViolationDetected("The block key is wrong. Did an attacker try to rename some blocks?");
  }
}

uint16_t VersionCountingBlockStore2::_readFormatHeader(const Data &data) {
  return *reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data());
}

uint32_t VersionCountingBlockStore2::_readClientId(const Data &data) {
  uint32_t clientId;
  std::memcpy(&clientId, data.dataOffset(CLIENTID_HEADER_OFFSET), sizeof(clientId));
  return clientId;
}

Key VersionCountingBlockStore2::_readBlockId(const Data &data) {
  return Key::FromBinary(data.dataOffset(ID_HEADER_OFFSET));
}

uint64_t VersionCountingBlockStore2::_readVersion(const Data &data) {
  uint64_t version;
  std::memcpy(&version, data.dataOffset(VERSION_HEADER_OFFSET), sizeof(version));
  return version;
}

Data VersionCountingBlockStore2::_removeHeader(const Data &data) {
  return data.copyAndRemovePrefix(HEADER_LENGTH);
}

void VersionCountingBlockStore2::_checkNoPastIntegrityViolations() const {
  if (_integrityViolationDetected) {
    throw std::runtime_error(string() +
                             "There was an integrity violation detected. Preventing any further access to the file system. " +
                             "If you want to reset the integrity data (i.e. accept changes made by a potential attacker), " +
                             "please unmount the file system and delete the following file before re-mounting it: " +
                             _knownBlockVersions.path().native());
  }
}

void VersionCountingBlockStore2::integrityViolationDetected(const string &reason) const {
  _integrityViolationDetected = true;
  throw IntegrityViolationError(reason);
}

VersionCountingBlockStore2::VersionCountingBlockStore2(unique_ref<BlockStore2> baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId, bool missingBlockIsIntegrityViolation)
: _baseBlockStore(std::move(baseBlockStore)), _knownBlockVersions(integrityFilePath, myClientId), _missingBlockIsIntegrityViolation(missingBlockIsIntegrityViolation), _integrityViolationDetected(false) {
}

bool VersionCountingBlockStore2::tryCreate(const Key &key, const Data &data) {
  _checkNoPastIntegrityViolations();
  uint64_t version = _knownBlockVersions.incrementVersion(key);
  Data dataWithHeader = _prependHeaderToData(key, _knownBlockVersions.myClientId(), version, data);
  return _baseBlockStore->tryCreate(key, dataWithHeader);
}

bool VersionCountingBlockStore2::remove(const Key &key) {
  _checkNoPastIntegrityViolations();
  _knownBlockVersions.markBlockAsDeleted(key);
  return _baseBlockStore->remove(key);
}

optional<Data> VersionCountingBlockStore2::load(const Key &key) const {
  _checkNoPastIntegrityViolations();
  auto loaded = _baseBlockStore->load(key);
  if (none == loaded) {
    if (_missingBlockIsIntegrityViolation && _knownBlockVersions.blockShouldExist(key)) {
      integrityViolationDetected("A block that should exist wasn't found. Did an attacker delete it?");
    }
    return optional<Data>(none);
  }
#ifndef CRYFS_NO_COMPATIBILITY
  if (FORMAT_VERSION_HEADER_OLD == _readFormatHeader(*loaded)) {
    Data migrated = _migrateBlock(key, *loaded);
    _checkHeader(key, migrated);
    Data content = _removeHeader(migrated);
    const_cast<VersionCountingBlockStore2*>(this)->store(key, content);
    return optional<Data>(_removeHeader(migrated));
  }
#endif
  _checkHeader(key, *loaded);
  return optional<Data>(_removeHeader(*loaded));
}

#ifndef CRYFS_NO_COMPATIBILITY
Data VersionCountingBlockStore2::_migrateBlock(const Key &key, const Data &data) {
  Data migrated(data.size() + Key::BINARY_LENGTH);
  std::memcpy(migrated.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
  std::memcpy(migrated.dataOffset(ID_HEADER_OFFSET), key.data(), Key::BINARY_LENGTH);
  std::memcpy(migrated.dataOffset(ID_HEADER_OFFSET + Key::BINARY_LENGTH), data.dataOffset(sizeof(FORMAT_VERSION_HEADER)), data.size() - sizeof(FORMAT_VERSION_HEADER));
  ASSERT(migrated.size() == sizeof(FORMAT_VERSION_HEADER) + Key::BINARY_LENGTH + (data.size() - sizeof(FORMAT_VERSION_HEADER)), "Wrong offset computation");
  return migrated;
}
#endif

void VersionCountingBlockStore2::store(const Key &key, const Data &data) {
  _checkNoPastIntegrityViolations();
  uint64_t version = _knownBlockVersions.incrementVersion(key);
  Data dataWithHeader = _prependHeaderToData(key, _knownBlockVersions.myClientId(), version, data);
  return _baseBlockStore->store(key, dataWithHeader);
}

uint64_t VersionCountingBlockStore2::numBlocks() const {
  return _baseBlockStore->numBlocks();
}

uint64_t VersionCountingBlockStore2::estimateNumFreeBytes() const {
  return _baseBlockStore->estimateNumFreeBytes();
}

uint64_t VersionCountingBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
  uint64_t baseBlockSize = _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
  if (baseBlockSize <= HEADER_LENGTH) {
    return 0;
  }
  return baseBlockSize - HEADER_LENGTH;
}

void VersionCountingBlockStore2::forEachBlock(std::function<void (const Key &)> callback) const {
  if (!_missingBlockIsIntegrityViolation) {
    return _baseBlockStore->forEachBlock(std::move(callback));
  }

  std::unordered_set<blockstore::Key> existingBlocks = _knownBlockVersions.existingBlocks();
  _baseBlockStore->forEachBlock([&existingBlocks, callback] (const Key &key) {
    callback(key);

    auto found = existingBlocks.find(key);
    if (found != existingBlocks.end()) {
      existingBlocks.erase(found);
    }
  });
  if (!existingBlocks.empty()) {
    integrityViolationDetected("A block that should have existed wasn't found.");
  }
}

#ifndef CRYFS_NO_COMPATIBILITY
void VersionCountingBlockStore2::migrateFromBlockstoreWithoutVersionNumbers(BlockStore2 *baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId) {
  std::cout << "Migrating file system for integrity features. Please don't interrupt this process. This can take a while..." << std::flush;
  KnownBlockVersions knownBlockVersions(integrityFilePath, myClientId);
  baseBlockStore->forEachBlock([&baseBlockStore, &knownBlockVersions] (const Key &key) {
    migrateBlockFromBlockstoreWithoutVersionNumbers(baseBlockStore, key, &knownBlockVersions);
  });
  std::cout << "done" << std::endl;
}

void VersionCountingBlockStore2::migrateBlockFromBlockstoreWithoutVersionNumbers(blockstore::BlockStore2* baseBlockStore, const blockstore::Key& key, KnownBlockVersions *knownBlockVersions) {
  uint64_t version = knownBlockVersions->incrementVersion(key);

  auto data_ = baseBlockStore->load(key);
  if (data_ == boost::none) {
    LOG(WARN, "Block not found, but was returned from forEachBlock before");
    return;
  }
  cpputils::Data data = std::move(*data_);
  cpputils::Data dataWithHeader = _prependHeaderToData(key, knownBlockVersions->myClientId(), version, std::move(data));
  baseBlockStore->store(key, std::move(dataWithHeader));
}
#endif

}
}
