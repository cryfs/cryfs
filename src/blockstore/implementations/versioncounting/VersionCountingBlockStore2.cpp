#include "VersionCountingBlockStore2.h"

namespace blockstore {
namespace versioncounting {

cpputils::Data VersionCountingBlockStore2::_prependHeaderToData(uint32_t myClientId, uint64_t version, const cpputils::Data &data) {
  static_assert(HEADER_LENGTH == sizeof(FORMAT_VERSION_HEADER) + sizeof(myClientId) + sizeof(version), "Wrong header length");
  cpputils::Data result(data.size() + HEADER_LENGTH);
  std::memcpy(result.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
  std::memcpy(result.dataOffset(CLIENTID_HEADER_OFFSET), &myClientId, sizeof(myClientId));
  std::memcpy(result.dataOffset(VERSION_HEADER_OFFSET), &version, sizeof(version));
  std::memcpy((uint8_t*)result.dataOffset(HEADER_LENGTH), data.data(), data.size());
  return result;
}

void VersionCountingBlockStore2::_checkHeader(const Key &key, const cpputils::Data &data) const {
  _checkFormatHeader(data);
  _checkVersionHeader(key, data);
}

void VersionCountingBlockStore2::_checkFormatHeader(const cpputils::Data &data) const {
  if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data()) != FORMAT_VERSION_HEADER) {
    throw std::runtime_error("The versioned block has the wrong format. Was it created with a newer version of CryFS?");
  }
}

void VersionCountingBlockStore2::_checkVersionHeader(const Key &key, const cpputils::Data &data) const {
  uint32_t clientId = _readClientId(data);
  uint64_t version = _readVersion(data);

  if(!_knownBlockVersions.checkAndUpdateVersion(clientId, key, version)) {
    integrityViolationDetected("The block version number is too low. Did an attacker try to roll back the block or to re-introduce a deleted block?");
  }
}

uint32_t VersionCountingBlockStore2::_readClientId(const cpputils::Data &data) {
  uint32_t clientId;
  std::memcpy(&clientId, data.dataOffset(CLIENTID_HEADER_OFFSET), sizeof(clientId));
  return clientId;
}

uint64_t VersionCountingBlockStore2::_readVersion(const cpputils::Data &data) {
  uint64_t version;
  std::memcpy(&version, data.dataOffset(VERSION_HEADER_OFFSET), sizeof(version));
  return version;
}

cpputils::Data VersionCountingBlockStore2::_removeHeader(const cpputils::Data &data) const {
  return data.copyAndRemovePrefix(HEADER_LENGTH);
}

void VersionCountingBlockStore2::_checkNoPastIntegrityViolations() const {
  if (_integrityViolationDetected) {
    throw std::runtime_error(std::string() +
                             "There was an integrity violation detected. Preventing any further access to the file system. " +
                             "If you want to reset the integrity data (i.e. accept changes made by a potential attacker), " +
                             "please unmount the file system and delete the following file before re-mounting it: " +
                             _knownBlockVersions.path().native());
  }
}

void VersionCountingBlockStore2::integrityViolationDetected(const std::string &reason) const {
  _integrityViolationDetected = true;
  throw IntegrityViolationError(reason);
}

VersionCountingBlockStore2::VersionCountingBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId, bool missingBlockIsIntegrityViolation)
: _baseBlockStore(std::move(baseBlockStore)), _knownBlockVersions(integrityFilePath, myClientId), _missingBlockIsIntegrityViolation(missingBlockIsIntegrityViolation), _integrityViolationDetected(false) {
}

boost::future<bool> VersionCountingBlockStore2::tryCreate(const Key &key, const cpputils::Data &data) {
  _checkNoPastIntegrityViolations();
  uint64_t version = _knownBlockVersions.incrementVersion(key);
  cpputils::Data dataWithHeader = _prependHeaderToData(_knownBlockVersions.myClientId(), version, data);
  return _baseBlockStore->tryCreate(key, dataWithHeader);
}

boost::future<bool> VersionCountingBlockStore2::remove(const Key &key) {
  _checkNoPastIntegrityViolations();
  _knownBlockVersions.markBlockAsDeleted(key);
  return _baseBlockStore->remove(key);
}

boost::future<boost::optional<cpputils::Data>> VersionCountingBlockStore2::load(const Key &key) const {
  _checkNoPastIntegrityViolations();
  return _baseBlockStore->load(key).then([this, key] (boost::future<boost::optional<cpputils::Data>> loaded_) {
    auto loaded = loaded_.get();
    if (boost::none == loaded) {
      if (_missingBlockIsIntegrityViolation && _knownBlockVersions.blockShouldExist(key)) {
        integrityViolationDetected("A block that should exist wasn't found. Did an attacker delete it?");
      }
      return boost::optional<cpputils::Data>(boost::none);
    }
    _checkHeader(key, *loaded);
    return boost::optional<cpputils::Data>(_removeHeader(*loaded));
  });
}

boost::future<void> VersionCountingBlockStore2::store(const Key &key, const cpputils::Data &data) {
  _checkNoPastIntegrityViolations();
  uint64_t version = _knownBlockVersions.incrementVersion(key);
  cpputils::Data dataWithHeader = _prependHeaderToData(_knownBlockVersions.myClientId(), version, data);
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

}
}
