#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include "KnownBlockVersions.h"

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
    uint64_t version = blockStore->knownBlockVersions()->incrementVersion(key);
    cpputils::Data dataWithHeader = _prependHeaderToData(blockStore->knownBlockVersions()->myClientId(), version, data);
    return baseBlockStore_->tryCreate(key, dataWithHeader);
  }

  boost::future<bool> remove(const Key &key) override {
    _checkNoPastIntegrityViolations();
    _knownBlockVersions.markBlockAsDeleted(key);
    return baseBlockStore->remove(key);
  }

  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override {
    _checkNoPastIntegrityViolations();
    auto loaded = baseBlockStore_->load(key);
    loaded.then([this, key] (boost::future<boost::optional<cpputils::Data>> loaded_) {
      auto loaded = loaded_.get();
      if (boost::none == loaded) {
        if (_missingBlockIsIntegrityViolation && _knownBlockVersions.blockShouldExist(key)) {
          integrityViolationDetected("A block that should exist wasn't found. Did an attacker delete it?");
        }
        return boost::none;
      }
      if (!_checkHeader(key, *loaded)) {
        return boost::none;
      }
      return *loaded;
    });
  }

  boost::future<void> store(const Key &key, const cpputils::Data &data) override {
    _checkNoPastIntegrityViolations();
    TODO Need to load first so it can see if the version number changed by another
    THIS BUG IS ALSO IN THE NEXT BRANCH (i.e. without these changes here)
    //...
  }

  static constexpr uint64_t VERSION_ZERO = 0;
  static constexpr unsigned int CLIENTID_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER);
  static constexpr unsigned int VERSION_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t);
  static constexpr unsigned int HEADER_LENGTH = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t) + sizeof(VERSION_ZERO);

private:

  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;

  cpputils::Data VersionCountingBlock::_prependHeaderToData(uint32_t myClientId, uint64_t version, const cpputils::Data &data) {
    static_assert(HEADER_LENGTH == sizeof(FORMAT_VERSION_HEADER) + sizeof(myClientId) + sizeof(version), "Wrong header length");
    cpputils::Data result(data.size() + HEADER_LENGTH);
    std::memcpy(result.dataOffset(0), &FORMAT_VERSION_HEADER, sizeof(FORMAT_VERSION_HEADER));
    std::memcpy(result.dataOffset(CLIENTID_HEADER_OFFSET), &myClientId, sizeof(myClientId));
    std::memcpy(result.dataOffset(VERSION_HEADER_OFFSET), &version, sizeof(version));
    std::memcpy((uint8_t*)result.dataOffset(HEADER_LENGTH), data.data(), data.size());
    return result;
  }

  void _checkHeader(const Key &key, const cpputils::Data &data) {
    _checkFormatHeader(data);
    _checkVersionHeader(key, data);
  }

  void _checkFormatHeader(const cpputils::Data &data) {
    if (*reinterpret_cast<decltype(FORMAT_VERSION_HEADER)*>(data.data()) != FORMAT_VERSION_HEADER) {
      throw std::runtime_error("The versioned block has the wrong format. Was it created with a newer version of CryFS?");
    }
  }

  void _checkVersionHeader(const Key &key, const cpputils::Data &data) {
    uint32_t clientId;
    std::memcpy(&clientId, _dataWithHeader.dataOffset(CLIENTID_HEADER_OFFSET), sizeof(clientId));

    uint64_t version;
    std::memcpy(&version, _dataWithHeader.dataOffset(VERSION_HEADER_OFFSET), sizeof(version));

    if(!_knownBlockVersions.checkAndUpdateVersion(lastClientId, key, version)) {
      integrityViolationDetected("The block version number is too low. Did an attacker try to roll back the block or to re-introduce a deleted block?");
    }
  }

  void _checkNoPastIntegrityViolations() {
    if (_integrityViolationDetected) {
      throw std::runtime_error(string() +
                               "There was an integrity violation detected. Preventing any further access to the file system. " +
                               "If you want to reset the integrity data (i.e. accept changes made by a potential attacker), " +
                               "please unmount the file system and delete the following file before re-mounting it: " +
                               _knownBlockVersions.path().native());
    }
  }

  void integrityViolationDetected(const string &reason) const {
    _integrityViolationDetected = true;
    throw IntegrityViolationError(reason);
  }

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  KnownBlockVersions _knownBlockVersions;
  const bool _missingBlockIsIntegrityViolation;
  mutable bool _integrityViolationDetected;

  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlockStore2);
};

}
}

#endif
