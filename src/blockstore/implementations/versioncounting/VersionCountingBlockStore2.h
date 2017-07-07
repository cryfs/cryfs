#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include "KnownBlockVersions.h"
#include "IntegrityViolationError.h"

namespace blockstore {
namespace versioncounting {

//TODO Format version headers

class VersionCountingBlockStore2 final: public BlockStore2 {
public:
  VersionCountingBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId, bool missingBlockIsIntegrityViolation)
      : _baseBlockStore(std::move(baseBlockStore)), _knownBlockVersions(integrityFilePath, myClientId), _missingBlockIsIntegrityViolation(missingBlockIsIntegrityViolation), _integrityViolationDetected(false) {
  }

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override {
    _checkNoPastIntegrityViolations();
    uint64_t version = _knownBlockVersions.incrementVersion(key);
    cpputils::Data dataWithHeader = _prependHeaderToData(_knownBlockVersions.myClientId(), version, data);
    return _baseBlockStore->tryCreate(key, dataWithHeader);
  }

  boost::future<bool> remove(const Key &key) override {
    _checkNoPastIntegrityViolations();
    _knownBlockVersions.markBlockAsDeleted(key);
    return _baseBlockStore->remove(key);
  }

  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override {
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

  boost::future<void> store(const Key &key, const cpputils::Data &data) override {
    _checkNoPastIntegrityViolations();
    uint64_t version = _knownBlockVersions.incrementVersion(key);
    cpputils::Data dataWithHeader = _prependHeaderToData(_knownBlockVersions.myClientId(), version, data);
    return _baseBlockStore->store(key, dataWithHeader);
  }

  uint64_t numBlocks() const override {
    return _baseBlockStore->numBlocks();
  }

  uint64_t estimateNumFreeBytes() const override {
    return _baseBlockStore->estimateNumFreeBytes();
  }

  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override {
    uint64_t baseBlockSize = _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
    if (baseBlockSize <= HEADER_LENGTH) {
      return 0;
    }
    return baseBlockSize - HEADER_LENGTH;
  }

  void forEachBlock(std::function<void (const Key &)> callback) const override {
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

private:
  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;

public:
  static constexpr uint64_t VERSION_ZERO = 0;
  static constexpr unsigned int CLIENTID_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER);
  static constexpr unsigned int VERSION_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t);
  static constexpr unsigned int HEADER_LENGTH = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t) + sizeof(VERSION_ZERO);

private:

  cpputils::Data _prependHeaderToData(uint32_t myClientId, uint64_t version, const cpputils::Data &data) {
    static_assert(HEADER_LENGTH == sizeof(FORMAT_VERSION_HEADER) + sizeof(myClientId) + sizeof(version), "Wrong header length");
    cpputils::Data result(data.size() + HEADER_LENGTH);
    std::memcpy(result.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
    std::memcpy(result.dataOffset(CLIENTID_HEADER_OFFSET), &myClientId, sizeof(myClientId));
    std::memcpy(result.dataOffset(VERSION_HEADER_OFFSET), &version, sizeof(version));
    std::memcpy((uint8_t*)result.dataOffset(HEADER_LENGTH), data.data(), data.size());
    return result;
  }

  void _checkHeader(const Key &key, const cpputils::Data &data) const {
    _checkFormatHeader(data);
    _checkVersionHeader(key, data);
  }

  void _checkFormatHeader(const cpputils::Data &data) const {
    if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data()) != FORMAT_VERSION_HEADER) {
      throw std::runtime_error("The versioned block has the wrong format. Was it created with a newer version of CryFS?");
    }
  }

  void _checkVersionHeader(const Key &key, const cpputils::Data &data) const {
    uint32_t clientId = _readClientId(data);
    uint64_t version = _readVersion(data);

    if(!_knownBlockVersions.checkAndUpdateVersion(clientId, key, version)) {
      integrityViolationDetected("The block version number is too low. Did an attacker try to roll back the block or to re-introduce a deleted block?");
    }
  }

  static uint32_t _readClientId(const cpputils::Data &data) {
    uint32_t clientId;
    std::memcpy(&clientId, data.dataOffset(CLIENTID_HEADER_OFFSET), sizeof(clientId));
    return clientId;
  }

  static uint64_t _readVersion(const cpputils::Data &data) {
    uint64_t version;
    std::memcpy(&version, data.dataOffset(VERSION_HEADER_OFFSET), sizeof(version));
    return version;
  }

  cpputils::Data _removeHeader(const cpputils::Data &data) const {
    return data.copyAndRemovePrefix(HEADER_LENGTH);
  }

  void _checkNoPastIntegrityViolations() const {
    if (_integrityViolationDetected) {
      throw std::runtime_error(std::string() +
                               "There was an integrity violation detected. Preventing any further access to the file system. " +
                               "If you want to reset the integrity data (i.e. accept changes made by a potential attacker), " +
                               "please unmount the file system and delete the following file before re-mounting it: " +
                               _knownBlockVersions.path().native());
    }
  }

  void integrityViolationDetected(const std::string &reason) const {
    _integrityViolationDetected = true;
    throw IntegrityViolationError(reason);
  }

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  mutable KnownBlockVersions _knownBlockVersions;
  const bool _missingBlockIsIntegrityViolation;
  mutable bool _integrityViolationDetected;

  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlockStore2);
};

}
}

#endif
